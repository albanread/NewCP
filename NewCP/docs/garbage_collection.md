# NewCP Garbage Collection (v2)

This is the design contract for the GC. Code that diverges from this
document is wrong; the document and the code stay in sync.

The previous design (`docs/archive/garbage-collection-v1.md`,
`docs/archive/gc_v1.rs`) was single-thread by construction and broke
silently the first time we hit non-trivial heap pressure with multiple
test threads. v2 starts over.

---

## Goals

1. **Correctness, even under stress.** No use-after-free, no missed
   roots, no stale-TypeDesc dereferences.
2. **Multi-threaded mutators.** Every CP-managed thread can allocate,
   read, and write the heap concurrently. Tests can run in parallel.
3. **Stop-the-world collection.** Mutators cooperatively park while
   the collector runs. No write barriers, no concurrent marking, no
   compaction in v2.
4. **Transparent.** Every state change is observable. There is a
   `dump-gc` command that prints the full state of every heap, every
   thread, every TypeDesc, and the recent allocation/collection
   history. Tests can pin the GC to deterministic behaviour.
5. **Robust against module retirement.** A loaded module's TypeDescs
   stay alive until every block tagged with them is reclaimed. Module
   drop is gated explicitly on this, not on a quiescent-epoch
   heuristic.

## Non-goals

Generational, concurrent marking, compaction, sub-millisecond pauses,
write barriers, finalizer threads, low-fragmentation allocator. We may
add any of these later — but not until they're driven by a concrete
need.

---

## Architecture overview

```
        ┌──────────────────────────────────────────────────────────┐
        │  Mutator threads (any number, registered explicitly)     │
        │                                                          │
        │   Thread A         Thread B         Thread C             │
        │   ───────────      ───────────      ───────────          │
        │   state            state            state                │
        │   stack_top        stack_top        stack_top            │
        │   tlab cursor      tlab cursor      tlab cursor          │
        │   parked_sp        parked_sp        parked_sp            │
        │   spill[16]        spill[16]        spill[16]            │
        │      │ TLAB miss      │ TLAB miss      │ TLAB miss       │
        └──────┼────────────────┼────────────────┼─────────────────┘
               ▼                ▼                ▼
        ┌──────────────────────────────────────────────────────────┐
        │  Heap (one global Mutex)                                 │
        │                                                          │
        │  Clusters: [block][block][free][block][block]…           │
        │  Free-list head per cluster                              │
        │  Block-start bitmap per cluster                          │
        │  TypeDesc registry: addr → { ref_count, owner_module }   │
        │  Module roots:      var_base + ptr offsets               │
        │  Allocation log     (ring buffer, off by default)        │
        │  Collect log        (every cycle)                        │
        └──────────────────────────────────────────────────────────┘
```

Two locks, in this order to avoid deadlock:
1. `THREADS` — `RwLock`. Read-locked by collector while iterating
   mutators; write-locked by `register_thread` / `unregister_thread`.
2. `HEAP` — `Mutex`. Held during TLAB refill, large alloc, sweep,
   `register_module`, and during the entire collect cycle.

Never hold `HEAP` while waiting on `THREADS` or vice versa.

---

## Data structures

### BlockHeader (unchanged from v1, JIT codegen depends on it)

```
+0  tag         usize     [bit 0 = mark; bits 1..47 = TypeDesc addr]
+8  block_size  usize     [total block bytes incl. header, 16-aligned]
```

A `tag` whose value (mark bit cleared) is 0 marks a free block; the
payload then begins with a `FreeBlockLink` to the next free block.

### TypeDesc (unchanged from v1; codegen-emitted)

```
+0   size        isize     [payload bytes, header excluded]
+8   module      ptr       [owning ModuleDesc or null]
+16  finalizer   ptr       [optional fn(*mut u8)]
+24  base        ptr       [direct base TypeDesc or null]
+32  vtable      ptr       [array of method fn ptrs]
+40  vtable_len  u64
+48  name        ptr       [UTF-32 NUL-terminated qualified name]
+56  ptroffs[]   isize…    [sentinel-terminated pointer offsets]
```

### Mutator (one per registered thread)

```rust
struct Mutator {
    thread_id:  ThreadId,
    stack_top:  usize,        // highest live stack address (set at register-time)
    state:      AtomicU8,     // Running | SafepointRequested | Parked
    parked_sp:  AtomicUsize,  // valid only when state == Parked
    spill:      UnsafeCell<[usize; 16]>,
    tlab:       UnsafeCell<Tlab>,

    // Per-thread accounting for introspection.
    alloc_blocks_lifetime: AtomicU64,
    alloc_bytes_lifetime:  AtomicU64,
    park_count:            AtomicU64,
}
```

State transitions:
- `Running` → `Parked`: mutator polls a safepoint flag, transitions
  itself, blocks on a condvar.
- `Parked` → `Running`: collector signals after sweep; mutator
  transitions and resumes.
- The collector never modifies a mutator's state except as failsafe.

### Tlab — Thread-Local Allocation Buffer

```rust
struct Tlab {
    cursor: *mut u8,    // next free byte
    end:    *mut u8,    // one past last byte
}
```

Allocation fast-path: `cursor + size <= end` ⇒ bump and return. Else
slow path takes the heap lock and refills.

Default TLAB size is 64 KB. Allocations larger than `TLAB_BIG_LIMIT`
(currently 16 KB) bypass the TLAB and go to a direct heap path.

### TypeDescRegistry

```rust
struct TypeDescRegistry {
    entries: HashMap<usize, TypeDescEntry>,
}
struct TypeDescEntry {
    block_count: u64,            // # of live heap blocks tagged with this TD
    owner_module: Option<String>,
    pinned_until_zero: bool,     // module retirement waits on this
}
```

- `__newcp_register_type` inserts.
- `__newcp_new_rec` increments `block_count` after tagging.
- Sweep decrements `block_count` for every reclaimed block.
- Module retirement: loader queries `every TD's block_count == 0` for
  each owned TD; if not all zero, image stays retired-pending.

### Heap

```rust
struct Heap {
    clusters:        Vec<Cluster>,
    type_descs:      TypeDescRegistry,
    modules:         Vec<ModuleRoots>,
    alloc_log:       AllocLog,        // ring buffer, optional
    collect_log:     CollectLog,      // last N cycles
}
```

`Cluster`'s structure stays close to v1 (base, bump, free_list,
block-starts bitmap) but with the layout of one minimum-allocation
slot per `BLOCK_ALIGN`-aligned position.

---

## Allocation

```
__newcp_new_rec(td) -> *mut u8 {
    let m = current_mutator();
    let total = total_block_size(td.size);

    // Fast path: TLAB bump.
    if m.tlab.cursor + total <= m.tlab.end:
        ptr = m.tlab.cursor
        m.tlab.cursor += total
        write_header(ptr, td)
        type_descs.entry(td).block_count.fetch_add(1)
        return ptr.payload()

    // Slow path: TLAB miss.
    return slow_alloc(td, total)
}

slow_alloc(td, total) -> *mut u8 {
    if total > TLAB_BIG_LIMIT:
        return alloc_large(td, total)   // direct heap

    if heap_pressure_high():
        request_collect()                // STW path

    refill_tlab(m, total)                // grabs heap lock briefly
    return __newcp_new_rec(td)           // retry; guaranteed to fit
}
```

Counters update on every allocation, regardless of path.

---

## Safepoints

A **safepoint** is a point in JIT-emitted code where every thread
is in a known parkable state: registers spilled, no in-flight write
to a GC-managed location.

Three sources of safepoints in v2:

1. **Allocation slow-path.** When a mutator takes the heap lock to
   refill its TLAB, it implicitly enters a safepoint — registers are
   spilled to the spill buffer first, sp captured, state set to
   `Parked` while we hold the lock.

2. **Explicit poll.** `__newcp_safepoint()` — currently a no-op stub.
   v2 makes it real: load the global flag; if set, park. Codegen
   emits one at every function entry that allocates and at every
   long-running loop's back-edge.

3. **Already-parked threads.** A thread that's blocked in a system
   call (file I/O, condvar wait) is implicitly at a safepoint as long
   as the call doesn't return into managed code without polling first.
   We require explicit `enter_safepoint()` / `leave_safepoint()`
   bracketing around any blocking syscall.

Until codegen emits explicit polls, source #2 produces no safepoints
and a tight allocation-free loop on a mutator can stall the collector.
That's a known limitation of phase 1; it's only a *liveness* issue,
not a correctness one. (A thread that doesn't allocate can't
introduce new garbage either, so it's never the thread the collector
needs to wait on for safety.)

The collector waits up to a deadline (1 s default) for all mutators
to park. If it can't, it logs the offenders and aborts the cycle —
better than corrupting state.

---

## Collection cycle

```
collect():
    THREADS.read_lock()                        // freeze membership
    HEAP.lock()
    SAFEPOINT_REQUESTED.store(1)
    for m in THREADS:
        wait until m.state == Parked
    // All mutators parked, holding THREADS read lock and HEAP lock.

    clear_marks()                              // every cluster
    for m in THREADS:
        scan_thread_stack(m.parked_sp, m.stack_top, m.spill)
    for module in modules:
        scan_module_roots(module)
    // Mark phase done.

    sweep()                                     // per cluster, decrements td.block_count
    SAFEPOINT_REQUESTED.store(0)
    SAFEPOINT_CONDVAR.notify_all()
    HEAP.unlock()
    THREADS.read_unlock()
```

The mark phase walks each thread's stack from `parked_sp` (deepest
live address) up to `stack_top`. Conservative: every word that
resolves to a heap-block payload becomes a root.

`scan_thread_stack` must include the spill buffer — the values that
were in callee-saved registers at park time. The buffer lives on the
mutator's own stack, so it's already covered if `parked_sp` ≤
`spill_buf_start`. The mutator's park entry guarantees this.

---

## TypeDesc lifetime

The bug we hit: a JIT image got dropped while live blocks were still
tagged with TypeDescs in that image's memory. v2 fixes it explicitly:

1. Codegen emits `__init_types` which calls
   `__newcp_register_type(name, td)` for each TypeDesc the module
   owns.
2. The registry creates a `TypeDescEntry { block_count: 0,
   owner_module: Some(name), pinned_until_zero: true }`.
3. `__newcp_new_rec(td)` increments `block_count`.
4. Sweep, when reclaiming a block whose tag points at `td`,
   decrements `block_count`. After decrement, if the entry is
   `pinned_until_zero` and `block_count == 0` and `owner_module`
   has been requested for retirement, the loader is notified.
5. The loader's `RetiredImageDropPredicate` returns `true` only when
   every TD owned by the module has `block_count == 0`.

`__newcp_lookup_typedesc` (the cross-module NEW path) and the
existing `Kernel.ThisType` reflection both go through the same
registry. The registry is the single source of truth for "is this
TypeDesc address still live?".

---

## Introspection

A new module `gc_introspect` (replacing `heap_introspect` in scope and
expanding it). It exposes:

```rust
pub struct GcSnapshot {
    pub generation:        u64,
    pub clusters:          Vec<ClusterSnapshot>,
    pub threads:           Vec<ThreadSnapshot>,
    pub type_descs:        Vec<TypeDescSnapshot>,
    pub modules:           Vec<ModuleSnapshot>,
    pub counters:          GlobalCounters,
    pub recent_collects:   Vec<CollectRecord>,
    pub recent_allocs:     Option<Vec<AllocRecord>>,    // when alloc-log enabled
}
```

Driver subcommands:
- `dump-gc`           — full snapshot, pretty-printed.
- `dump-gc --json`    — same, JSON.
- `dump-gc --threads` — just the thread table.
- `dump-gc --types`   — just the TypeDesc registry.
- `dump-gc --collect` — force a collection then dump.

Environment toggles (must be off in production runs):
- `NEWCP_GC_LOG_ALLOCS`     — record every allocation in the ring buffer.
- `NEWCP_GC_LOG_COLLECTS=N` — keep last `N` collect records (default 8).
- `NEWCP_GC_TRACE`          — verbose trace to stderr (alloc, park, mark, sweep).

Every state mutation that's interesting goes through one helper
function (`emit_event`) so trace output is consistent and one place
controls verbosity.

---

## ABI surface (must remain compatible with codegen)

These symbols keep their signatures and semantics:

```
__newcp_init_gc(stack_base: *const u8)
__newcp_register_thread(stack_base: *const u8)        [NEW in v2]
__newcp_unregister_thread()                            [NEW in v2]
__newcp_register_module(var_base, offsets, count)
__newcp_register_module_named(name_utf8, name_len, var_base, offsets, count)
__newcp_register_type(name_utf8, name_len, td)
__newcp_lookup_typedesc(name_utf8) -> i64
__newcp_new_rec(td) -> *mut u8
__newcp_sys_new(n) -> *mut u8
__newcp_safepoint()                                    [becomes real]
__newcp_trap(code)
__newcp_unimpl_method_trap()
__newcp_string_eq_char(a, b) -> i64
__newcp_string_eq_shortchar(a, b) -> i64
```

`BlockHeader` layout is unchanged. `TypeDesc` layout is unchanged.
`ModuleDesc` layout is unchanged. The JIT does not need recompilation
to switch from v1 to v2; the C ABI is preserved.

---

## Migration plan

The rewrite lands in one commit. We don't ship v1 and v2 side-by-side
because the lock structure differs and the data is global. The order
of operations is:

1. Move `gc.rs` to `docs/archive/gc_v1.rs`. (Done.)
2. Move `garbage-collection.md` and the notes file to
   `docs/archive/`. (Done.)
3. Write this document. (Done.)
4. Write new `gc.rs` against the design above.
5. Update `heap_introspect.rs` to use the new snapshot types (or
   replace it with `gc_introspect.rs`).
6. Update `kernel_sys.rs` and `lib.rs` to expose the new entry points.
7. Bring up the test suite. Existing tests must pass with
   `TextBufferChars = 65536` (the failure that motivated this work).
8. Add a multi-threaded stress test: N threads × M allocations,
   collector cycles in between.

## Open questions

- Should TLAB size auto-tune based on collection frequency? Probably
  later; start with a fixed 64 KB.
- Should we emit safepoint polls eagerly (every loop back-edge) or
  lazily (only when the JIT can't bound a loop)? Pick the lazy default
  once we have a multi-threaded mutator program to stress.
- The sweep currently runs in the calling thread (the collector
  thread). Move to a dedicated finalizer thread later if finalizer
  cost grows.
