# JIT & heap improvements — adopting from NewCormanLisp

*Drafted 2026-05-12.  Status: review complete, no code yet.*

This is a plan for two NCL ideas we surveyed.  One we adopt
wholesale; one we explicitly defer.  The whole document is here so
the *reason* for adopting one and not the other survives.

## Background: what NCL did

NCL (`E:\CL\NewCormanLisp`) is a parallel Rust-on-LLVM JIT
project — Common Lisp instead of Component Pascal, but the same
runtime + ncl-ir + ncl-llvm + ncl-loader + ncl-runtime shape we
have.  They hit two problems we will also hit and solved both:

1. Rust panics raised inside a runtime helper need to unwind
   cleanly through any depth of JIT'd frames back to a
   `catch_unwind` boundary in the host.  On Windows that's a
   four-piece machine (uwtable + custom MCJIT memory manager +
   `RtlAddFunctionTable` + `extern "C-unwind"`); each piece is
   silently broken without the other three.  Their writeup of
   the bug-hunt is `E:\CL\NewCormanLisp\docs\the_seg_windup.md`
   and worth reading before any of the work below.
2. Multi-threaded allocation through a single global heap lock
   serialises every cons cell.  Their answer is per-thread
   TLABs sitting on a copying generational young heap; allocation
   becomes a two-instruction bump-and-compare with no atomics.

We adopt (1) on the next pass.  We do not adopt (2) and the rest
of this document is the reason why.

## Part 1 — adopt: unwinding through JIT frames

### The current state

Survey of our tree:

```
$ rg -c 'extern "C"'    NewCP/src/newcp-runtime/src
total: 217

$ rg 'uwtable|RtlAddFunctionTable|MCJITMemoryManager'    NewCP/src
(no matches)
```

Concretely:

- `module.create_jit_execution_engine(opt_level)` is called via
  inkwell's high-level wrapper in
  `newcp-llvm/src/jit.rs`.  Inkwell 0.9 doesn't expose the
  `MCJMM` slot on `LLVMMCJITCompilerOptions`, so we cannot
  install a custom memory manager from where we sit today.
- No JIT'd function carries the `uwtable` attribute.  Even if
  we wired up `RtlAddFunctionTable`, there would be nothing to
  register.
- 217 `pub extern "C"` declarations in `newcp-runtime` —
  every helper bound into the JIT via `add_global_mapping`.
  Plain `extern "C"` on Rust 1.71+ inserts a `__fastfail` guard
  at the boundary; a panicking escape surfaces as
  `STATUS_STACK_BUFFER_OVERRUN` (0xC0000409), which looks
  *exactly* like a stack-corruption bug.  This is a tripwire
  that nobody has stepped on only because we have no
  panic-from-runtime path.
- `__newcp_trap(code)` aborts the process.  Array-bounds checks,
  CP `ASSERT` failures, and the `HALT(n)` instruction all
  funnel through it.  There is no host catch boundary.

### What we'd build, in dependency order

Each step is independently testable and produces an artefact
the next step needs.

**Step 1.  `uwtable=2` on every JIT-emitted function.**

In `newcp-llvm/src/module.rs` (wherever we set
`FunctionValue` attributes on emitted procedures), add the
`uwtable` enum attribute with value 2.  Async unwind tables
are usable at any PC, not only call sites; uwtable=1 is enough
for synchronous panic-at-call-site unwinds but LLVM 22 elides
unwind info for "leaf-ish" sequences with =1.  Setting =2 is
the safe default for any function that might be unwound
through, which for us is every CP procedure.

Verification: `NEWCP_DUMP_IR=1` (or whichever env we wire) on
a sample probe should emit a `.obj` whose `dumpbin /headers`
shows `.pdata` and `.xdata` sections populated.

**Step 2.  Audit `extern "C"` and switch to `extern "C-unwind"`.**

Every Rust helper bound into the JIT engine via
`add_global_mapping` is on the panic propagation path and must
be `extern "C-unwind"`.  Plain `extern "C"` becomes a silent
`__fastfail` trap.  217 hits total in `newcp-runtime`; not all
of them are JIT-callable (some are Rust↔Rust shims between
modules), so categorise first:

  - JIT-bound (`add_global_mapping` callee) → `extern "C-unwind"`.
  - Internal Rust-only → drop the `extern "C"`, just `pub fn`.
  - Function-pointer typedefs in `newcp-llvm` for callbacks
    bound into the engine → `extern "C-unwind"`.

The lint we want: every `pub extern "C"` in `newcp-runtime`
that's bound into the engine must be `-unwind`.  Worth a
single comment marker at the top of `lib.rs` listing the
contract; a grep-able invariant beats a runtime trap.

**Step 3.  Custom MCJIT memory manager
(`newcp-llvm/src/jit_mm.rs`).**

This is the load-bearing piece.  Mostly a verbatim port of
NCL's file (`E:\CL\NewCormanLisp\src\ncl-llvm\src\jit_mm.rs`,
489 lines).  The Windows API surface is identical; the LLVM
section-allocation callbacks are identical; the RuntimeDyld
behaviour we have to work around is identical.  Five things
the port has to get right:

  - **Single contiguous reservation per module.**
    `VirtualAlloc(MEM_RESERVE, PAGE_NOACCESS)` for some envelope
    big enough to hold all of `.text`/`.pdata`/`.xdata` for the
    largest CP module we might emit.  NCL uses 4 MiB; pick
    8–16 MiB for us — TextModels with full method bundles is
    bigger than a typical Lisp function.  All sections must
    sit within u32 RVA of each other for
    `IMAGE_REL_AMD64_ADDR32NB` relocs to resolve.
  - **Bump-allocate sections inside the reservation,
    page-aligned, page-committed on demand.**  Page alignment
    makes `VirtualProtect` to `PAGE_EXECUTE_READ` on finalize
    a one-shot flip per code section.
  - **Capture `.text` / `.pdata` / `.xdata` by exact name.**
    Match on the C-string passed by LLVM's section callback.
  - **On finalize, filter zero-padded `.pdata` slots in
    place.**  LLVM/RuntimeDyll over-allocates `.pdata`; the
    trailing all-zero `RUNTIME_FUNCTION` entries all share
    `BeginAddress=0`, and `RtlLookupFunctionEntry`'s binary
    search lands on one of them, sees `EndAddress=0`, and
    skips the frame's unwind — leaving RSP unrestored.  The
    next `/GS` canary then fast-fails as 0xC0000409.  Partition
    live entries (Begin < End) to the front, sort by
    `BeginAddress`, register only the live count.  NCL paid a
    full day to find this; we get to skip that day.
  - **Sanity-check before calling `RtlAddFunctionTable`.**
    `base + first_entry.BeginAddress` must land inside the
    captured `.text` range.  If not, log and skip the
    registration — better a panic-fails-loudly outcome than a
    silent table corrupting the global SEH dispatcher.

Module retirement: NCL leaks for the process lifetime; we
match that for v1.  When the loader gains real retirement,
`destroy()` pairs `RtlDeleteFunctionTable` + `VirtualFree` on
each captured allocation.

**Step 4.  Drop one layer below inkwell.**

In `newcp-llvm/src/jit.rs::from_module`, replace
`module.create_jit_execution_engine(opt_level)` with the
llvm-sys triad:

```rust
LLVMLinkInMCJIT();
LLVMInitializeMCJITCompilerOptions(&mut opts, size);
opts.MCJMM = jit_mm::make_mm();
LLVMCreateMCJITCompilerForModule(&mut engine, mod_ptr, &opts, size, &err);
```

Re-implement `add_global_mapping` calls via
`LLVMAddGlobalMapping(engine, function.as_value_ref(), addr)`.
This is the most invasive of the five steps because it
changes the `engine` handle's type — `inkwell::ExecutionEngine`
gives way to a raw `LLVMExecutionEngineRef`.  Everywhere we
called `engine.get_function_address(...)` becomes
`LLVMGetFunctionAddress(engine, c_name)`.

Budget half a day to convert and re-pass all existing JIT
tests.  Worth checking whether the inkwell maintainers have
shipped MCJMM-aware bindings in a newer version before
committing to llvm-sys; if they have, we keep the type-safe
wrapper.

**Step 5.  `catch_unwind` boundary + trap-carrying payload.**

The host-side glue.  Two pieces:

  - In the test harness (`tests/newcp-tests/src/lib.rs::run_function`
    and `run_void_function`), wrap the `unsafe { f() }` call
    in `std::panic::catch_unwind(AssertUnwindSafe(|| ...))`.
    The harness can then report failures inline instead of
    crashing the test process on the first ASSERT.
  - Replace `__newcp_trap`'s `abort()` (or `process::exit`)
    with `std::panic::panic_any(NewCpTrapPayload { ... })`
    where the payload carries the trap information we want to
    surface — see § "Trap payload shape" below.  The runtime's
    panic hook silences the default stderr line for our
    sentinel payload so a clean ASSERT fail in a probe doesn't
    spam unrelated noise.

### Trap payload shape

We agreed (verbal, 2026-05-12) that the panic payload IS the
trap-data carrier — no parallel side channel via TLS or
errno-style globals.  Shape:

```rust
pub struct NewCpTrapPayload {
    /// Trap class — CP `ASSERT`, array bounds, NIL deref,
    /// integer-divide-by-zero, type-guard mismatch, explicit
    /// HALT(n), etc.  Encoded so the test harness can
    /// pattern-match on class without parsing strings.
    pub kind: TrapKind,
    /// `HALT(n)` carries `n` here; other kinds set 0.
    pub code: i32,
    /// Source coordinates of the trap site, when the JIT has
    /// emitted them.  `None` for traps raised from Rust
    /// helpers that don't have an obvious CP source location
    /// (e.g. an internal invariant).
    pub site: Option<TrapSite>,
}

pub struct TrapSite {
    pub module:    &'static str,    // "Mod/Tests/Matrix/M_Foo.cp"
    pub procedure: &'static str,    // "Foo.Run"
    pub line:      u32,
    pub column:    u32,
}

pub enum TrapKind {
    Assert,         // ASSERT(cond, n)
    ArrayBounds,    // a[i] with i out of range
    NilDeref,       // p^ with p = NIL
    DivByZero,
    TypeGuard,      // x(Foo) where x is not a Foo
    Halt,           // HALT(n)
    Sema,           // internal sema invariant violation
    Internal,       // catchall — should never fire in v1
}
```

Three things this gets us that the current abort path can't:

  - `cargo test` reports `tests::matrix_foo_runs ... FAILED:
    ASSERT at Mod/Tests/Foo.cp:42 in Foo.Run` instead of
    *the whole test process aborting*.  This is the change
    that lets us run the full matrix without single-test
    failures clobbering everything after them.
  - The test harness can write probes that *expect* a trap —
    `assert_traps!(run_function("…", "Run"), TrapKind::Assert,
    line: 42)`.  Today we just `#[should_panic]` and lose
    classification.
  - When CP gets an exception-handling construct (proposed
    elsewhere, not in standard CP), `TRY/EXCEPT` becomes a
    `catch_unwind` that downcasts the payload to
    `NewCpTrapPayload` and dispatches on `kind`.  The runtime
    is already shaped for it.

Emit-side: the IR's `Instr::Trap { kind: TrapKind }` already
exists in `newcp-ir`.  The LLVM emitter calls into
`__newcp_trap_<kind>` helpers (renamed from `__newcp_trap`).
Each helper takes the call-site coordinates as i32/string
constants embedded in the IR — we already pass module/line in
debug-info metadata, plumbing them through to the trap shim
is the same plumbing.

### Verification

The test that drives this change is a permanent matrix entry:

```
M_Unwind_DeepRecursion — recursive CP procedure calls a
helper that explicitly traps at recursion depth 100; the
host's catch_unwind must observe a `TrapKind::Halt` payload
with `code = 42` (or whichever sentinel).  Today this would
crash the test process.
```

Plus a smaller probe for each `TrapKind` variant — these
guard against silent regressions when somebody changes the
codegen for a trap path.

### Effort

  - Steps 1, 2, 5: half a day each.
  - Step 3 (memory manager): one focused day; mechanical port
    of code that already exists in NCL.
  - Step 4 (drop to llvm-sys): half a day plus any test
    fallout from the engine-handle type change.

Total: 2–3 focused days.  Single biggest win available right
now in terms of correctness-per-line.

## Part 2 — defer: per-thread TLABs

NCL's `docs/GC.md` describes a TLAB design: each Lisp OS
thread carries a `MutatorState { coord, handle, tlab,
young_base, young_starts }` and bump-allocates into a
512 KB thread-local slab without acquiring any lock.  Slow
path takes the heap mutex to refill.  Stop-the-world is
cooperative (the existing shape we already have); the GC is
generational copying with a single young semispace.

We have:

```
$ rg 'MUTATORS|SAFEPOINT_REQUESTED' NewCP/src/newcp-runtime/src/gc.rs
…cooperative-park flag, condvar, registry of Mutator handles…

$ rg 'fn alloc_under_lock|heap_lock\(' NewCP/src/newcp-runtime/src/gc.rs
…every allocation acquires the global heap mutex…
```

So our threading **shell** is already shaped like NCL's
(cooperative STW, per-thread Mutator handles), but every
single `__newcp_new_rec` call serialises on the global heap
mutex.  The TLAB design would replace that serialisation
point with a lock-free bump-and-compare.

### Why NCL needs it

Common Lisp is inherently allocation-heavy.  Cons cells,
closures, let-bindings, intermediate floats — every Lisp
expression of any complexity makes heap garbage.  Their
profiling has `alloc_cons` near the top of the hot list; in a
20-thread program, a global heap lock around `alloc_cons` is
the cap on throughput.

### Why our pressure is different

CP is a record-and-array language, not a cons-cell language.
The hot path of typical compiled CP is field GEPs and method
dispatch — neither allocates.  `__newcp_new_rec` runs *much*
less often than NCL's `alloc_cons`.  Our matrix probes
typically allocate single-digit objects per run.  The
global heap mutex hasn't shown measurable contention under
any workload we've profiled.

We do not yet have a multi-threaded CP workload at all.  The
host-side test runner spawns parallel tests but each test runs
its own JIT'd procedure on a single OS thread.  The CP
language has no `THREAD` primitive of its own; threads enter
the picture only through host code (the iGui mailbox, etc.)
which has its own concurrency story already.

### Why it's also structurally harder for us

NCL's TLAB design *presupposes* a copying generational heap.
The young heap is a bump-allocated semispace; unused TLAB
tails become invisible runs in the young start-bit bitmap; the
next minor GC compacts the gaps away for free.  None of this
is true for our heap, which is **mark-and-sweep with
free-list-allocated clusters**.  A TLAB on top of that has
to choose between two bad options:

  1. Pre-allocate a fixed-size slab from a cluster's free list
     and bump within it.  Unused tails cannot be reclaimed for
     free — they either sit unused until the next sweep (which
     wastes memory proportional to N_threads × TLAB_size /
     sweep_period), or get returned to the free list at TLAB
     retirement (which takes the heap mutex on every retire,
     defeating the lock-free property the TLAB exists to
     provide).
  2. Reshape the heap to be generational + copying.  This is a
     larger change than the whole unwinding plan above — it
     touches root scanning, the write barrier (we'd need one;
     currently we don't), finalization, the `Cluster` type
     itself, the heap-introspection commands, every test that
     exercises the GC.  Worth doing if we have an independent
     reason to go generational; not worth doing just to make
     TLABs work.

There's a secondary cost: variable allocation sizes.  TLABs
shine when most allocations are small fixed-size objects.  CP
records vary from 16 bytes to multi-kilobyte; the TLAB-bump
fast path becomes "TLAB-bump + initialise header + register
typedesc + zero payload", and the bump itself is a small
fraction of that cost.  The relative win shrinks compared to
the cons-cell case.

### What we'd do instead if contention becomes real

In rough order of escalation:

  - **Profile first.**  Add `HEAP_COUNTERS::alloc_lock_wait_nanos`
    measuring time spent in `heap_lock().lock()`.  Threshold:
    if it exceeds 5% of total mutator time under any
    realistic workload, the next step kicks in.
  - **Sharded free lists.**  Partition each cluster's free
    list into N shards (one per CPU).  Allocator picks the
    shard by `thread::current().id().as_u64() % N`.  Same
    heap, same collector, but no contention between threads
    on disjoint shards.  Roughly one week of work; the
    collector has to walk all shards on sweep but each shard
    is independent on the mutator side.
  - **Only then consider TLABs**, *and* only as part of a
    generational rework.  Two large changes lockstep'd.

### One sentence for the file

NCL's TLAB design is right for NCL.  The same design is
wrong for us until our heap shape changes, and our heap
shape has no other reason to change.

## Summary

| Change | Cost | Payoff | Decision |
|---|---|---|---|
| `uwtable=2` on JIT'd fns | 1h | unwind tables emitted | **adopt** |
| `extern "C-unwind"` audit | 4h | no silent fastfail | **adopt** |
| Custom MCJIT memory mgr | 1d | SEH tables registered | **adopt** |
| Drop to llvm-sys for MCJMM | 4h | MCJMM slot reachable | **adopt** |
| `catch_unwind` + `NewCpTrapPayload` | 4h | matrix runs to completion | **adopt** |
| Per-thread TLABs | 2–3wk + heap rework | lock-free alloc fast-path | **defer** |

The unwinding work lands as one PR / one milestone, in the
order above.  Treat the five steps as inseparable — three of
five is silently broken (this is exactly the trap NCL fell
into; their writeup walks through it).  Pick a stretch where
we can land all five in sequence, not piecemeal.

TLABs come back to the table only when (a) profiling shows
heap-lock contention is real, or (b) we go generational for
some other reason.  Neither is true today.

## References

  - `E:\CL\NewCormanLisp\docs\the_seg_windup.md` — the
    bug-hunt narrative.  Required reading before touching the
    memory manager.
  - `E:\CL\NewCormanLisp\src\ncl-llvm\src\jit_mm.rs` — the
    custom memory manager (489 lines, Apache-2.0 or whatever
    NCL ships).  Port target.
  - `E:\CL\NewCormanLisp\src\ncl-llvm\src\lib.rs` — the engine
    construction via llvm-sys, including the runtime-helper
    `add_global_mapping` binding pattern.
  - `E:\CL\NewCormanLisp\docs\GC.md` — generational copying GC
    design with TLABs.  Surveyed for the "defer TLABs"
    decision above.
  - `E:\CL\NewCormanLisp\src\ncl-runtime\src\threads.rs` —
    `exit-thread` / `terminate-thread` via panic_any +
    quiet-panic-hook; reference for our trap-payload design.
  - `E:\NewCP\NewCP\src\newcp-runtime\src\gc.rs` — our current
    GC (mark-and-sweep, clusters, cooperative STW already in
    place).
  - `E:\NewCP\NewCP\src\newcp-llvm\src\jit.rs` — our current
    JIT engine construction via inkwell.  Step 4 above
    rewrites this.
