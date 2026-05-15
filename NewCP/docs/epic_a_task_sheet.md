# Epic A Task Sheet — Stores Backbone Recovery

Implementation-ready task sheet for Epic A from [docs/framework_recovery_backlog.md](docs/framework_recovery_backlog.md).

Date: 2026-05-15
Scope: Stores, Kernel reflection/diagnostics, HostStores merge seams, and typed persistence recovery.

## Objective

Turn Stores into the single honest persistence backbone for NewCP's BlackBox-style framework recovery.

This means:

1. complete the remaining Kernel reflection and loader-diagnostic primitives Stores depends on
2. remove the split between HostStores and Stores as separate typed-object paths
3. finish typed reader/writer/store behavior against real `.odc` data
4. make cloning and externalization trustworthy
5. support Alien fallback so partial subsystem recovery does not corrupt documents

## Status Snapshot

Landed:

1. A1.1 through A1.4 are implemented and covered with focused runtime/CP probe tests.
2. A2.1 and A2.2 are complete: the merge direction is documented in [docs/stores_module_design.md](docs/stores_module_design.md).
3. `Stores` now owns the typed reader/materialization seam (`NewReader`, `SplitQualifiedName`, `NewStoreByName`, `NewLikeOf`, `InternalizeFrom`, `NewStore`).
4. `TextModels.StdModelDesc` and `TextViews.StdViewDesc` have been moved onto the `Stores.StoreDesc` path, and their probe modules now target `Stores.NewStore`.

Still open in the current slice:

1. thin `HostStores` down to forwarding wrappers or remove it once all remaining call sites are migrated
2. rerun the migrated text-model/text-view fixture probes cleanly after the session terminal tooling stops dropping verdict lines
3. move on to the A3 reader/writer audit once the hierarchy merge is stable enough to stop shifting underneath it

## Work Order

Do the tasks in this order unless a concrete code dependency forces a local swap.

1. A1.1 `Kernel.GetLoaderResult`
2. A1.2 `Kernel.GetModName`
3. A1.3 `Kernel.ModOf`
4. A1.4 reflection failure tests
5. A2 merge note and hierarchy inventory
6. A2 path merge work
7. A3 reader/writer audit and fixes
8. A4 clone/externalize hardening
9. A5 Alien A2
10. A5 Alien A3

## A1 — Kernel Reflection And Loader Diagnostics

### A1.1 Implement `Kernel.GetLoaderResult`

Intent:

- provide a stable runtime-side last-loader-result slot so CP code can distinguish lookup/load failures cleanly

Primary files:

- [Mod/Kernel.cp](Mod/Kernel.cp)
- [Mod/KernelSys.cp](Mod/KernelSys.cp)
- [src/newcp-runtime/src/kernel_sys.rs](src/newcp-runtime/src/kernel_sys.rs)
- [src/newcp-loader/src/lib.rs](src/newcp-loader/src/lib.rs)
- [src/newcp-runtime/src/lib.rs](src/newcp-runtime/src/lib.rs)

Likely entry points:

- `Kernel.GetLoaderResult` declaration already present in [Mod/Kernel.cp](Mod/Kernel.cp)
- `KernelSys.LastLoaderResult` declaration already present in [Mod/KernelSys.cp](Mod/KernelSys.cp)
- deferred note in [src/newcp-runtime/src/kernel_sys.rs](src/newcp-runtime/src/kernel_sys.rs)
- loader failure phases in [src/newcp-loader/src/lib.rs](src/newcp-loader/src/lib.rs)

Implementation notes:

- add a process-global last-loader-result structure in `kernel_sys.rs`
- expose setters for success and failure paths
- map loader failures to the `loader*` constants already declared in `KernelSys.cp`
- export both `LastLoaderResult` and `GetLoaderResult` through the native artifact list
- clear or reset the slot on successful loads so `loaderOk` is observable

Focused validation:

- add runtime/loader tests that record a failure and then read back the code and messages
- add a CP-level probe once the plumbing is exposed to loaded modules

Suggested test targets:

- [tests/newcp-tests/src/lib.rs](tests/newcp-tests/src/lib.rs)
- new probe module beside [Mod/Tests/KernelProbe.cp](Mod/Tests/KernelProbe.cp)

BRK use:

- use `BRK` if the CP-visible result disagrees with the runtime slot after a loader failure

### A1.2 Implement `Kernel.GetModName`

Intent:

- reverse-map a module handle back to the registered module name

Primary files:

- [Mod/Kernel.cp](Mod/Kernel.cp)
- [Mod/KernelSys.cp](Mod/KernelSys.cp)
- [src/newcp-runtime/src/kernel_sys.rs](src/newcp-runtime/src/kernel_sys.rs)

Likely entry points:

- `module_name_from_handle` already exists in [src/newcp-runtime/src/kernel_sys.rs](src/newcp-runtime/src/kernel_sys.rs)
- module registry snapshot helpers already exist in the same file

Implementation notes:

- add a C-ABI export that writes the module name into an open-array `OUT ARRAY OF CHAR`
- return empty string for NIL or out-of-range handles
- register the export in both `KernelSys` and `Kernel`

Focused validation:

- CP probe: `ThisMod("Console") -> GetModName(handle) == "Console"`
- out-of-range / NIL handle should yield empty string

Suggested test targets:

- [Mod/Tests/KernelProbe.cp](Mod/Tests/KernelProbe.cp)
- [tests/newcp-tests/src/lib.rs](tests/newcp-tests/src/lib.rs)

### A1.3 Implement `Kernel.ModOf`

Intent:

- recover type-to-module ownership lookup using the current qualified-name fallback

Primary files:

- [Mod/Kernel.cp](Mod/Kernel.cp)
- [Mod/KernelSys.cp](Mod/KernelSys.cp)
- [src/newcp-runtime/src/kernel_sys.rs](src/newcp-runtime/src/kernel_sys.rs)

Likely entry points:

- `type_desc_qualified_name_string` in [src/newcp-runtime/src/kernel_sys.rs](src/newcp-runtime/src/kernel_sys.rs)
- `register_known_module` and module handle reverse mapping in the same file

Implementation notes:

- parse the module segment from the qualified type name
- resolve it through the existing module registry rather than assuming direct module pointers exist
- make `TypeMod` and `ModOf` share one implementation

Focused validation:

- CP probe: `NEW(w); ModOf(TypeOf(w))` must resolve the declaring module
- pair with `GetModName` to confirm exact owner name

Suggested test targets:

- [Mod/Tests/KernelProbe.cp](Mod/Tests/KernelProbe.cp)
- [tests/newcp-tests/src/lib.rs](tests/newcp-tests/src/lib.rs)

### A1.4 Add reflection failure tests

Intent:

- lock down the behavior Stores will consume before Stores depends on it more heavily

Primary files:

- [Mod/Tests/KernelProbe.cp](Mod/Tests/KernelProbe.cp)
- [tests/newcp-tests/src/lib.rs](tests/newcp-tests/src/lib.rs)

Test cases:

1. unknown module returns NIL and records the right loader result behavior
2. known module + unknown type returns NIL without corrupting the last loader result contract
3. `GetModName(NIL)` yields empty string
4. `ModOf(NIL)` yields NIL
5. `GetLoaderResult` reports success after a clean load path

## A2 — Merge HostStores Back Into Stores

### A2.1 Write the merge note first

Intent:

- avoid ad hoc edits that deepen the split

Primary files:

- [docs/stores_module_design.md](docs/stores_module_design.md)
- [Mod/Stores.cp](Mod/Stores.cp)
- [Mod/HostStores.cp](Mod/HostStores.cp)
- [Mod/TextViews.cp](Mod/TextViews.cp)

Required output:

- a short note answering:
  - which APIs stay on `Stores`
  - which typed-object responsibilities currently live in `HostStores`
  - which symbols move, which symbols disappear, and which call sites change first

Completion criteria:

- the merge plan is explicit before code edits begin

### A2.2 Inventory split call sites

Files to inspect first:

- [Mod/HostStores.cp](Mod/HostStores.cp)
- [Mod/Stores.cp](Mod/Stores.cp)
- [Mod/TextViews.cp](Mod/TextViews.cp)
- [Mod/Models.cp](Mod/Models.cp)
- [Mod/Views.cp](Mod/Views.cp)

Questions to answer:

1. where is typed object allocation currently done
2. where does typed `Internalize` dispatch happen
3. which modules depend on HostStores-specific types
4. which modules assume the split hierarchy and will break when merged

### A2.3 Merge sequence

Recommended local order:

1. unify typed store allocation/factory entry points
2. move typed reader/writer usage to Stores-owned types
3. update deserializing modules like TextViews to consume the merged path
4. delete or thin HostStores until it is either gone or purely transitional

Focused validation:

- synthetic typed `.odc` load tests
- one real fixture-based load test
- BRK on a freshly materialized store to confirm the runtime type is in the real hierarchy

## A3 — Reader/Writer Audit

Primary files:

- [Mod/Stores.cp](Mod/Stores.cp)
- [Mod/HostStores.cp](Mod/HostStores.cp)
- [src/newcp-runtime/src/stores_sys.rs](src/newcp-runtime/src/stores_sys.rs)
- [docs/stores_module_design.md](docs/stores_module_design.md)
- [Mod/Tests/StoresProbe.cp](Mod/Tests/StoresProbe.cp)
- [tests/newcp-tests/src/lib.rs](tests/newcp-tests/src/lib.rs)

Audit checklist:

1. eof/cancel behavior
2. cursor position and clamping
3. inline child store read/skip behavior
4. version stamp read/write
5. partial reads and failure propagation
6. reader-from-writer round-trip correctness

Suggested tests:

- nested inline child stores
- read/skip/read sequences
- invalid-handle behavior remains loud but non-corrupting

BRK use:

- `BRK(ptr)` on the owning store when an inline child lands at the wrong type
- plain `BRK` when cursor movement diverges from expected body bounds

## A4 — Clone And Externalize Hardening

Primary files:

- [Mod/Stores.cp](Mod/Stores.cp)
- [Mod/Models.cp](Mod/Models.cp)
- overriding framework modules that implement `Externalize` or `Internalize`
- [src/newcp-runtime/src/stores_sys.rs](src/newcp-runtime/src/stores_sys.rs)

Work items:

1. list every current `Externalize` override and mark real vs placeholder behavior
2. add trap-cleaner protection where clone/load can leave inconsistent marks or partially-built objects
3. confirm clone paths preserve type identity but not object identity

Suggested tests:

- `Stores.CopyOf` for a nested model/view graph
- externalize + open-reader-from-writer + internalize round-trip on typed content

## A5 — Alien A2 Then A3

Primary files:

- [docs/stores_module_design.md](docs/stores_module_design.md)
- [Mod/Stores.cp](Mod/Stores.cp)
- [src/newcp-runtime/src/stores_sys.rs](src/newcp-runtime/src/stores_sys.rs)
- [src/newcp-odc/src](src/newcp-odc/src)
- fixture-based tests in [tests/newcp-tests/src/lib.rs](tests/newcp-tests/src/lib.rs)

A2 target:

- unknown type becomes a CP Alien store with preserved bytes
- re-externalize writes those bytes unchanged

A3 target:

- version mismatch can fall back mid-Internalize through `TurnIntoAlien`
- cause reporting is preserved for diagnostics

Suggested validation:

- synthetic unknown-type `.odc`
- real BlackBox corpus round-trip where unsupported types remain intact

BRK use:

- BRK at alien creation sites with module, type, version, and byte-span context

## First Code Slice To Start Now

Start with A1.1 `Kernel.GetLoaderResult`.

Reason:

- smallest self-contained kernel item
- directly required by the Stores recovery plan
- establishes the pattern for the remaining reflection/diagnostic exports

Local implementation path:

1. add the runtime-side last-loader-result storage in [src/newcp-runtime/src/kernel_sys.rs](src/newcp-runtime/src/kernel_sys.rs)
2. export the C-ABI getter there
3. expose it through the native module artifact list
4. wire setter calls from loader success/failure paths
5. add narrow tests before moving on to `GetModName`

## Done For Epic A

Epic A is done only when:

1. Stores can diagnose module/type lookup failures through real kernel APIs
2. HostStores no longer represents a parallel authoritative typed-store path
3. typed load/store/clone behavior is validated against real fixtures
4. unsupported or version-skewed types survive round-trip through Alien fallback
5. Stores is safe to use as the persistence backbone for broader framework recovery
