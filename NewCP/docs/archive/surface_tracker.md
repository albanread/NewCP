# Surface Tracker

## Purpose

This tracker turns the `surface_design.md` architecture into concrete work items
for a high-quality `surface` implementation in wingui.

This tracker is for the general MVC `surface` and its font manager.
It is not a tracker for `text-grid`, `rgba-pane`, `indexed-graphics`, or other
specialized fast panes.

---

## Status Summary

### Facts on the ground

- [x] `surface` exists as its own pane kind in the native path
- [x] `surface` batches are drained on the UI thread, not the language thread
- [x] pane-addressed `WinBatch` command transport is in place for `surface`
- [x] host-side surface command execution is distinct from `text-grid`
- [x] host-side surface presenter/resources are distinct from the legacy RGBA pane path
- [x] Direct2D and DirectWrite interop is live under wingui
- [x] `DrawTextRun`, `MeasureTextRun`, `CharIndexAtPoint`, and `PointAtCharIndex`
  are routed through DirectWrite-backed host exports
- [x] geometry, clip, offset, overlay, scrolling, and present commands are routed
  through surface-specific host execution
- [x] `InstallChildViewBounds` no longer drops on the floor; it is retained as
  host-owned per-surface-pane state

### Primary remaining gaps

- [ ] reusable `WinguiFontManager` component still needs to be factored out as a
  first-class host service
- [ ] final child-view composition semantics are not implemented yet; retained
  child bounds are stored and queryable, but not yet consumed by a render-time
  child composition stage
- [ ] `DrawPath` payloads and path semantics still need a richer final design
- [ ] `PresentHint` is still effectively a present request, not a richer policy
- [ ] MVC document/view consumers still need to move onto the upgraded surface path

### High-level direction

- [x] keep `surface` separate from the fast panes
- [x] keep execution ownership on the UI thread below the batch boundary
- [x] route general MVC drawing through Direct2D and DirectWrite-backed surface paths
- [ ] finish the reusable font manager and retained child-view composition layer

---

## Primitive Coverage

### Batch and channel status

- [x] typed per-pane `WinBatch` command stream exists
- [x] pane id is carried end-to-end through Rust host bridge and native host queue
- [x] native host command vocabulary includes dedicated surface commands for text,
  primitives, clip, offset, composition reset, and child-view bounds install
- [x] current transport is a shared host command queue with pane-addressed surface
  commands
- [ ] there is not a separate OS-level queue per surface pane

### Composition and lifecycle primitives

- [x] `PushClipRect`
- [x] `PopClipRect`
- [x] `PushOffset`
- [x] `PopOffset`
- [x] batch-start composition reset to prevent stack leakage across batches
- [x] `ScrollRect`
- [x] `PresentHint`
- [~] `InstallChildViewBounds` retained host-side state implemented; render-time
  child composition still pending

### Text primitives

- [x] `DrawTextRun`
- [x] `MeasureTextRun`
- [x] `CharIndexAtPoint`
- [x] `PointAtCharIndex`
- [~] text currently uses DirectWrite-backed host exports, but the reusable font
  manager and stronger text-style descriptor work are still pending

### Geometry primitives

- [x] `FillRect`
- [x] `StrokeRect`
- [x] `DrawLine`
- [x] `FillOval`
- [x] `StrokeOval`
- [x] `DrawArc`
- [~] `DrawPath` is routed and rendered, but still needs the final rich path-command
  model rather than the current limited payload shape

### Overlay and feedback primitives

- [x] `MarkRect`
- [x] `Caret`
- [x] `SelectionRange`
- [x] `FocusRing`

### Host query surface

- [x] pane layout queries exist
- [x] retained surface child-view bounds are queryable from the host cache
- [ ] no final child-view renderer consumes those bounds yet

---

## Workstreams

### Workstream 1 — D3D11 and DXGI compatibility

**Goal**: make the existing wingui graphics device compatible with Direct2D surface rendering.

**Files**: likely `multiwingui/src/wingui.cpp`, related context creation code, and any pane buffer creation code

**Status**: [x] substantially complete

Tasks:

1. [x] add BGRA support to D3D11 device creation
2. [x] audit pane-buffer usage for Direct2D-compatible surface drawing
3. [x] use DXGI surface interop for Direct2D bitmap-backed drawing into surface buffers
4. [x] validate that native build still succeeds after the interop changes
5. [ ] keep validating specialized panes under real mixed workloads

Acceptance criteria:

1. [x] wingui can create Direct2D-compatible render targets for `surface` buffers
2. [~] specialized panes continue to build; broader runtime regression coverage is still desirable

---

### Workstream 2 — DirectWrite and font manager foundation

**Goal**: add a reusable host-side font and text-layout service for `surface`.

**Files**: likely new font-manager files in `multiwingui`, plus public API updates in `include/wingui/wingui.h` and `include/wingui/spec_bind.h`

**Status**: [~] partial

Tasks:

1. [x] add DirectWrite factory creation and lifetime management
2. [ ] define `WinguiFontManager` and its internal caches as a distinct host component
3. [ ] define stable host-side text format keys and layout keys
4. [~] use system font resolution through DirectWrite for the current path
5. [ ] decide how application-provided fonts will be registered later

Acceptance criteria:

1. [ ] host can resolve and cache text formats independently of any pane
2. [~] host can create DirectWrite layouts for draw, measure, and hit-test, but not yet through a reusable font-manager abstraction

---

### Workstream 3 — Surface pane resources and executor

**Goal**: create a dedicated `surface` renderer path on the UI thread.

**Files**: likely `multiwingui/src/terminal.cpp`, `multiwingui/src/spec_bind.cpp`, and new Direct2D integration files

**Status**: [~] partial

Tasks:

1. [x] define dedicated host-owned surface presenter/resources in `terminal.cpp`
2. [x] bind `surface` pane buffers as Direct2D-capable render targets
3. [x] define host-owned clip and offset stacks for the `surface` executor
4. [x] wire current surface command execution to Direct2D and DirectWrite-backed exports
5. [ ] factor the current executor state into cleaner long-lived surface component types if that refactor still pays for itself

Acceptance criteria:

1. [x] `surface` panes have a distinct host-native render path
2. [x] batch drain calls the surface executor path for current text, geometry, composition, and overlay commands

---

### Workstream 4 — Text command replacement

**Goal**: replace the current atlas-based `surface` text approximation.

**Files**: `multiwingui/src/spec_bind.cpp`, `multiwingui/src/wingui.cpp`, new font-manager files, and related public headers

**Status**: [~] mostly implemented, not yet fully productized

Tasks:

1. [x] replace `DrawTextRun` implementation with DirectWrite layout draw
2. [x] replace `MeasureTextRun` with DirectWrite metrics
3. [x] replace `CharIndexAtPoint` with DirectWrite hit-testing
4. [x] replace `PointAtCharIndex` with DirectWrite caret query
5. [~] keep all four operations aligned around the same text-layout semantics; formal reusable caching remains pending
6. [ ] add richer style descriptors and explicit fallback policy where MVC needs them

Acceptance criteria:

1. [x] text draw, measure, and hit-test now run through the same DirectWrite-backed host layer
2. [x] caret and selection geometry derive from the same text engine family
3. [~] the old atlas-based approximation is no longer the main surface text path, but the final font-manager architecture still remains to be done

---

### Workstream 5 — Geometry command replacement

**Goal**: replace the current shader-vector approximation for general `surface` geometry with Direct2D primitives and paths.

**Files**: `multiwingui/src/spec_bind.cpp`, new Direct2D geometry helpers, and related headers

**Status**: [~] partial

Tasks:

1. [x] implement `FillRect` and `StrokeRect` with Direct2D-backed primitive drawing
2. [x] implement `DrawLine`, `FillOval`, and `StrokeOval`
3. [x] implement `DrawArc`
4. [ ] replace the current limited `DrawPath` payload/strategy with a richer path-command model

Acceptance criteria:

1. [x] geometry rendering respects the current host-owned clip and offset composition state
2. [~] most surface geometry no longer depends on the old vector path model, but the final high-fidelity `DrawPath` design remains open

---

### Workstream 6 — Batch payload evolution

**Goal**: grow the `surface` command vocabulary from placeholder payloads to real text and path descriptors.

**Files**: `NewCP/Mod/WinBatch.cp`, `NewCP/Mod/HostFrame.cp`, `NewCP/src/newcp-runtime/src/wingui_host.rs`, `multiwingui/include/wingui/spec_bind.h`, `multiwingui/src/spec_bind.cpp`

**Status**: [~] partial

Tasks:

1. [ ] define a richer text-style descriptor for `DrawTextRun`
2. [ ] define a real path-command descriptor for `DrawPath`
3. [~] current primitive payloads already carry enough data for the first slice, but richer stroke and fill style fields are still needed
4. [x] keep the current compatibility layer in place while the renderer is being upgraded
5. [x] add surface-specific lifecycle and composition payloads for clip, offset, reset, and child-view bounds install

Acceptance criteria:

1. [ ] the command protocol can express final text style and path geometry
2. [ ] the protocol no longer assumes the current simplified text and path models

---

### Workstream 7 — MVC consumption

**Goal**: connect the upgraded `surface` to real CP-side MVC views.

**Files**: likely `NewCP/Mod/Graph.cp` first, then additional CP-side view modules

**Status**: [ ] not started in earnest

Tasks:

1. keep `Graph.cp` as the initial surface demo, but move it onto the upgraded renderer
2. add a text-focused demo that exercises proportional text and hit-testing
3. add a mixed geometry and text demo with clips, offsets, and overlays
4. start using `surface` batches from actual CP-side MVC view logic rather than only fixed demo content

Acceptance criteria:

1. at least one real CP-side MVC view uses the upgraded `surface`
2. the demo proves text, overlays, and geometry are coherent together

---

### Workstream 8 — Child-view composition semantics

**Goal**: make retained child-view bounds participate in actual surface composition.

**Files**: likely `multiwingui/src/terminal.cpp`, related surface execution code, and the CP-side sites that emit `InstallChildViewBounds`

**Status**: [~] first host-state slice implemented

Tasks:

1. [x] retain `InstallChildViewBounds` per surface pane on the host
2. [x] clear retained child bounds at batch composition reset boundaries
3. [x] expose host-side lookup for retained child bounds
4. [ ] define the first real render-time child-view composition step that consumes those bounds
5. [ ] decide whether `child_id` remains surface-local or resolves to a stronger host identity

Acceptance criteria:

1. [x] child bounds are no longer dropped on receipt
2. [ ] a child view can be positioned by retained bounds during actual surface composition
3. [ ] the identity model for `child_id` is explicit and stable

---

### Workstream 9 — Surface presentation policy

**Goal**: decide whether `PresentHint` remains a thin present request or grows into richer policy.

**Files**: `NewCP/src/newcp-runtime/src/wingui_host.rs`, `multiwingui/src/terminal.cpp`, and any future CP-side presenter contract docs

**Status**: [~] thin implementation present

Tasks:

1. [x] route `PresentHint` end to end
2. [ ] decide whether MVC needs throttling, damage, sync, or coalescing semantics beyond a simple request-present
3. [ ] upgrade the payload only if real presenter requirements justify it

Acceptance criteria:

1. [x] current batches can request present without re-entering CP
2. [ ] any richer presentation semantics are explicit rather than implicit drift

---

## Immediate Next Slice

The recommended next implementation slice is:

1. define and implement the first real consumer of retained child-view bounds in
  surface composition
2. factor the current DirectWrite text path into a reusable `WinguiFontManager`
3. design the richer `DrawPath` payload and path-command model
4. only then revisit whether `PresentHint` needs richer policy

This keeps momentum on the two real remaining semantic gaps: child-view
composition and reusable text infrastructure.

---

## Explicit Non-Goals

1. do not add milestones that implement `surface` by aliasing it to `text-grid`
2. do not add milestones that implement `surface` by aliasing it to `rgba-pane`
3. do not treat retained child-bounds storage by itself as completed child-view composition
4. do not treat the current limited `DrawPath` payload as the finished path model
5. do not lower the quality bar just because the fast panes already exist
