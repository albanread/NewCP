# MVC Design for CP WinGui

## Overview

This document describes the Model-View-Controller architecture for NewCP's windowed
GUI layer.  The design is derived from BlackBox Component Builder's MVC framework
(`System/Models`, `System/Views`, `System/Controllers`) but adapted for wingui's
two-thread reality: a CP background thread for event dispatch and a D3D11 main
thread for frame rendering.

The key architectural decision is that the full MVC triad lives on the CP thread.
Models, controllers, and logical views may exchange arbitrarily granular synchronous
messages there, including BlackBox-style micro-view composition and targeted update
propagation.  The UI thread owns panes and rendering only.  It receives flattened,
pane-scoped render commands and executes them using host-native GPU code.

The pane split matters here.

- `surface` is the general-purpose MVC rendering target.
- `text-grid`, `rgba-pane`, `indexed-graphics`, and other fast panes remain
  specialized rendering substrates with narrower contracts.
- those fast panes may later be wrapped by special-purpose MVC views, but they are
  not acceptable stand-ins for the general MVC surface.
- the design should not drift into trying to implement ordinary document MVC by
  repurposing the existing fast panes as a brain-dead substitute for `surface`.

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
owns a `paneId` (resolved at startup) and participates in two channel-driven flows:

For ordinary MVC work, that pane should be a `surface`.  If a specialized fast
pane is ever used from MVC, it should be because a wrapper view intentionally
embeds that specialized substrate for a specific workload, not because the general
MVC surface contract was collapsed onto the wrong pane type.

The pane, not the individual embedded view, owns the fast channel.  A root CP view
is free to traverse many nested micro-views, but those micro-views do not each get
their own UI-thread mailbox.  Instead, the root view flattens the result into a
single pane-scoped render batch.

1. A **`WinDoc.Observer`** — called from the CP event thread when the model
   changes.  The observer posts a message to the pane's inbox via
  `WinFrame.PostPaneMsg` or records a frame command so that the UI thread can act on it.

2. A **host-side frame executor** — running on the D3D11/UI thread every
  ~16 ms.  It drains pane/frame command channels and issues `HostFrame.*`
  draw calls for only the dirty region.  The UI thread does not call CP view
  code directly.

The view never calls into the model.  It only reads exported model state (e.g.
`TextDoc.lines`) on the CP side when generating commands, or reacts to status
messages coming back from the UI thread.  The view protocol is message-based, not
callback-based.

```
MODULE TextEditor;
IMPORT WinDoc, WinFrame, TextDoc;
VAR pane*: INTEGER;

(* Observer — runs on CP event thread *)
PROCEDURE OnDocChanged*(docId, kind, detail: ARRAY OF SHORTCHAR);
  VAR dummy: INTSHORT;
BEGIN
  dummy := WinFrame.PostPaneMsg(pane, kind, detail)
END OnDocChanged;

PROCEDURE Init*(nodeId: ARRAY OF SHORTCHAR);
BEGIN
  IF WinFrame.ResolvePaneId(nodeId, pane) # 0 THEN
    WinDoc.AddObserver(TextDoc.Id, OnDocChanged)
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
  │                WinFrame.PostPaneMsg / FrameCommandQueue ───┐  │ │
  └─────────────────────────────────────────────────────────────┼──┘ │
                                                                │    │
  ┌─────────────────────────────────────────────────────────────┼──┐ │
  │              D3D11 / frame thread                           │  │ │
  │                                                             ▼  │ │
  │  spec_bind_runtime_run                                         │ │
  │    on_frame ──→ host frame executor                             │ │
  │                   WinFrame.PollPaneMsg / drain frame commands   │ │
  │                     → HostFrame.* draw calls                    │ │
  │                     → optional status / response messages ──────┘ │
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

### Channel C — pane / frame command channels (CP → UI thread)

- **Transport**: pane inboxes and/or dedicated frame command queues owned by the host.
- **Direction**: CP event thread → D3D11/UI thread.
- **Content**: dirty-region hints, flattened pane render batches, and pane-scoped requests.
- **Latency**: sub-frame (drained at the top of every `on_frame` call).
- **Analogy**: BlackBox `Views.UpdateIn(v, l,t,r,b, rebuild)` — a targeted partial
  repaint of a specific region of a specific view.

### Channel D — frame status / response channels (UI → CP)

- **Transport**: host-owned response/status queue.
- **Direction**: UI thread → CP event thread.
- **Content**: completion, visibility, hit-test results, selection/caret feedback,
  diagnostics, and other view-side responses.
- **Analogy**: BlackBox had no explicit equivalent because the UI and controller
  lived on one thread; in NewCP this channel replaces synchronous intra-thread calls.

No UI-thread execution of CP procedures is required.  All cross-thread behaviour
is expressed as channels.

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
  │       registers the slow-path spec builder on the CP thread
  │
  ├─ 2. WinLoop.Register("…", Controller.Handler)  [for each event]
  │
  ├─ 3. WinView.Render
  │       publishes initial spec; panes are created in wingui.dll
  │
  ├─ 4. TextEditor.Init("editorPane")
  │       ResolvePaneId → stores paneId
  │       WinDoc.AddObserver(TextDoc.Id, OnDocChanged)
  │
  └─ 5. WinLoop.Run  [blocks until window closed]
```

Steps 1–2 are wiring.  Step 3 creates the panes.  Step 4 connects model to view
for the frame path.  Step 5 drives the event loop; the frame thread runs
independently the whole time and drains pane-owned render channels.  No CP view
procedure is executed on the UI thread.

---

## BlackBox correspondence table

| BlackBox | NewCP wingui equivalent |
|---|---|
| `Models.Model` | CP module with exported state + `docId` string |
| `Stores.Domain` | `docId` string (document boundary by convention) |
| `Models.Broadcast(m, msg)` | `WinDoc.Notify(docId, kind, detail)` |
| `Views.View` | CP module with `paneId: INTEGER` |
| `Views.View.Restore(f, l,t,r,b)` | host-side frame executor drains pane/frame commands |
| `Views.View.HandleModelMsg(msg)` | `WinDoc.Observer` proc → `PostPaneMsg` |
| `Views.UpdateIn(v, l,t,r,b, rebuild)` | `WinFrame.PostPaneMsg(paneId, kind, detail)` |
| `Views.Update(v, rebuild)` | `WinView.Render` (slow path, full repaint) |
| `Models.Context` | `paneId: INTEGER` (view ↔ container handle) |
| `Controllers.Controller.HandleCtrlMsg` | `WinLoop.Handler` procedure |
| `Views.Frame` (repaint clip rect) | `detail` string passed via pane inbox |
| Single UI thread — no channel | CP thread + D3D11 thread → command/status channels |
| `TextModels.UpdateMsg{beg,end,delta}` | semantic text update message, preserved over channel payloads |
| `TextViews.PositionMsg{beg,end,focusOnly}` | viewport/selection request message, preserved as view-response or controller request |
| `Controllers.TrackMsg` (mouse drag) | `WinLoop.Handler` for drag/mouse events |
| `Stores.Operation` (undo) | Not yet designed — future work |
| `Models.BeginScript`/`EndScript` | Not yet designed — future work |

---

## BlackBox semantics to preserve

The important compatibility target is not byte-for-byte API matching; it is
preserving the **semantics** of BlackBox model/view/controller traffic.

### 1. Typed model messages

BlackBox models do not just say "changed".  They send structured messages:

- `Models.UpdateMsg` means "some content-affecting change happened".
- `TextModels.UpdateMsg` refines that with `op`, `beg`, `end`, and `delta`.
- `TextViews.PositionMsg` requests view motion / visibility without changing text.

NewCP must preserve that distinction.  A text document protocol therefore needs at least:

- structural text-change messages carrying operation kind and affected range,
- viewport / selection / caret visibility messages,
- style / ruler / metrics invalidation messages,
- full rebuild messages when the view cache cannot be updated incrementally.

Those semantic messages do **not** have to be the same as the UI-thread render
payload.  In the current design there are two levels:

- CP-side MVC messages, which may stay rich and document-specific.
- pane-side render commands, which must be flat, host-consumable, and independent
  of the original view tree.

This is the main architectural difference from the earlier callback-based drafts.
The view remains on the CP thread; only render output crosses to the pane.

### 2. Domain-wide fanout

BlackBox `Models.Broadcast(model, msg)` fans the message to every view in the model's
domain.  NewCP can preserve this with `WinDoc.Notify(docId, kind, detail)` plus
multiple observers.  The transport differs, but the semantics are the same:

- one model mutation,
- many observing views,
- each view updates itself independently.

### 3. Targeted repaint, not only full rebuild

BlackBox `TextViews.HandleModelMsg2` distinguishes between updates it can handle
incrementally and updates that force rebuild:

- insert/delete/replace → targeted `UpdateView(v, beg, end, delta)`
- unknown update → `RebuildView(v)`
- position message → `ShowRange`

NewCP must keep the same capability split.  The equivalent requirement is:

- targeted frame commands for line/cell/range redraw,
- explicit full-rebuild command when incremental repair is impossible,
- explicit show-range / ensure-visible command for controller-driven navigation.

### 4. Texts are more than glyph streams

BlackBox text messages include document-structure consequences:

- embedded views,
- style/ruler changes,
- scrolling/position state,
- undoable scripts and grouped operations.

So the final NewCP text/document protocol must eventually support:

- text mutation semantics,
- layout/style invalidation semantics,
- viewport semantics,
- undo/script grouping metadata.

That is the real equivalence target for reimplementing `Texts` over wingui.

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


