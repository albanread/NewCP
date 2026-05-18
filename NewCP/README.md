# NewCP

NewCP is a new Component Pascal compiler and runtime that recreates the BlackBox programming model on a modern, memory-resident, JIT-first architecture.

The engineering rule is to keep the bootstrap slice minimal: implement only the resident Rust pieces and temporary facade modules required to bring up the compiler, compile the first CP modules, and then replace those facades with CP-built equivalents.

NewCP is not a batch compiler that emits native object files as its primary product. It is a memory-resident system that can:

- parse and type-check Component Pascal modules
- lower them through a modern compiler pipeline
- generate LLVM IR
- emit reviewable textual artifacts from every compiler phase
- JIT modules into memory
- load additional modules dynamically on demand
- preserve enough of the BlackBox runtime model to support documents, commands, views, stores, reflection, and dynamic services

## Core decisions

NewCP is **JIT-first** and **64-bit first**.

The original BlackBox loader worked with object files, symbol files, type descriptors, fixups, and dynamic module initialization. NewCP keeps the dynamic module model, but the default execution model is different:

- the runtime is memory-resident
- the initial implementation targets a 64-bit address space
- the first supported execution target is `x86_64-pc-windows-msvc`
- module code, data, descriptors, and metadata stay resident by default
- modules are JIT-compiled when first required
- additional modules may be JIT-compiled later into the same process
- unloading is optional and secondary; correctness of dynamic loading matters more than memory reclamation

Memory is no longer the scarce resource that shaped the original loader. BlackBox is small enough now that the whole environment can live in memory.

## Current status (2026-05-18)

NewCP is past the bootstrap-shape phase. The compiler pipeline, JIT, runtime, and a usable framework slice are all live in one process.

### Compiler pipeline

All ten phases run end-to-end, each with its own textual dump:

- lexer / parser / sema / module-graph / CFG / typed IR / LLVM IR / native asm
- ORC-style materialization through MCJIT (single execution engine per module today)
- runtime registration via a structured native-module artifact, so Rust-hosted modules and CP modules look the same from the loader's point of view

The driver exposes the pipeline directly: `dump-tokens`, `dump-ast`, `dump-sema`, `dump-module-graph`, `dump-cfg`, `dump-ir`, `dump-llvm`, `dump-asm`, `dump-heap`, plus `describe-interface` and `load-module`.

### Language

Working on the JIT today, with regression tests in [`Mod/Tests`](Mod/Tests) and [`tests/newcp-tests`](tests/newcp-tests):

- scalars, arrays (including multidimensional and open arrays), records, sets, strings
- procedures, nested procedures, value/`VAR`/`IN` parameter modes (with sema rejecting value-mode record/array params and writes through `IN`)
- control flow: `IF`, `CASE`, `WHILE`, `REPEAT`, `LOOP`, `FOR`, `WITH`, `EXIT`, `RETURN`
- pointers and `NEW`, including pointer-aliased records
- type tests (`IS` / `WITH`) via `Instr::TypeCheck` + `Terminator::TypeTest`
- record extension and inherited field access via typed GEP across the extension chain
- **virtual method dispatch** end-to-end on the JIT: vtables emitted as mutable globals, patched after MCJIT materialization via `LLVMGetPointerToGlobal` (working around MCJIT's lack of relocation for function-pointer constants in vtable initializers — see [`docs/oop_runtime_status.md`](docs/oop_runtime_status.md))
- abstract-base virtual dispatch
- `SYSTEM` intrinsics: `AddrOf`, `BitCast`, `LoadRaw`, `StoreRaw`, `Lsh`, `Rot`, `MemCopy`, `SysNew`
- module-body init pipeline; cross-module record-by-reference; cross-module type-alias unwrap

87+ pure-compute JIT integration tests cover spec compliance for arithmetic boundaries, arrays, sets, and multidimensional access.

### Runtime

- garbage collector with cluster/block layout, tagged allocations via `__newcp_new_rec`, mark/sweep, finalizers, module roots
- `heap_introspect` snapshot facility (`dump-heap`) — counters always-on, structured `HeapSnapshot` on demand, full type-by-instance walks for diagnostics
- panic-safe execution scopes; cross-module vtable patching; `LoaderSession` with active/retired generations and a drop-predicate hook for hot reload
- ODC envelope reader (`newcp-odc`) round-trips a 675-file `.odc` corpus from BlackBox 1.7
- read-only document walker (Stores S1) backed by `newcp-odc` via a `stores_sys.rs` shim

### Framework slice

CP modules currently live in [`Mod/`](Mod) and run on the JIT:

- **`Kernel`** — the BlackBox-equivalent runtime surface. Bound to Rust today: `Time`, `Beep`, `TypeOf`, `BaseOf`, `SizeOf`, `LevelOf`, `NewObj`, `Loop`, `Quit`, `Event`, `EventHandler`, `PushTrapCleaner`, `PopTrapCleaner`, `GetTypeName`, `GetQualifiedTypeName`, `ThisMod`, `ThisType`, `GetModName`, `ModOf`, `GetLoaderResult`. Reflection lookup failures now also update the loader-result slot Stores uses for module/type diagnostics.
- **`Files`** + **`HostFiles`** — abstract interface module + concrete subclass; cross-module OOP is exercised here.
- **`Dates`** + **`HostDates`** — abstract surface plus host-backed implementation; `DatesArith` / `DatesClock` integration tests pass on the JIT.
- **`Fonts`** + **`HostFonts`** — full `HostFonts` impl with `FontProbe` smoke test. `HostFonts` ships with renamed local types as a workaround for the documented sema name-collision case.
- **`Math`**, **`SMath`**, **`Strings`** — full `RealToStringForm`, `StringIntFmt`, etc.
- **`Console`**, **`Log`**, **`Integers`**
- **`Stores`** + **`StoresSys`** — the Stage S1 envelope walker is live, and the first S2 merge seam is now real: `Stores` owns typed reader lifecycle and typed materialization helpers (`NewReader`, `SplitQualifiedName`, `NewStoreByName`, `NewLikeOf`, `InternalizeFrom`, `NewStore`). `TextModels.StdModelDesc` and `TextViews.StdViewDesc` now hang off the `Stores.StoreDesc` path; `HostStores` is now transitional and slated for thinning.
- **`iGui`** — the integrated GUI; see below.

The `HostXxxSys` layering pattern is honored: abstract `Xxx.cp` stays import-free, `HostXxx.cp` imports `HostXxxSys.cp`, and only the `Sys` module imports `iGui`.

### iGui — integrated MDI windowing

iGui replaces the old external `multiwingui` / `wingui.dll` host with an MDI windowing layer implemented directly inside `newcp-runtime`. It is `x86_64-pc-windows-msvc` only, backed by Direct2D + DirectWrite, and inverts the previous startup ownership so that the GUI is the main thread and the language runtime is launched after the GUI is up.

The phased demos under [`Mod/demo/igui/`](Mod/demo/igui) match the iGui design phases:

| Phase | Demo | What it shows |
|---|---|---|
| 2 | `Phase2EventDemo.cp` | event mailbox round-trip on the language thread |
| 3a | `Phase3aMdiDemo.cp` | MDI frame + child windows |
| 3b/3c | `Phase3bGeometryDemo.cp` / `Phase3cGeometryDemo.cp` | rect/path/ellipse/arc primitives + DPI + cursor |
| 4 | `Phase4TextDemo.cp` | DirectWrite text + synchronous query channel + layout cache |
| 5 | `Phase5CompositionDemo.cp` | composition + overlays + paths + system colors |
| 6 | `Phase6MenuDemo.cp` | menu bar + standard MDI verbs |
| 7 | `Phase7TickDemo.cp` | animation tick |

A Rust-thread `redit` UI-thread fail-safe editor for CP source ships alongside iGui as a recovery tool, and the `Log` view is a durable Rust ring buffer surfaced through a Tools window.

### Tests

- 188 / 188 integration tests
- 50 / 50 runtime tests
- 29 / 29 loader tests
- 474 / 490 newcp-tests pass (16 under active work — TextViews pointer-cast IR bug + Tour.odc decode)
- all other crates green

Recent compiler fixes (May 2026):
- `ORD` builtin now accepts `BYTE` (CP §10.2 BlackBox extension), unblocking TextRulers and its dependents
- `SHORT` type chain correctly narrows `ShortInt → Byte` (`IrType::I16 → U8`)
- `INCL` / `EXCL` sema now resolves root variable names for qualified-field designators (`a.opts`)
- BlackBox wire format: `ReadInt` correctly reads 2-byte i16 (not 4-byte i32)

## What's next

See [`docs/next_milestones.md`](docs/next_milestones.md) for the rolling plan. The current sequence:

1. **Finish the Stores/HostStores merge** — keep migrating remaining call sites, revalidate the migrated typed-load paths on real fixtures, and thin `HostStores` down to forwarding wrappers or remove it.
2. **Stores reader/writer audit** — finish the remaining eof/cancel / inline-child / version-stamp semantics against real `.odc` fixtures.
3. **Alien fallback (A2 then A3)** — preserve unknown or version-skewed stores without corrupting documents.
4. **Heap-side dangling-TypeDesc probe** while more CP records start surviving on the heap across reload boundaries.
5. **`HostMenus` from CP on top of `Kernel.Loop`** — a separate work-stream once `iGui.OpenChild` and the MDI plumbing it needs are in place.

Documented background:

- [`docs/blackbox_architecture.md`](docs/blackbox_architecture.md) — what the original BlackBox shape looked like
- [`docs/blackbox-jit-compatibility.md`](docs/blackbox-jit-compatibility.md) — what we keep, what we change
- [`docs/compiler-architecture.md`](docs/compiler-architecture.md) — pipeline + binding policy (direct CP-to-CP, native Rust-hosted, late-bound)
- [`docs/roadmap.md`](docs/roadmap.md) — phases 0–7
- [`docs/oop_runtime_status.md`](docs/oop_runtime_status.md) — virtual dispatch on MCJIT
- [`docs/igui_design.md`](docs/igui_design.md) — GUI process / thread model
- [`docs/stores_module_design.md`](docs/stores_module_design.md) — full Stores port plan (S1–S6)
- [`docs/heap_introspection.md`](docs/heap_introspection.md) — `dump-heap` design
- [`docs/garbage-collection.md`](docs/garbage-collection.md) — heap layout, mark/sweep, multi-thread roadmap
- [`docs/odc_doc.md`](docs/odc_doc.md) — in-memory document model
- [`docs/system-module.md`](docs/system-module.md) — `SYSTEM` intrinsic surface
- [`docs/usersguide_to_cp_oop.md`](docs/usersguide_to_cp_oop.md) — OOP patterns the framework relies on

Open issues worth knowing about:

- [`docs/bug_report_short.md`](docs/bug_report_short.md) — `SHORT(LONGINT)` truncation; the full chain (`I64→I32→I16→U8`) is now fixed in the IR lowering
- [`docs/bug_report_sema_name_collision.md`](docs/bug_report_sema_name_collision.md) — sema infinite recursion on certain local-vs-import name collisions; `HostFonts` works around it with renamed locals
- [`docs/deferred_fixes.md`](docs/deferred_fixes.md) — index of known shipped workarounds

## Phase visibility

NewCP must not behave like an opaque compiler pipeline.

Each major phase is:

- discrete in implementation
- invokable independently where practical
- observable through stable textual dumps
- suitable for regression tests and human review

The shipped review artifacts: token stream, AST, bound symbol/type, module graph, CFG, typed IR, LLVM IR, final assembly, heap snapshot.

This is a design requirement, not an optional debugging feature.

## Source layout

```text
NewCP/
  docs/
  Mod/                        Component Pascal modules + Tests/ + demo/igui/
  src/
    newcp-lexer/
    newcp-parser/
    newcp-sema/
    newcp-ir/
    newcp-llvm/
    newcp-runtime/            kernel, GC, heap_introspect, iGui, host_*_sys, stores_sys
    newcp-loader/
    newcp-odc/                .odc envelope reader
    newcp-driver/
  tests/
    newcp-tests/
    newcp-compat-tests/
```

## Driver surface

- `newcp-driver describe-interface InitShell` — show the in-memory interface descriptor for a live module
- `newcp-driver load-module HostMenus` — look for `Mod/HostMenus.cp` and load it
- `newcp-driver load-module HostMenus HostMenus.OpenApp` — load and invoke a command immediately
- `newcp-driver dump-tokens|dump-ast|dump-sema|dump-module-graph|dump-cfg|dump-ir|dump-llvm|dump-asm|dump-heap`

## Application shape

Same broad split as BlackBox:

- a resident `Kernel` equivalent for module tables, type descriptors, runtime services, and loader/JIT coordination
- a separate `Init` equivalent that boots the application, loads the first modules, registers core services, and enters the live environment

The compiler itself lives alongside that resident core as a Rust component and compiles Component Pascal modules into the running process.

## Replacement rule

- start with Rust only where the compiler cannot yet provide the module
- once a CP module can be compiled and loaded reliably, prefer the CP module over the Rust facade
- do not add new Rust-hosted framework modules unless the bootstrap path is blocked without them

## Host language

Rust hosts the resident runtime and the compiler. Reasons:

- native deployment without a managed VM
- suitable for low-level runtime and kernel work
- suitable for compiler front-end and IR work
- one toolchain for the whole system
- practical interop with LLVM through Inkwell

Component Pascal modules are compiled by that Rust-hosted system and materialized into memory on demand. The pieces written in Rust from the start are: the resident `Kernel` equivalent, the startup/bootstrap `Init` equivalent, the compiler pipeline and driver, the JIT loader and runtime symbol/link infrastructure, the GC, the iGui MDI host, and the small `HostXxxSys` shims.
