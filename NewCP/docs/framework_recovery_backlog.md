# NewCP Framework Recovery Backlog

Working backlog for recovering the BlackBox-style resident MVC/document environment on top of the already-live NewCP compiler, JIT, loader, and runtime.

Date: 2026-05-15
Status: active execution baseline

## Purpose

This document turns the framework-recovery assessment into a concrete backlog.

Use it for three things:

1. Decide what to work on next without reopening the architecture.
2. Keep temporary bootstrap layers from solidifying into the product.
3. Measure progress against real BlackBox behaviors rather than against isolated compiler progress.

## Current Position

What is already real:

- compiler pipeline, JIT, loader, GC, and runtime residency
- CP module loading and cross-module execution
- ODC envelope parsing and round-trip tooling
- enough framework slice to prove object-model and host integration direction
- Kernel reflection/loader diagnostics Stores needs (`GetModName`, `ModOf`, `GetLoaderResult`, failure-state probes)
- Stores-owned typed reader/materialization seam (`NewReader`, `SplitQualifiedName`, `NewStoreByName`, `NewLikeOf`, `InternalizeFrom`, `NewStore`)

What is still incomplete in a load-bearing way:

- Stores is not yet the single honest persistence backbone for partial-framework documents
- HostStores is still present as a transitional duplicate layer and still needs thinning/removal
- MVC core modules still contain tolerated no-op or NIL-return behavior
- text views are only partially recovered as live runtime views
- host rendering and input are still incomplete for real editor behavior
- parity testing is still stronger at language level than at framework/system level

## Working Rules

These rules are part of the backlog. Violating them is scope drift.

1. Do not add new Rust-hosted framework modules unless bootstrap is genuinely blocked.
2. Every Host or temporary shim must name the CP-side replacement or merge target.
3. Prefer one honest implementation path over two partial ones.
4. If a failure is unclear, use `BRK` to dump state before adding another workaround.
5. Do not treat a method that returns NIL or no-ops as "done" unless BlackBox itself did.
6. Update planning docs only after code and tests prove the new state.

## Recovery Order

The dependency order is:

1. Stores unification and resilience
2. Kernel reflection and loader diagnostics needed by Stores
3. Domain, sequencing, cloning, and trap safety
4. MVC core completion
5. Text subsystem completion
6. Host rendering and input completion
7. Resident shell and higher subsystem recovery
8. BlackBox parity and artifact-driven testing

The rest of this document follows that order.

## Epic A — Stores As The Real Backbone

Goal: make Stores the authoritative persistence and typed-object loading layer, including partial-framework document survival.

### A1. Close reflection gaps Stores needs

- [x] A1.1 Implement `Kernel.ModOf` with the current name-based fallback.
- [x] A1.2 Implement `Kernel.GetModName` for runtime-visible modules.
- [x] A1.3 Implement `Kernel.GetLoaderResult` and plumb the last loader result through the runtime.
- [x] A1.4 Add focused tests for `(module, type)` lookup failure reporting.

Acceptance:

- Stores can distinguish module-not-found from type-not-found without guesswork.
- Reflection used by deserialization no longer depends on comments or TODOs.

Validation:

- integration tests for successful and failing module/type lookup
- BRK dump of module and type registry when a lookup fails unexpectedly

### A2. Remove the split between HostStores and Stores

- [x] A2.1 Inventory every place where `HostStores.StoreDesc` and `Stores.StoreDesc` diverge in behavior or type usage.
- [x] A2.2 Pick the merge direction explicitly and document it in code comments before editing.
- [x] A2.3 Merge typed reader/writer/store materialization into the real Stores hierarchy.
- [ ] A2.4 Remove duplicated factory logic once the real hierarchy owns it.
- [ ] A2.5 Update deserializing views/models to use the merged path only.

Recent A2 progress:

- `Stores` now owns the typed reader/materialization helper path that used to exist only in `HostStores`
- `TextModels.StdModelDesc` and `TextViews.StdViewDesc` now inherit from `Stores.StoreDesc`
- probe modules have started switching from `HostStores.NewStore` to `Stores.NewStore`
- runtime revalidation still needs another pass before `HostStores` can be thinned aggressively

Acceptance:

- there is one authoritative typed store hierarchy
- deserialized objects participate in the same runtime contracts as live framework objects

Validation:

- typed load of a real `.odc` file lands in objects that also satisfy the runtime MVC interfaces
- BRK on a deserialized object shows the expected runtime type and field layout

### A3. Finish typed reader/writer behavior

- [ ] A3.1 Audit `Reader` and `Writer` against the Stores design note and BlackBox behavior.
- [ ] A3.2 Finish missing primitive behaviors and failure-state handling.
- [ ] A3.3 Verify `ReadStore`, `SkipStore`, `ReadVersion`, and `WriteVersion` against real fixtures.
- [ ] A3.4 Add regression tests for nested inline store handling and cursor positioning.

Acceptance:

- reader and writer semantics are stable enough for typed graph load/store work
- cursor movement and eof/cancel semantics are explicit and tested

Validation:

- focused tests around inline child store sequences
- BRK at the failing read site dumping reader position, body bounds, and last type name

### A4. Make `Externalize` and `CopyOf` honest

- [ ] A4.1 Audit every current `Externalize` override in framework modules.
- [ ] A4.2 Replace placeholder `CopyOf` expectations with real round-trip behavior through the typed Stores path.
- [ ] A4.3 Add trap-cleaner integration around clone/deserialize paths where required.
- [ ] A4.4 Verify that deep copy never aliases the original object graph.

Acceptance:

- `CopyOf` is a trustworthy cloning path for framework data, not just a convenience wrapper
- deserialization and copy do not silently skip important children

Validation:

- clone tests for models and views with nested child stores
- heap snapshot before/after clone to confirm distinct live objects

### A5. Ship Alien A2, then A3

- [ ] A5.1 Implement Alien A2: unknown type becomes a preserved CP Alien store.
- [ ] A5.2 Ensure Alien `Externalize` writes preserved bytes back unchanged.
- [ ] A5.3 Add real corpus tests for partial-framework typed load + re-save.
- [ ] A5.4 Implement Alien A3: version-tolerant fallback and `TurnIntoAlien` mid-Internalize.
- [ ] A5.5 Connect alien-cause reporting to diagnostics.

Acceptance:

- documents containing unported or version-skewed types survive a typed load/store cycle
- the framework can grow incrementally without corrupting user documents

Validation:

- corpus round-trip tests including unsupported types
- BRK on alien creation with module, type, version, and body span information

## Epic B — Domain, Sequencing, and Trap Safety

Goal: restore the runtime semantics that make Stores useful in a live environment.

### B1. Finish `Stores.Domain`

- [ ] B1.1 Port or complete Domain ownership, join, and unattached semantics.
- [ ] B1.2 Verify domain relationships on cloned and deserialized stores.
- [ ] B1.3 Add tests for illegal domain transitions and join behavior.

Acceptance:

- stores and models have meaningful domain ownership
- domain membership is not a placeholder for future work

### B2. Restore Sequencers through the real path

- [ ] B2.1 Connect `Models` to real Stores-backed sequencing instead of placeholder store handles.
- [ ] B2.2 Implement or complete domain-level dispatch through the actual sequencer path.
- [ ] B2.3 Add tests for script grouping, undo, redo, and dirty-state transitions.

Acceptance:

- edits flow through the runtime in the BlackBox shape: model -> domain -> sequencer

Validation:

- focused tests for `BeginScript`, `Do`, `EndScript`, bunching, and dirty state
- BRK at sequencer dispatch boundaries showing model/domain/script identity

### B3. Harden trap and reload safety

- [ ] B3.1 Add the heap-side probe that blocks unsafe retirement of live type metadata.
- [ ] B3.2 Exercise reload while typed CP records are live.
- [ ] B3.3 Connect trap-cleaner usage to the new clone/load/edit flows.

Acceptance:

- hot reload refuses to drop images whose type metadata is still live on the heap
- trap-cleaner use is deliberate and tested

Validation:

- loader/runtime tests with live typed objects across reload attempts
- BRK plus heap snapshot when a retirement is refused

## Epic C — Recover The MVC Core

Goal: stop relying on tolerated skeleton behavior in Models/Views/Controllers/Services/Dialog.

### C1. Views

- [ ] C1.1 Audit deferred or intentionally omitted View behaviors against actual BlackBox expectations.
- [ ] C1.2 Implement the next missing load-bearing View procedures in dependency order.
- [ ] C1.3 Remove safe no-op behavior where it currently masks missing work.

Acceptance:

- core View behavior is present because it works, not because callers tolerate emptiness

### C2. Controllers

- [ ] C2.1 Replace NIL focus queries with real focus routing.
- [ ] C2.2 Complete controller/view/model message flow for current framework slices.
- [ ] C2.3 Add tests for focus transfer and routed controller actions.

Acceptance:

- focus and controller routing are real runtime behaviors

Validation:

- controller integration tests using live view graphs
- BRK at focus changes showing active view, model, and controller

### C3. Services

- [ ] C3.1 Finish the host-connected deferred action path.
- [ ] C3.2 Connect `Step` and `Loop` to the actual event loop instead of a partial local walk.
- [ ] C3.3 Add tests for time-ordered deferred execution and immediate actions.

Acceptance:

- deferred actions execute through the real host/event loop path

### C4. Dialog and Properties support

- [ ] C4.1 Expand `Dialog` beyond the current type-only slice to the minimum real framework contract needed by Stores and tools.
- [ ] C4.2 Recover the property/message behavior required by actual views and tools.
- [ ] C4.3 Add diagnostic surfaces needed by alien/version reporting.

Acceptance:

- Dialog is a real subsystem slice, not a color-wrapper placeholder

## Epic D — Recover The Text Stack Honestly

Goal: make the text/document subsystem a real live subsystem rather than a partially loaded wire-format shell.

### D1. Unify wire-loaded and runtime text views

- [ ] D1.1 Remove the split between deserializing `StdView` readers and live runtime text views.
- [ ] D1.2 Ensure loaded text views satisfy the same View/Container contracts as newly created ones.
- [ ] D1.3 Materialize controller, ruler, and attributes child stores instead of skipping them.

Acceptance:

- one coherent text view hierarchy handles both deserialized and live views

### D2. Finish text layout state and behavior

- [ ] D2.1 Implement real visible-range tracking.
- [ ] D2.2 Implement real hit-testing and position mapping.
- [ ] D2.3 Implement scrolling behavior beyond stored origin fields.
- [ ] D2.4 Implement selection and marks behavior where currently deferred.
- [ ] D2.5 Add tests that assert behavior, not just field storage.

Acceptance:

- text pane behavior matches the actual model and visible geometry

Validation:

- integration tests on real text fixtures
- BRK at text-view restore/show-range/hit-test failures dumping origin, visible range, and model length

### D3. Finish text persistence round-trip

- [ ] D3.1 Audit `TextModels`, `TextViews`, `TextRulers`, and attributes persistence.
- [ ] D3.2 Ensure typed load/store preserves real text semantics and not just bytes.
- [ ] D3.3 Compare typed graph walks against YAML projections for known fixtures.

Acceptance:

- text documents round-trip through the typed CP graph without semantic drift

## Epic E — Finish Host Rendering And Input

Goal: complete the host-side behavior needed for a real resident BlackBox-style environment.

### E1. Complete HostPorts

- [ ] E1.1 Implement the remaining geometry methods required by current views.
- [ ] E1.2 Implement scrolling, cursor, and input semantics.
- [ ] E1.3 Add tests or probes for host drawing and input dispatch where automatable.

Acceptance:

- current framework slices no longer stop at partial host drawing primitives

### E2. Connect MVC to iGui end-to-end

- [ ] E2.1 Wire controller input through the actual event mailbox and child surfaces.
- [ ] E2.2 Verify menu and child-window flows needed by the resident shell.
- [ ] E2.3 Keep the surface architecture intact; do not backslide into hidden native-widget logic.

Acceptance:

- a CP view can receive real input, mutate a model, and render through the host path in one loop

Validation:

- manual and scripted smoke probes for event dispatch, repaint, focus, and scroll
- BRK dumps of mailbox events and pane command queues when behavior diverges

## Epic F — Recover The Resident Shell And Higher Subsystems

Goal: move from infrastructure and core framework recovery into the actual BlackBox environment shape.

### F1. Shell and menus

- [ ] F1.1 Finish `HostMenus` on top of the completed event loop and child-window plumbing.
- [ ] F1.2 Recover the minimum app-shell behavior needed for resident tools and commands.
- [ ] F1.3 Add tests or repeatable probes for command invocation from the shell.

### F2. Higher subsystem order

Recover higher subsystems in this order unless a strict dependency says otherwise:

1. `Properties`
2. `Containers`
3. `TextControllers`
4. menu/app shell
5. next resident subsystems and tools

Rule:

- a subsystem is not "ported" until it runs through the real Stores/MVC/host contracts

## Epic G — Parity And Artifact-Driven Testing

Goal: stop measuring success only at the language/compiler level.

### G1. Real artifact lanes

- [ ] G1.1 Add typed-load tests over real `.odc` fixtures.
- [ ] G1.2 Add framework integration tests that exercise model/view/controller flows.
- [ ] G1.3 Add text subsystem behavior tests using real documents.
- [ ] G1.4 Add reload-safety tests with live typed objects.

### G2. Probe policy

- [ ] G2.1 Keep small CP probes for each recovered subsystem slice.
- [ ] G2.2 Require one regression probe for each real bug fixed in framework recovery.
- [ ] G2.3 Prefer artifact-backed probes over abstract surface-only tests when possible.

Acceptance:

- passing tests mean the resident framework works on real artifacts, not only on synthetic language samples

## Immediate Backlog — Next 15 Tasks

These are the recommended next tasks in exact order.

1. Implement `Kernel.GetLoaderResult`.
2. Implement `Kernel.GetModName`.
3. Implement `Kernel.ModOf`.
4. Add reflection failure tests that Stores will rely on.
5. Write the HostStores vs Stores merge note before code edits.
6. Merge factory/type-materialization logic into one Stores path.
7. Re-run typed `.odc` load tests on the merged path.
8. Audit `Reader`/`Writer` behavior against the design note.
9. Add inline-child store regression tests.
10. Add trap-cleaner protection around the current clone path if still missing.
11. Implement Alien A2.
12. Add corpus round-trip tests for unknown typed content.
13. Finish Domain attachment/join semantics.
14. Route Models sequencing through the real Stores path.
15. Add a hot-reload safety test with live typed objects.

If one of these uncovers missing architectural information, stop and dump with `BRK` rather than inserting another placeholder.

## BRK Policy

`BRK` is part of the recovery workflow, not an emergency-only tool.

Use plain `BRK` when you need a process snapshot at:

- failed module/type lookup during deserialization
- unexpected domain or sequencer state
- event-loop or mailbox divergence
- reload-safety refusal or stale metadata suspicion

Use `BRK(ptr)` when you need a targeted typed dump of:

- a just-deserialized store or child store
- a model, view, or controller whose runtime type is suspect
- a text-view or text-model object whose fields do not match behavior
- a sequencer/domain object during edit dispatch

When adding or debugging probes, prefer small dedicated modules under `Mod/Tests/`.

Recommended probe families to add as work progresses:

- `StoresAlienProbe`
- `StoresDomainProbe`
- `SequencerDomainProbe`
- `TextViewStateProbe`
- `HostPortsInputProbe`
- `ReloadTypedescProbe`

## Definition Of Done

Framework recovery is done only when all of the following are true:

1. CP modules, not Rust facades, own the framework behavior except where the host must own it.
2. Stores is the single persistence backbone and survives partial-framework documents.
3. MVC behavior is real end-to-end: focus, events, broadcast, deferred actions, and sequencing.
4. Text documents load into a real typed graph and behave like live views.
5. Host rendering/input is complete enough for resident editing and shell behavior.
6. Parity tests operate on real BlackBox-style artifacts, not just synthetic compiler probes.

## Maintenance

Update this document when one of these happens:

- an epic changes shape because reality disproved the current sequence
- a temporary layer is merged away or a new removal target appears
- a recovered subsystem changes the dependency order for downstream work
- a bug class proves the BRK policy needs a new standard probe

Do not update it just to make the project sound further along than the code is.
