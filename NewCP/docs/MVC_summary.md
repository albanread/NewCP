# MVC Design for CP WinGui

## Overview

This document describes the Model-View-Controller architecture for NewCP's windowed
GUI layer.  The design is derived from BlackBox Component Builder's MVC framework
(`System/Models`, `System/Views`, `System/Controllers`) but adapted for wingui's
two-thread reality: a CP background thread for event dispatch and a D3D11 main
thread for frame rendering.

---

## The three roles

### Model

A **model** owns data.  It has no knowledge of how it is displayed.  After any
mutation it announces the change by calling `WinDoc.Notify(docId, kind, detail)`.
The `detail` string carries a machine-readable hint about what changed (e.g. a
line range, a cell coordinate, a selection boundary) so that observers can repaint
only the affected region rather than the whole pane.

A model is a plain CP module with exported state — no base class, no store, no
domain.  Its `docId` is just a short string agreed between the model and its
observers.

Example:

```
MODULE TextDoc;
IMPORT WinDoc;
CONST Id* = "textdoc1";
VAR lines*: ARRAY 4096 OF SHORTCHAR;
    lineCount*: INTEGER;

PROCEDURE Insert*(pos: INTEGER; s: ARRAY OF SHORTCHAR);
BEGIN
  (* mutate lines … *)
  WinDoc.Notify(Id, "text", "12,15")   (* rows 12-15 changed *)
END Insert;

PROCEDURE SetCursor*(row, col: INTEGER);
BEGIN
  (* update cursor state … *)
  WinDoc.Notify(Id, "cursor", "")
END SetCursor;

END TextDoc.
```

The `kind` string is a loose vocabulary agreed between model and views:

| kind | Meaning of detail |
|---|---|
| `"text"` | `"firstRow,lastRow"` — line range that changed |
| `"cursor"` | `""` or `"row,col"` |
| `"selection"` | `"beg,end"` character positions |
| `"scroll"` | `"row"` new top line |
| `"reload"` | `""` — full content replaced |

---

### View

A **view** renders a model into a pane.  In wingui a view is a CP module that
owns a `paneId` (resolved at startup) and registers two things:

1. A **`WinDoc.Observer`** — called from the CP event thread when the model
   changes.  The observer posts a message to the pane's inbox via
   `WinFrame.PostPaneMsg` so that the frame thread can act on it.

2. A **`WinFrame.PaneProc`** — called from the D3D11 frame thread every
   ~16 ms.  It drains the pane inbox with `WinFrame.PollPaneMsg` and
   issues `HostFrame.*` draw calls for only the dirty region.

The view never calls into the model.  It only reads exported model state (e.g.
`TextDoc.lines`) and issues draw calls.  Reading exported model state from the
frame thread is safe as long as the model only mutates on the CP event thread
(which it always does — `WinLoop` dispatches handlers serially).

```
MODULE TextEditor;
IMPORT WinDoc, WinFrame, HostFrame, TextDoc;
VAR pane*: INTEGER;

(* Observer — runs on CP event thread *)
PROCEDURE OnDocChanged*(docId, kind, detail: ARRAY OF SHORTCHAR);
  VAR dummy: INTSHORT;
BEGIN
  dummy := WinFrame.PostPaneMsg(pane, kind, detail)
END OnDocChanged;

(* PaneProc — runs on D3D11 frame thread *)
PROCEDURE OnFrame*(paneId: INTEGER);
  VAR kind, detail: ARRAY 64 OF SHORTCHAR;
      dummy: INTSHORT;
BEGIN
  LOOP
    IF WinFrame.PollPaneMsg(paneId, kind, detail) = 0 THEN EXIT END;
    IF StrEq(kind, "text")   THEN RedrawLineRange(paneId, detail)
    ELSIF StrEq(kind, "cursor") THEN RedrawCursor(paneId) END
  END
END OnFrame;

PROCEDURE Init*(nodeId: ARRAY OF SHORTCHAR);
BEGIN
  IF WinFrame.ResolvePaneId(nodeId, pane) # 0 THEN
    WinDoc.AddObserver(TextDoc.Id, OnDocChanged);
    WinFrame.RegisterPaneRenderer(pane, OnFrame)
  END
END Init;

END TextEditor.
```

---

### Controller

A **controller** translates user input (keyboard, mouse, scroll) into model
mutations and view commands.  In wingui, controllers are `WinLoop.Handler`
procedures registered for the relevant event names.  They run on the CP event
thread.

```
MODULE TextController;
IMPORT WinLoop, WinPayload, TextDoc, TextEditor, WinView;

PROCEDURE OnKey*(name, payload: ARRAY OF SHORTCHAR);
  VAR ch: INTEGER; dummy: INTSHORT;
BEGIN
  dummy := WinPayload.GetInt(payload, "key", ch);
  TextDoc.Insert(TextDoc.cursor, (* ch as string … *));
  (* WinDoc.Notify fires inside Insert; no explicit Render needed for frame path *)
END OnKey;

PROCEDURE OnScroll*(name, payload: ARRAY OF SHORTCHAR);
  VAR delta: INTEGER; dummy: INTSHORT;
BEGIN
  dummy := WinPayload.GetInt(payload, "delta", delta);
  TextDoc.Scroll(delta);
END OnScroll;

PROCEDURE Register*;
BEGIN
  WinLoop.Register("texteditor_key",    OnKey);
  WinLoop.Register("texteditor_scroll", OnScroll)
END Register;

END TextController.
```

Controllers never touch `HostFrame` directly and never post to pane inboxes.
They only call model procedures and occasionally `WinView.Render` (if a slow-path
widget like a toolbar button label also needs to update).

---

## Communication channels

There are three distinct communication paths, each serving a different purpose.

```
  ┌──────────────────────────────────────────────────────────────────┐
  │                        CP event thread                           │
  │                                                                  │
  │  WinLoop.Run                                                     │
  │    HostWindows.WaitNamedEvent ← wingui event queue               │
  │      → Controller.Handler                                        │
  │          → Model.Mutate                                          │
  │              → WinDoc.Notify ──────────────────────────────────┐ │
  │                  ↓ slow-path observers                         │ │
  │                WinView.Render  (JSON spec → PublishUi)         │ │
  │                  ↓ frame-path observers                        │ │
  │                WinFrame.PostPaneMsg ────────────────────────┐  │ │
  └─────────────────────────────────────────────────────────────┼──┘ │
                                                                │    │
  ┌─────────────────────────────────────────────────────────────┼──┐ │
  │              D3D11 / frame thread                           │  │ │
  │                                                             ▼  │ │
  │  spec_bind_runtime_run                                         │ │
  │    on_frame ──→ WinFrame dispatcher                            │ │
  │                   ↓ per pane                                   │ │
  │                 PaneProc(paneId)                               │ │
  │                   WinFrame.PollPaneMsg ←── inbox ring buf ─────┘ │
  │                     → HostFrame.* draw calls                     │
  └──────────────────────────────────────────────────────────────────┘
```

### Channel A — event queue (host → CP)

- **Transport**: `WinguiSpecBindRuntime` → Rust `on_event` → `EVENT_QUEUE` →
  `HostWindows.WaitNamedEvent` unblocks on CP thread.
- **Direction**: host → CP event thread.
- **Content**: named event + JSON payload (button clicked, key pressed, etc.).
- **Latency**: a few milliseconds — fine for UI reactions, not for animation.
- **Analogy**: BlackBox `Controllers.Controller.HandleCtrlMsg` receiving user input.

### Channel B — JSON spec (CP → host)

- **Transport**: `WinView.Render` → `HostWindows.PublishUi(json)` → Rust →
  `wingui_spec_bind_runtime_load_spec_json` → C++ smart diff → minimal patch.
- **Direction**: CP event thread → host.
- **Content**: full declarative layout description (widget tree, values, labels).
- **Latency**: 5–20 ms round trip; only for slow-path UI changes (labels, values,
  showing/hiding widgets).
- **Analogy**: BlackBox `Views.Update(v, rebuild)` triggering a full `Restore`
  repaint via the host's diff engine.

### Channel C — pane inbox (CP → frame thread)

- **Transport**: `WinFrame.PostPaneMsg` → lock-free MPSC ring buffer in
  `wingui.dll` → `WinFrame.PollPaneMsg` on frame thread.
- **Direction**: CP event thread → D3D11 frame thread.
- **Content**: short `(kind, detail)` string pair — a dirty-region hint.
- **Latency**: sub-frame (the ring buffer is drained at the top of every `on_frame`
  call, so within ~16 ms of the `PostPaneMsg` call).
- **Analogy**: BlackBox `Views.UpdateIn(v, l,t,r,b, rebuild)` — a targeted partial
  repaint of a specific region of a specific view.

No other cross-thread communication is needed.  Model state is written only on
the CP event thread and read only on that thread (by the event-path) or read-only
from the frame thread (safe because the model is not mutated during `on_frame`).

---

## Multiple views of one model

A model can have any number of views registered as observers.  Each view owns its
own pane; each calls `WinDoc.AddObserver(docId, …)` independently.  When the
model calls `WinDoc.Notify`, every observer is called in registration order.
Each observer posts to its own pane inbox.  The frame thread then repaints each
pane independently.

```
  TextDoc (model)
      │
      ├── WinDoc.Notify("text", "12,15")
      │       │
      │       ├── TextEditor.OnDocChanged  → PostPaneMsg(editorPane, …)
      │       ├── MiniMap.OnDocChanged     → PostPaneMsg(minimapPane, …)
      │       └── LineNumbers.OnDocChanged → PostPaneMsg(lineNumberPane, …)
      │
      └── (slow-path) WinView.Render  (update line count label etc.)
```

This is identical to BlackBox's behaviour when a text model is displayed in a
split-view: both pane views receive the same `TextModels.UpdateMsg` and repaint
their own dirty region.

---

## Life cycle

```
App.Run
  │
  ├─ 1. WinView.SetRenderer(BuildWindow)
  │       registers the slow-path spec builder
  │
  ├─ 2. WinFrame.SetRenderer(FallbackOnFrame) [optional global fallback]
  │
  ├─ 3. WinLoop.Register("…", Controller.Handler)  [for each event]
  │
  ├─ 4. WinView.Render
  │       publishes initial spec; panes are created in wingui.dll
  │
  ├─ 5. TextEditor.Init("editorPane")
  │       ResolvePaneId → stores paneId
  │       WinDoc.AddObserver(TextDoc.Id, OnDocChanged)
  │       WinFrame.RegisterPaneRenderer(pane, OnFrame)
  │
  └─ 6. WinLoop.Run  [blocks until window closed]
```

Steps 1–3 are wiring.  Step 4 creates the panes.  Step 5 connects model to view
for the frame path.  Step 6 drives the event loop; the frame thread runs
independently the whole time.

---

## BlackBox correspondence table

| BlackBox | NewCP wingui equivalent |
|---|---|
| `Models.Model` | CP module with exported state + `docId` string |
| `Stores.Domain` | `docId` string (document boundary by convention) |
| `Models.Broadcast(m, msg)` | `WinDoc.Notify(docId, kind, detail)` |
| `Views.View` | CP module with `paneId: INTEGER` |
| `Views.View.Restore(f, l,t,r,b)` | `PaneProc(paneId)` registered via `WinFrame.RegisterPaneRenderer` |
| `Views.View.HandleModelMsg(msg)` | `WinDoc.Observer` proc → `PostPaneMsg` |
| `Views.UpdateIn(v, l,t,r,b, rebuild)` | `WinFrame.PostPaneMsg(paneId, kind, detail)` |
| `Views.Update(v, rebuild)` | `WinView.Render` (slow path, full repaint) |
| `Models.Context` | `paneId: INTEGER` (view ↔ container handle) |
| `Controllers.Controller.HandleCtrlMsg` | `WinLoop.Handler` procedure |
| `Views.Frame` (repaint clip rect) | `detail` string passed via pane inbox |
| Single UI thread — no channel | CP thread + D3D11 thread → `PostPaneMsg`/`PollPaneMsg` ring buffer |
| `TextModels.UpdateMsg{beg,end,delta}` | `(kind="text", detail="firstRow,lastRow")` |
| `Controllers.TrackMsg` (mouse drag) | `WinLoop.Handler` for drag/mouse events |
| `Stores.Operation` (undo) | Not yet designed — future work |
| `Models.BeginScript`/`EndScript` | Not yet designed — future work |

---

## What is deliberately not ported

| BlackBox feature | Reason omitted |
|---|---|
| `Stores.Store` / persistence / serialisation | NewCP has its own file I/O; no need for the BB store graph |
| `Stores.Domain` as typed object | A string `docId` is sufficient for the observer pattern |
| `Stores.Operation` / undo graph | Useful later; not required for the initial wingui design |
| `Views.Frame` tree / recursive embedding | Replaced by flat pane IDs — simpler and sufficient for all planned widgets |
| `Properties.Property` / `PropMessage` | No property inspector planned; widgets are spec-driven |
| `Controllers.PollOpsMsg` / clipboard | CP clipboard support is a separate future module |
| `Fonts` / `Ports` / `Printers` | The host (wingui.dll / DirectWrite) owns all font and print concerns |
