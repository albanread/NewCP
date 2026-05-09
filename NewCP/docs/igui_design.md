# iGui — Integrated GUI for NewCP

## Purpose

iGui is an MDI windowing layer implemented directly inside `newcp-runtime`,
replacing the previous external `multiwingui` / `wingui.dll` host. It is
deliberately narrower than wingui:

- one main MDI frame plus N child windows
- each child window contains exactly one rendering surface
- one rendering primitive set, backed by Direct2D + DirectWrite
- one threadsafe event mailbox from the GUI to the NewCP language thread
- one fast command queue per surface pane from the language thread to the GUI
- the same MVC discipline previously documented for wingui, kept intact

iGui drops the declarative JSON spec, the C++ smart-diff reconciler, the
specialized fast panes (`text-grid`, `rgba-pane`, `indexed-graphics`,
sprite/indexed paths), and all native widget controls
(`button`, `textarea`, `select`, `checkbox`, etc.). Anything a CP module
wants to draw is drawn through the surface pane's command queue.

This document supersedes the wingui design set; those documents have been
moved to [archive/](archive/) for reference.

## Non-goals

- Cross-platform: iGui targets `x86_64-pc-windows-msvc` only. Other platforms
  remain free to build a different integrated GUI later behind the same CP
  surface; this document does not promise that surface is portable.
- Native widgets: no buttons, edit controls, listboxes, tabs, splitters, or
  RichEdit. If the application needs a button, it draws it on a surface and
  receives input events for that surface.
- Declarative layout: there is no spec, no JSON, no diff. Window structure is
  imperative: open frame, open children, draw into each child's surface.
- Custom-rendered docking, floating tear-off panels, modeless tool windows.
  Win32 MDI is the chrome; nothing more.
- Backwards compatibility with wingui CP modules. The `Win*` modules
  (`WinSpec`, `WinView`, `WinLoop`, `WinFrame`, `WinBatch`, `HostFrame`,
  `HostWindows`, `WinPayload`, `WinDoc`, `WinDoc`-style observer plumbing,
  `Console`) are not preserved. They will be replaced by a single `iGui`
  CP module surface.

## Process and thread model

iGui inverts the previous startup ownership rule. The GUI is the main
thread; the language runtime is launched after the GUI is up.

```
  Main thread (process startup)
  ─────────────────────────────
  newcp-driver enters GUI mode
    ├─ iGui::run()                          ← creates MDI frame, message pump
    │     ├─ register window classes
    │     ├─ create MDI frame window
    │     ├─ initialize D3D11 + Direct2D + DirectWrite
    │     ├─ initialize per-process font manager
    │     └─ spawn language thread (below)
    │
    └─ Win32 message loop until WM_QUIT
        ├─ dispatches WM_PAINT → drains pending pane command queue
        ├─ posts input events to the language mailbox
        └─ honors child-window create/close requests from the language

  Language thread (spawned by iGui::run after GUI is ready)
  ────────────────────────────────────────────────────────
  newcp-runtime kernel
    ├─ load and JIT modules
    ├─ run the application's entry command
    │     ├─ open MDI children with iGui.OpenChild
    │     ├─ register MVC observers, start controllers
    │     ├─ block in iGui.RunLoop reading the event mailbox
    │     │     ↓ on each event: dispatch to controllers
    │     │     ↓ controllers mutate models
    │     │     ↓ models notify; views build pane batches
    │     │     ↓ views push batches into per-pane command queues
    │     └─ exit when "__close" event arrives or app calls iGui.Quit
    └─ on app exit: language thread joins, GUI thread posts WM_QUIT
```

The language thread never owns an HWND, never holds a Direct2D resource,
never calls Win32 from a callback. The GUI thread never executes JIT'd
CP procedures and never touches the GC. Everything between them is
queue traffic.

### Headless mode

When `newcp-driver` is run without GUI mode (e.g. `load-module`, dump
commands, test invocations), iGui is not started. The language thread
runs as the only thread, and there is no event mailbox to read.

## Window model

There is a fixed window taxonomy:

| Window | Role | HWND class |
|---|---|---|
| **Frame** | Top-level MDI parent. Holds menu, status bar, and the MDI client area. Exactly one per process. | iGui custom class wrapping `MDIFRAME` semantics |
| **MDI client** | Win32 `MDICLIENT` child of the frame. Hosts all document windows. Created automatically by iGui::run. | `MDICLIENT` |
| **Child** | One MDI child window per open document. Each child contains exactly one surface pane filling its client area. | iGui custom class wrapping `WS_CHILD | MDICHILD` |

Children are opened and closed imperatively from the language thread:

```
PROCEDURE iGui.OpenChild(title: ARRAY OF SHORTCHAR;
                         VAR childId: INTEGER): INTSHORT;
PROCEDURE iGui.CloseChild(childId: INTEGER): INTSHORT;
PROCEDURE iGui.SetTitle (childId: INTEGER; title: ARRAY OF SHORTCHAR);
```

Each child owns one surface pane, identified by `paneId = childId`. There
is no separate pane resolution step — opening a child is the same act
as creating its surface. (Previous wingui designs used widget-tree node
ids and a deferred `ResolvePaneId` because of the JSON spec; iGui has
no spec.)

The frame's menu is configured with a small imperative API
(`iGui.SetMenu`, `iGui.AddMenuItem`) and emits events into the mailbox
when items are chosen. Standard MDI verbs (Cascade / Tile / Arrange /
Close All) are wired automatically.

## Rendering — Direct2D + DirectWrite

The renderer is a single, focused stack:

1. **D3D11 device** with BGRA support, owned by iGui at process startup.
2. **Direct2D factory** + **device** + per-frame **device contexts**.
3. **DirectWrite factory** + a process-wide **font manager** that resolves
   family/weight/style/stretch tuples and caches text formats and layouts.
4. Per surface pane: a swap chain (or DXGI bitmap target), a Direct2D
   render target view, a clip stack, an offset transform stack, and a
   retained child-view bounds table.

There is exactly one pane kind: **surface**. The surface contract is the
previous `display_primitives.md` set, kept intact.

### Surface command vocabulary

Carried directly from the archived display-primitives contract.

| Family | Commands |
|---|---|
| Lifecycle | `Clear`, `PresentHint` |
| Composition | `PushClipRect`, `PopClipRect`, `PushOffset`, `PopOffset`, `ScrollRect` |
| Geometry | `FillRect`, `StrokeRect`, `DrawLine`, `FillOval`, `StrokeOval`, `DrawArc`, `DrawPath` |
| Text | `DrawTextRun`, `MeasureTextRun`, `CharIndexAtPoint`, `PointAtCharIndex` |
| Overlays | `MarkRect`, `Caret`, `SelectionRange`, `FocusRing` |
| Composition state | `InstallChildViewBounds` |

`DrawTextRun`, `MeasureTextRun`, `CharIndexAtPoint`, and `PointAtCharIndex`
must all be answered against the same DirectWrite layout object. No
approximation split is allowed between draw, measure, and hit-test.

### What surface is not

- not a glyph-cell grid (no monospaced atlas)
- not a raw RGBA upload buffer
- not an indexed-palette pipeline
- not a sprite renderer

If a future workload genuinely needs one of those, it can be added as a
second pane kind later, but it is out of scope for the first iGui slice.

## Channels

Three channels, no callbacks across thread boundaries.

```
┌──────────────────────────────────────────────────────────────────────┐
│  GUI thread                            Language thread                │
│  ──────────                            ───────────────                │
│                                                                       │
│   Win32 input ─┐                                                      │
│                ├──► EVENT MAILBOX ──► iGui.NextEvent (blocking)       │
│   menu select ─┤   (M:1 SPMC)         → controller dispatch           │
│   close req  ──┘                                                      │
│                                                                       │
│   WM_PAINT  ◄───── PANE COMMAND QUEUE ◄── iGui.SubmitBatch            │
│   draws     ◄──── (per-pane SPSC ring) ◄── (one per surface)          │
│                                                                       │
│   measure /                                                           │
│   hit-test  ─────► EVENT MAILBOX ──► language reads as events         │
│   replies                              with kind = "surface-reply"    │
└──────────────────────────────────────────────────────────────────────┘
```

### Channel 1 — Event mailbox (GUI → language)

A single MPSC queue (multiple GUI-thread producers, one language consumer)
holding typed events. The language thread owns the only consumer and
blocks on it via `iGui.NextEvent(timeoutMs)`.

Event kinds (initial set):

| Kind | Carried fields |
|---|---|
| `key` | childId, vkey, scancode, mods, repeat, down/up |
| `char` | childId, codepoint, mods |
| `mouse` | childId, x, y, button, mods, down/up/move/wheel |
| `focus` | childId, gained |
| `resize` | childId, width, height |
| `paint` | childId — hint that a redraw is desired (rare; usually CP drives) |
| `close` | childId — user clicked the child's close box |
| `menu` | menuId, itemId |
| `frame-close` | the user closed the MDI frame |
| `surface-reply` | childId, requestId, reply payload (from MeasureTextRun, CharIndexAtPoint, etc.) |

Events are typed, not JSON. The mailbox transports a fixed
`IGuiEvent` struct.

### Channel 2 — Pane command queue (language → GUI)

One **SPSC** ring per surface pane. Producer: language thread. Consumer:
GUI thread, drained inside `WM_PAINT` (or a present tick).

The language side calls `iGui.SubmitBatch(childId, batch)` to enqueue a
fully built batch. If a batch with sequence `N+1` is submitted while
`N` is still pending, the new batch supersedes the old one before the
GUI thread executes either — stale intermediate frames are dropped on
purpose.

Batch envelope:

```text
PaneBatch {
    childId:  u64
    sequence: u64
    flags:    u32
    cmds:     Vec<SurfaceCmd>
}
```

The ring carries `Box<PaneBatch>` slots. Allocations happen on the
language side; ownership transfers to the GUI thread when consumed.

### Channel 3 — Surface-reply events (GUI → language)

Synchronous-style queries (`MeasureTextRun`, `CharIndexAtPoint`,
`PointAtCharIndex`) cannot be answered on the language thread because
the authoritative DirectWrite layout lives on the GUI thread. The
language thread submits a request command in a batch with a `requestId`,
then waits for a `surface-reply` event on the mailbox carrying the
same `requestId`.

`iGui.MeasureTextRun` etc. are blocking helpers in the iGui CP module
that hide this round-trip from MVC code.

## MVC roles on the language side

Unchanged from the archived [MVC_summary](archive/MVC_summary.md), but
expressed in iGui terms.

### Model

A plain CP module owning data and a short `docId` string. After mutation
it calls `iGui.Notify(docId, kind, detail)`. Models do not see panes,
children, or the GUI.

### View

A CP module that owns a `childId` (the pane). It registers as an
observer of one or more `docId`s. On each notification it builds a
`PaneBatch` for its child and submits it via `iGui.SubmitBatch`.

A view may compose arbitrarily nested logical micro-views on the
language side, but all output is flattened into a single batch per
pane per update. The GUI thread never sees the view tree.

### Controller

A handler procedure registered with `iGui.OnEvent(kind, handler)`. The
event loop calls it on the language thread when matching events arrive
from the mailbox. Controllers mutate models and may issue
view-targeted commands; they do not draw.

### `iGui.Notify` and observer registration

```
TYPE Observer = PROCEDURE (docId, kind, detail: ARRAY OF SHORTCHAR);

PROCEDURE iGui.AddObserver*    (docId: ARRAY OF SHORTCHAR; o: Observer);
PROCEDURE iGui.RemoveObserver* (docId: ARRAY OF SHORTCHAR; o: Observer);
PROCEDURE iGui.Notify*         (docId, kind, detail: ARRAY OF SHORTCHAR);
```

`Notify` calls observers synchronously on the language thread. There is
no broadcast across threads — the language thread owns the entire MVC
triad. Cross-thread traffic is only batches and events.

## CP-side surface — the `iGui` module

A single CP `DEFINITION MODULE` replaces the entire `Win*` family.

```
DEFINITION iGui;

  CONST
    BatchClear*, BatchPushClipRect*, BatchPopClipRect*,
    BatchPushOffset*, BatchPopOffset*,
    BatchFillRect*, BatchStrokeRect*, BatchDrawLine*,
    BatchFillOval*, BatchStrokeOval*, BatchDrawArc*, BatchDrawPath*,
    BatchDrawTextRun*, BatchMarkRect*, BatchCaret*,
    BatchSelectionRange*, BatchFocusRing*,
    BatchScrollRect*, BatchPresentHint*, BatchInstallChildViewBounds* : INTSHORT;

  TYPE
    Observer*  = PROCEDURE (docId, kind, detail: ARRAY OF SHORTCHAR);
    Handler*   = PROCEDURE (childId: INTEGER; payload: ARRAY OF SHORTCHAR);

  (* lifecycle *)
  PROCEDURE Quit*;
  PROCEDURE RunLoop*;                    (* blocking event loop on language thread *)

  (* windows *)
  PROCEDURE OpenChild* (title: ARRAY OF SHORTCHAR; VAR childId: INTEGER): INTSHORT;
  PROCEDURE CloseChild*(childId: INTEGER): INTSHORT;
  PROCEDURE SetTitle*  (childId: INTEGER; title: ARRAY OF SHORTCHAR);

  (* menu *)
  PROCEDURE SetMenu*   (spec: ARRAY OF SHORTCHAR);   (* compact textual menu spec *)

  (* events *)
  PROCEDURE OnEvent*   (kind: ARRAY OF SHORTCHAR; h: Handler);

  (* MVC observer plumbing *)
  PROCEDURE AddObserver*    (docId: ARRAY OF SHORTCHAR; o: Observer);
  PROCEDURE RemoveObserver* (docId: ARRAY OF SHORTCHAR; o: Observer);
  PROCEDURE Notify*         (docId, kind, detail: ARRAY OF SHORTCHAR);

  (* batches: build, then submit *)
  PROCEDURE BeginBatch* (childId: INTEGER);
  PROCEDURE EmitFillRect*  (x0, y0, x1, y1, r, g, b, a: REAL);
  PROCEDURE EmitStrokeRect*(x0, y0, x1, y1, halfThick, r, g, b, a: REAL);
  PROCEDURE EmitDrawLine*  (x0, y0, x1, y1, halfThick, r, g, b, a: REAL);
  PROCEDURE EmitDrawTextRun*(text: ARRAY OF SHORTCHAR;
                             x, y, fontSize: REAL;
                             family: ARRAY OF SHORTCHAR;
                             weight, style: INTSHORT;
                             r, g, b, a: REAL);
  PROCEDURE EmitPushClipRect* (x0, y0, x1, y1: REAL);
  PROCEDURE EmitPopClipRect*;
  PROCEDURE EmitPushOffset*   (dx, dy: REAL);
  PROCEDURE EmitPopOffset*;
  PROCEDURE EmitClear*        (r, g, b, a: REAL);
  PROCEDURE EmitCaret*        (x, y, height: REAL; r, g, b, a: REAL);
  PROCEDURE EmitSelectionRange*(x0, y0, x1, y1: REAL; r, g, b, a: REAL);
  (* … remaining surface commands per the table above … *)
  PROCEDURE SubmitBatch*();              (* enqueues the current batch, returns immediately *)

  (* synchronous queries (mailbox round-trip; block on reply) *)
  PROCEDURE MeasureTextRun*(childId: INTEGER;
                            text: ARRAY OF SHORTCHAR;
                            family: ARRAY OF SHORTCHAR;
                            fontSize: REAL;
                            VAR width, height, ascent: REAL): INTSHORT;
  PROCEDURE CharIndexAtPoint*(childId: INTEGER; runId: INTEGER;
                              x, y: REAL; VAR index: INTEGER): INTSHORT;
  PROCEDURE PointAtCharIndex*(childId: INTEGER; runId: INTEGER;
                              index: INTEGER; VAR x, y: REAL): INTSHORT;

END iGui.
```

There is intentionally no separate `Console`, `Log`, `WinPayload`, or
`HostFrame` module. Logging is done by drawing into a designated child;
event payloads are typed in the event struct, not in JSON.

(This is a sketch. The exact set of `Emit*` procedures vs. a single
`Emit(opcode, …)` form is open; the principle is that the language side
builds a typed batch and submits it, with no JSON in the path.)

## What is removed from the wingui design

| Removed | Replacement |
|---|---|
| `multiwingui` / `wingui.dll` external native host | `igui` Rust module inside `newcp-runtime` |
| JSON spec + spec_bind smart-diff | imperative `OpenChild`/`CloseChild` + per-pane batches |
| Native widgets (button, textarea, select, listbox, …) | drawn primitives on a surface; controllers handle input |
| `text-grid`, `rgba-pane`, `indexed-graphics`, sprite, palette panes | one `surface` pane kind |
| Spec rebuild + diff for state changes | direct surface batches |
| `WinSpec`, `WinView`, `WinLoop`, `WinFrame`, `WinBatch`, `HostFrame`, `HostWindows`, `WinPayload`, `WinDoc`, `Console` modules | one `iGui` CP module |
| Pane resolution (`ResolvePaneId` after first publish) | child id == pane id at creation time |
| `SUPERTERMINAL_WINDOW_FLAG_MDI_FRAME` opt-in MDI flag | MDI is the only mode |
| `kind/detail` string protocol on the pane wire | typed `SurfaceCmd` enum carried in `PaneBatch` |
| `PostPaneMsg` / `PollPaneMsg` semantic notification ring | not needed: views own all batch generation on the language thread |
| Slow-path full-spec re-publish for label/text changes | redraw the affected surface region |

## What carries forward

- the **MVC ownership rule**: models, controllers, views all on the
  language thread; only batches and events cross the boundary
- the **surface command vocabulary** (display primitives)
- the **DirectWrite-backed text contract**: draw / measure / hit-test
  must agree on the same layout
- the **font manager** as a process-wide reusable service
- **per-pane batch supersession**: newer sequence drops older pending
- **typed event mailbox** for input
- **child-view bounds retention** as host-owned state for nested view
  composition

## Crate structure

The implementation lives in `newcp-runtime` under a new module tree:

```
src/newcp-runtime/src/igui/
    mod.rs              — public entry: igui::run, igui::quit
    frame.rs            — MDI frame window, message pump
    child.rs            — MDI child window, surface attachment
    d3d.rs              — D3D11 device + DXGI swap chains
    d2d.rs              — Direct2D factory, device, contexts
    dwrite.rs           — DirectWrite factory + font manager
    surface_executor.rs — drains pane command queues, executes Direct2D ops
    channels.rs         — event mailbox + per-pane SPSC rings
    cp_exports.rs       — #[unsafe(export_name = "iGui.*")] shims
    Mod/iGui.cp         — DEFINITION MODULE for the language side
```

The existing `wingui_host.rs`, `wingui_ffi.rs`, `wingui_spec_ffi.rs` and
the related `WinSpec` / `HostFrame` / `WinBatch` exports are out of scope
for iGui. They remain compilable for now under the existing `gui` cargo
feature so test suites are not broken in one step; a follow-up phase
will retire them.

## Implementation phases

### Phase 1 — Frame, MDI client, D3D/D2D bring-up

1. Create the iGui module tree in `newcp-runtime` behind a new
   `--features igui` flag (parallel to the existing `gui` feature).
2. Register a frame window class and create the MDI frame + MDI client.
3. Initialize D3D11, Direct2D, DirectWrite, and a stub font manager.
4. Run a Win32 message loop that paints a solid colour into the frame.
5. No language thread yet; verify the GUI alone runs cleanly.

Acceptance: `newcp-driver run-igui` opens a window, paints, closes
cleanly.

### Phase 2 — Language thread + event mailbox

1. Spawn the existing kernel/runtime on a worker thread once the frame
   is up.
2. Add the event mailbox (MPSC, bounded). Wire WM_KEY*, WM_CHAR,
   WM_MOUSE*, WM_SIZE, WM_CLOSE, WM_COMMAND into typed events.
3. Add `iGui.NextEvent`, `iGui.OnEvent`, `iGui.RunLoop`,
   `iGui.AddObserver`, `iGui.Notify`, `iGui.Quit`.
4. Provide the `Mod/iGui.cp` DEFINITION module.

Acceptance: a CP module can register a key handler and receive typed
key events.

### Phase 3 — Child windows + surface command queue

1. Add MDI child window class, `iGui.OpenChild`, `iGui.CloseChild`,
   `iGui.SetTitle`.
2. Allocate one SPSC pane batch ring per child.
3. Implement the surface executor: drain the pending batch on
   `WM_PAINT`, execute the simplest commands (`Clear`, `FillRect`,
   `StrokeRect`, `DrawLine`).
4. Add `iGui.BeginBatch` / `iGui.Emit*` / `iGui.SubmitBatch`.

Acceptance: a CP module opens two children and paints a different
geometry into each, end to end.

### Phase 4 — Text via DirectWrite

1. Wire the font manager: format cache, family/weight/style/stretch
   resolution, layout cache.
2. Implement `EmitDrawTextRun`.
3. Implement the synchronous `MeasureTextRun`,
   `CharIndexAtPoint`, `PointAtCharIndex` round-trips via the
   `surface-reply` event.

Acceptance: a CP module renders proportional text and round-trips a
hit-test that matches caret geometry.

### Phase 5 — Composition + overlays

1. Implement clip stack, offset stack, `ScrollRect`.
2. Implement `MarkRect`, `Caret`, `SelectionRange`, `FocusRing`.
3. Implement `InstallChildViewBounds` retention + retrieval.
4. Implement `DrawPath` with a real path-command stream.

Acceptance: a CP-side text view exercises selection, caret blinking,
and clipped scrolling without flicker.

### Phase 6 — Menu + standard MDI commands

1. Add `iGui.SetMenu` / `iGui.AddMenuItem` and the `menu` event kind.
2. Wire automatic Cascade / Tile / Arrange / Close All verbs.

Acceptance: a multi-document demo with a working File / Window menu.

### Phase 7 — Retire wingui

1. Delete `wingui_host.rs`, `wingui_ffi.rs`, `wingui_spec_ffi.rs`.
2. Delete the `Win*` and `HostFrame` / `HostWindows` / `Console`
   DEFINITION modules under `Mod/`.
3. Migrate `App.cp`, `Graph.cp`, `WinDoc.cp`, `WinView.cp`,
   `WinLoop.cp` to the new `iGui` API or delete them.
4. Drop the `gui` cargo feature in favour of `igui`.

Acceptance: `newcp-runtime` builds without any wingui references and
the demo modules under `Mod/` run on iGui.

## Open questions

- **Per-child swap chain vs. shared swap chain.** A shared swap chain
  on the frame's MDI client area is simpler and probably enough for
  the first slice; per-child swap chains buy us flicker-free
  child resizing later. Decision deferred to Phase 3.
- **Batch builder ergonomics.** A flat list of `Emit*` procedures is
  easy to bind but unwieldy for large batches. A second pass might
  introduce a typed `SurfaceCmd` record array passed across the FFI
  in one call.
- **Animation tick.** iGui does not yet have a heartbeat. If
  applications need 60fps redraw they will currently re-submit batches
  in response to a timer event. A future phase may add a `tick`
  event kind at a configurable rate.
- **Font manager surface.** The font manager is host-owned for now.
  CP code passes family / size / weight / style by value on each text
  run. A later optimization can introduce opaque format handles to
  cut per-run resolution cost.
- **Synchronous query timeout.** `MeasureTextRun` and friends block
  on a reply event. A pathological GUI hang would deadlock the
  language thread. Decide whether to add a timeout + diagnostic
  fallback or treat a stalled GUI thread as a hard process error.

## Anti-goals (preventing future drift)

1. Do not reintroduce native Win32 widgets to the spec layer. If a
   widget is needed, draw it.
2. Do not reintroduce JSON or any string-shaped wire format on the
   pane channel.
3. Do not collapse `surface` into a fast specialized pane. If a
   workload truly needs cell-text or raw RGBA, propose a second pane
   kind explicitly.
4. Do not allow the GUI thread to call CP procedure pointers, even via
   a clever "deferred" mechanism. Cross-thread traffic stays as
   typed messages.
5. Do not allow MVC traffic (model notify / observer dispatch) to
   cross threads. The language thread owns the entire MVC triad.
