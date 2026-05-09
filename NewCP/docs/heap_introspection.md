# Heap Introspection — Design

## Motivation

The compiler side of NewCP has deep introspection: `dump-tokens`, `dump-ast`, `dump-sema`, `dump-module-graph`, `dump-cfg`, `dump-ir`, `dump-llvm`, `dump-asm` all expose intermediate state as text. The loader exposes structured status: `LoaderSessionStatus` with active / retired generations, pinned execution scopes, failure phases, recovery state, all rendered through `LoaderSession::report()`.

The runtime side has no equivalent. The GC ([`gc.rs`](../src/newcp-runtime/src/gc.rs)) maintains a substantial internal state — clusters, blocks, type tags, mark bits, free lists, finalizers, module roots, conservative-scan stack base — but exposes none of it. The only signal a developer gets is "the program ran" or "the program crashed". We can't ask:

- how much memory is live;
- how many GC cycles have run, and how long they took;
- what the heap is shaped like (cluster occupancy, fragmentation);
- which modules registered global roots, and what those roots currently point at;
- how many instances of `TextModels.Run` are live;
- whether a particular object is still reachable, and from where.

This document specifies a `heap_introspect` module that closes that gap.

## Sources reviewed

- [`gc.rs`](../src/newcp-runtime/src/gc.rs) — `BlockHeader`, `TypeDesc`, `ModuleDesc`, `Cluster`, `GcState`, `__newcp_new_rec`, `__newcp_register_module`, `collect`, `mark_object`, `resolve_heap_ptr`.
- [`garbage-collection.md`](garbage-collection.md) — heap layout, mark/sweep cycle, multi-thread roadmap.
- [`newcp-loader/src/lib.rs`](../src/newcp-loader/src/lib.rs) — `LoaderSessionStatus`, `LoaderModuleStatus`, generation tracking, the existing rendering pattern.
- [`newcp-driver/src/main.rs`](../src/newcp-driver/src/main.rs) — subcommand dispatch (`dump-*` family) and how reports get rendered to stdout.

## Design goals

1. **Read-only by default.** Introspection must never mutate the heap. The snapshot path takes the GC lock, copies metadata, releases.
2. **Layered detail.** Cheap counters always-on; rich snapshots on demand; full graph walks only when the user asks for them.
3. **Match existing patterns.** Same surface shape as `LoaderSessionStatus`: a typed `HeapSnapshot` struct with `render()` for text and serde-friendly fields for JSON. Driver subcommand fits the `dump-*` family.
4. **No hot-path regressions.** Counter updates on the alloc/sweep paths use atomics with `Relaxed` ordering. No locks in hot path.
5. **Forward-compatible with multi-thread.** Snapshot is a stop-the-world-ish operation today (single mutex); the API does not assume single-threading and survives the per-thread `MutatorState` evolution from [`garbage-collection.md` §7](garbage-collection.md).
6. **Not a profiler, not a debugger.** Allocation sampling and source-level debugging are separate concerns. This module is for "what is the heap right now" diagnostics.

## Module shape

```
NewCP/src/newcp-runtime/src/heap_introspect.rs   # new module
NewCP/src/newcp-runtime/src/gc.rs                # add counters + snapshot hooks
NewCP/src/newcp-loader/src/lib.rs                # pass module names into __newcp_register_module
NewCP/src/newcp-driver/src/main.rs               # add `dump-heap` subcommand
NewCP/Mod/Diagnostics.cp                         # phase-2: CP-callable surface
```

`heap_introspect` is part of `newcp-runtime` because it depends on `GcState`. Other crates consume it as a public API; nothing else reaches into `gc.rs` directly.

## Three layers of introspection

### Layer 1 — Always-on counters

A `HeapCounters` struct of atomics, maintained on the alloc and sweep paths:

```rust
pub struct HeapCounters {
    // Lifetime totals
    alloc_blocks_lifetime: AtomicU64,
    alloc_bytes_lifetime: AtomicU64,
    free_blocks_lifetime: AtomicU64,
    free_bytes_lifetime: AtomicU64,

    // Path attribution (updated in __newcp_new_rec)
    bump_path_blocks: AtomicU64,
    free_list_path_blocks: AtomicU64,
    grow_events: AtomicU64,

    // Collection
    collect_cycles: AtomicU64,
    collect_total_nanos: AtomicU64,
    collect_last_nanos: AtomicU64,
    collect_last_reclaimed_bytes: AtomicU64,

    // Live (computed by sweep, not on hot alloc path)
    live_blocks: AtomicU64,
    live_bytes: AtomicU64,
    cluster_count: AtomicU64,
    module_root_count: AtomicU64,

    // Pressure
    peak_live_bytes: AtomicU64,
}
```

All updates are `Relaxed` except `collect_total_nanos` and the `peak_live_bytes` CAS, which need `AcqRel` to make a counter read inside a `take_snapshot()` consistent with the cluster walk that follows.

Hot-path cost: two atomic `fetch_add` per allocation (one for blocks, one for bytes), branch-distinguished by alloc path. On x86_64 these compile to `lock add` instructions — measurable on micro-benchmarks but invisible at app scale, and removable behind a `#[cfg(feature = "heap-counters")]` flag if it ever shows up in profiles.

API:

```rust
pub fn current_counters() -> HeapCounters;          // copies atomically
pub fn reset_counters();                            // for tests / dev only
```

`reset_counters` is intended for the test harness and `dump-heap --reset` so successive runs of the same scenario start from zero. Production / CP code never calls it.

### Layer 2 — Heap snapshot

Locks the GC, walks every cluster + module + type, returns owned structured data. Equivalent to `LoaderSessionStatus`:

```rust
pub struct HeapSnapshot {
    pub taken_at_ns_since_epoch: u64,
    pub counters: HeapCounters,
    pub clusters: Vec<ClusterSnapshot>,
    pub modules: Vec<ModuleRootSnapshot>,
    pub types: Vec<TypeSnapshot>,
}

pub struct ClusterSnapshot {
    pub index: usize,
    pub base: usize,            // numeric address; never deref'd by consumers
    pub size: usize,
    pub bump: usize,
    pub live_blocks: u64,
    pub live_bytes: u64,
    pub free_blocks: u64,
    pub free_bytes: u64,
    pub largest_free_block: u64,
    pub fragmentation_ratio: f32,   // free_blocks > 1 ? 1 - (largest / total_free) : 0
}

pub struct ModuleRootSnapshot {
    pub name: String,           // resolved from TypeDesc.module → ModuleDesc
    pub var_base: usize,
    pub offset_count: usize,
    pub offsets: Vec<isize>,
    pub current_pointer_values: Vec<usize>,   // *((var_base + offsets[i]) as *const usize)
}

pub struct TypeSnapshot {
    pub display_name: String,   // "TextModels.Run" — best-effort, see naming below
    pub type_desc_addr: usize,
    pub size: isize,
    pub vtable_len: u64,
    pub has_finalizer: bool,
    pub instance_count: u64,
    pub instance_bytes: u64,
}
```

Public API:

```rust
pub fn take_snapshot() -> HeapSnapshot;
pub fn take_snapshot_lite() -> HeapSnapshot;   // skip per-block walk; counters + cluster summary only
```

`take_snapshot_lite` is for live dashboards / `Kernel.HeapStats` calls. The full snapshot's per-block walk is `O(heap)` and proportional to total allocated memory; lite is `O(clusters + modules + types)`.

Implementation note: the type catalog is built by walking blocks and bucketing by `BlockHeader.tag & !MARK_BIT`. The first time we see a `TypeDesc` we resolve its display name (see "Type naming" below). The result is cached for the duration of the snapshot.

### Layer 3 — Object graph walk

For "show me what is keeping this object alive" debugging:

```rust
pub fn walk_roots(mut visit: impl FnMut(RootInfo));
pub fn walk_objects(mut visit: impl FnMut(ObjectInfo));
pub fn walk_reachable_from(root: usize, mut visit: impl FnMut(EdgeInfo));

pub struct RootInfo {
    pub kind: RootKind,        // Module { module_name, offset } | StackWord { address }
    pub points_to: Option<usize>,   // resolved payload address, if heap-pointer
}

pub struct ObjectInfo {
    pub payload: usize,
    pub block_size: usize,
    pub type_desc_addr: usize,
    pub type_name: String,
    pub child_count: usize,
    pub is_marked: bool,
}

pub struct EdgeInfo {
    pub from_payload: usize,
    pub to_payload: usize,
    pub field_offset: isize,
}
```

These are heavy, take the GC lock for their full duration, and are intended for `dump-heap --graph` / interactive debugging — not for steady-state monitoring.

`walk_reachable_from` is the algorithm of `mark_object` with the side effect replaced by a callback: it traces precisely through `TypeDesc.ptroffs`, never via the conservative stack scan. This means it answers "what does object X own", not "who owns object X" — the latter requires a reverse index, which we are not building (a full graph dump can be post-processed externally if a user needs the inverse direction).

## Hooks needed in existing code

### `gc.rs`

1. **Add `HeapCounters` static.** Wire updates into `__newcp_new_rec` (allocation path) and `Cluster::sweep` (free + finalizer path).
2. **Time `collect()`.** Wrap `collect_inner` with `Instant::now()`; record the elapsed nanos on the global counter. Cheap; only fires on collect.
3. **Add a name field to `ModuleRoots`.** Today the registration is anonymous (`__newcp_register_module(var_base, offsets, count)`). The legitimate consumer of this — the loader — knows the name. Add a parallel registration:

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __newcp_register_module_named(
    name_utf8: *const u8,
    name_len: usize,
    var_base: *const u8,
    offsets_ptr: *const isize,
    count: usize,
);
```

The unnamed entry stays for legacy / direct callers; it stores `"<unnamed>"` as the module name. The loader migrates to the named form.

4. **Expose a snapshot accessor.** Define a private function in `gc.rs` that takes the lock and runs a closure with `&GcState`:

```rust
pub(crate) fn with_locked_state<R>(f: impl FnOnce(&GcState, &HeapCounters) -> R) -> R;
```

`heap_introspect` calls this; no other crate has access. The `pub(crate)` keeps `GcState` itself private.

### `newcp-loader`

When the loader registers a module's globals, pass the module name into `__newcp_register_module_named`. Today the loader does not register module roots at all (the GC has them defined but the runtime hasn't wired the loader-side call yet — see the registration site for compiled modules). When that wiring lands, it uses the named entry from day one.

### `newcp-driver`

Add `dump-heap` to the subcommand list. Modes:

```text
newcp-driver dump-heap                       # full text report
newcp-driver dump-heap --counters            # just the counters block
newcp-driver dump-heap --clusters            # cluster summary
newcp-driver dump-heap --types               # type catalog
newcp-driver dump-heap --roots               # module roots + current pointer values
newcp-driver dump-heap --graph [--from <addr>]  # object graph in DOT
newcp-driver dump-heap --json                # machine-readable snapshot
newcp-driver dump-heap --after <command>     # bootstrap, run command, then dump
```

The `--after` form is the one that gets used most: bootstrap the loader, invoke a command, take the snapshot, print. That's the workflow for "did running X leak memory?".

## Type naming

`TypeDesc` has `module: *const ModuleDesc` (line 127 of `gc.rs`), but no name field. The legacy CP runtime resolves a `Kernel.Type` to its name through `Kernel.GetTypeName(t, name)` and `t.mod.name`. For NewCP we have two options:

1. **Co-emit a names table.** When `newcp-llvm` synthesises a `TypeDesc`, also emit a `*const u8` / `usize` pair pointing at a static UTF-8 name. Adds 16 bytes per TypeDesc. Trivial to consume, exact, no runtime cost.
2. **Look up via the loader.** The loader knows every TypeDesc address it materialised and the type name it came from. `heap_introspect` consults a `(TypeDesc* → name)` registry maintained alongside the existing module registry.

Option 1 is simpler and is the recommended path. It also lines up with the eventual `Kernel.GetTypeName` implementation — the same name field serves both. Until codegen emits it, `TypeSnapshot.display_name` falls back to `format!("Type@0x{:x}", type_desc_addr)`.

## Output formats

### Text report

Mirrors the loader's `render()` style: line-oriented, one fact per line, indented blocks for sub-structure. Example:

```text
newcp heap snapshot
target: x86_64-pc-windows-msvc

counters:
  alloc-lifetime:        12,847 blocks   1,213,648 bytes
  free-lifetime:          9,322 blocks     872,144 bytes
  live:                   3,525 blocks     341,504 bytes
  peak-live:                              412,288 bytes
  cluster-count: 2
  module-roots: 12
  collect-cycles: 4 (last 0.31 ms, total 1.87 ms, last reclaimed 287 KiB)
  alloc-paths: bump 11,402 / free-list 1,445 / grow-events 1

clusters:
  #0  base 0x16ba0000  size 1.00 MiB  bump 768 KiB
      live 311 KiB (3,012 blocks)  free 457 KiB (3 blocks, largest 432 KiB, frag 0.05)
  #1  base 0x16cb0000  size 1.00 MiB  bump 320 KiB
      live 30 KiB (513 blocks)     free 290 KiB (1 block)

types (top by live bytes):
  Stores.StdDocument        instances 1      bytes 768   finalizer no
  TextModels.StdModel       instances 1      bytes 512   finalizer no
  TextModels.Run            instances 247    bytes 19,760
  TextModels.Attributes     instances 18     bytes 1,440
  Files.File                instances 3      bytes 192   finalizer YES

module roots:
  Kernel        var_base 0x000014a8  offsets [16, 24, 32]
  Stores        var_base 0x000014d0  offsets [8]
  TextModels    var_base 0x000014e8  offsets [0, 8, 16, 24]
  ...
```

### JSON

Direct `serde_json::to_string_pretty(&snapshot)`. The `HeapSnapshot` struct derives `Serialize` (behind `#[cfg(feature = "json")]` to keep `serde_json` out of the runtime's default dependency set).

### DOT

For `--graph`, emit a digraph: nodes are object payloads (labelled with type name + short address), edges are pointer fields (labelled with offset). Suitable for `graphviz` or a downstream visualiser. Unconditional; no extra deps.

## CP-callable surface (Phase 2)

Once the FFI shape is shaken out, expose a small `Diagnostics` module:

```
Diagnostics.HeapStats(
    OUT live, free, peak, cycles, lastNanos: INTEGER
);
Diagnostics.HeapDump(IN file: ARRAY OF CHAR);   (* writes a JSON snapshot *)
Diagnostics.CollectAndReport(VAR reclaimedBytes: INTEGER);
```

This is what `Kernel.Stats` was in BlackBox, brought up under a different module name so we don't collide with the legacy `Kernel` API surface during bootstrap. The CP-callable surface is **phase 2** — Phase 1 is pure Rust + driver subcommand, which already covers every real workflow short of "from inside a running CP program".

## Phasing

| Phase | Scope |
|---|---|
| **H1** | `HeapCounters` static + atomic updates in `__newcp_new_rec` and `Cluster::sweep`. `current_counters()` + `reset_counters()`. Driver: `dump-heap --counters`. |
| **H2** | Cluster + module-root snapshot (no per-block walk). `take_snapshot_lite`. Driver: `dump-heap --clusters` and `dump-heap --roots`. |
| **H3** | Per-block walk + type catalog. `take_snapshot`. Driver: `dump-heap`, `dump-heap --types`. Requires named module registration and (for sane output) co-emitted type names from `newcp-llvm`. |
| **H4** | Object graph walks. Driver: `dump-heap --graph`. JSON output via optional feature. |
| **H5** | CP-callable `Diagnostics` module. |

Each phase ships independently and is observable from the driver before the next one starts. H1 is roughly two days of work; H3 depends on the codegen change for type names; H5 depends on the FFI binding pattern from the Stores module bring-up.

## Tests

Reuse the existing `gc.rs` test harness shape (acquire `TEST_LOCK`, `reset_gc()`, allocate, assert). New tests:

- **counters track allocation**: allocate `N` records, assert `alloc_blocks_lifetime == N` and `alloc_bytes_lifetime == N * total_block_size`.
- **counters track sweep**: allocate then drop refs then `collect()`; assert `free_blocks_lifetime` increased by the survivors that died.
- **snapshot under load**: allocate a known mix; assert per-cluster `live_bytes` sums to a known value.
- **module-root resolution**: register a named module, allocate a record, store its payload at the registered offset, assert `take_snapshot().modules[i].current_pointer_values[0]` equals that payload address.
- **type catalog**: allocate K leaves of one type and J leaves of another; assert exact instance counts.
- **graph walk roundtrip**: build a small known graph (parent + N leaves wired through `wire_parent` from the existing tests), call `walk_reachable_from(parent)`, assert exactly the right edges.

## Non-goals (explicit)

- **Allocation sampling / heap profiling.** The chokepoint at `__newcp_new_rec` is the right place for it later, but per-allocation backtrace capture is a separate module.
- **Reverse-pointer index ("who owns X?").** Useful but expensive; left as a post-processing step on a `--graph` JSON dump.
- **Live mutation.** No `force_collect_now`, no `pin_object`, no `evict_cluster`. Introspection is read-only by contract.
- **Multi-thread snapshots while mutators run.** The MVP runtime is single-threaded; the API takes the GC lock and runs to completion. The multi-thread design in `garbage-collection.md` §7.3.2 already pencils in cooperative safepoints; once those exist, `take_snapshot` becomes a safepoint operation, but the consumer-visible API does not change.

## Summary

Three layers — counters, snapshot, graph — built directly on the existing GC bookkeeping with minimal additions: atomic counters, a named module-registration entry point, a co-emitted type name on each `TypeDesc`. Output integrates into the driver's `dump-*` family and matches the loader's `render()` style. Phased so each step ships an observable improvement before the next one begins.
