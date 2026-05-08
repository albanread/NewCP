# CP WinGui — Greenfield Design

## Goals

- CP modules never see or construct JSON.  That is purely the Rust layer's concern.
- The declarative builder (`WinSpec`) is stateless: call it, get a spec, done.
- A small framework (`WinView`, `WinLoop`) provides an MVC event backbone so any
  module can register handlers for the events it cares about without a monolithic
  switch statement.
- The whole stack runs on a background CP thread; the Rust main thread drives the
  D3D11 message loop independently.
- Cross-thread integration is always **channel-based, never callback-based**:
  the UI thread may emit events and status messages to the CP thread, and the CP
  thread may emit UI commands and repaint requests to the UI thread, but the UI
  thread must never execute CP/JIT procedures directly.
- The NewCP compiler must support the features listed in the **Compiler
  requirements** section; these are things to add to the language if not yet present.

---

## Layer map

```
┌──────────────────────────────────────────────────────────┐
│  Application modules                                     │
│  App, Factorial, Log, Editor, Graph, Game, …             │
│  — register handlers, mutate model state, call WinView   │
├──────────────────────────────────────────────────────────┤
│  Framework  (pure CP)                                    │
│  WinView   — owns spec buffer, spec builder, publish      │
│  WinLoop   — blocking event loop, handler dispatch table │
│  WinFrame  — pane resolution, pane messaging, frame state │
├──────────────────────────────────────────────────────────┤
│  Host interface  (CP definition modules, Rust shims)     │
│  WinSpec   — stateless declarative layout builder        │
│  HostWindows — PublishUi, WaitNamedEvent, RequestClose   │
│  HostFrame — text-grid, RGBA, indexed, sprites, vector   │
├──────────────────────────────────────────────────────────┤
│  wingui.dll  (C++ / Direct3D 11)                         │
│  spec_bind runtime — smart diff, patch, reconcile        │
│  per-frame draw — text_grid, rgba, indexed, sprites,     │
│                   vector, assets                         │
└──────────────────────────────────────────────────────────┘
```

---

## Compiler requirements

The following CP features must be supported by NewCP for this design.
Mark each as implemented when done.

| Feature | Notes |
|---|---|
| Procedure types | `TYPE Handler = PROCEDURE (name, payload: ARRAY OF SHORTCHAR)` |
| Procedure variables | `VAR h: Handler; h := MyProc` |
| Exported VAR | `VAR text*: ARRAY 4096 OF SHORTCHAR` readable by other modules |
| RECORD types | `TYPE Entry = RECORD … END` |
| Fixed arrays of RECORD | `VAR table: ARRAY 64 OF Entry` |
| BOOLEAN | `b: BOOLEAN; b := TRUE` |
| INTSHORT | 32-bit integer (`i32`); use for parameters whose Rust shim uses `i32` (gaps, readonly flags, etc.) |
| REAL | 32-bit float (`f32`); required for vector drawing coordinates and colours |

---

## WinSpec — stateless declarative builder

Backed entirely by a Rust thread-local builder.  CP modules call these procedures
to construct a layout description; `GetSpec` writes the finished JSON into a
caller-supplied buffer.  CP never reads or parses the JSON.

### Container procedures

```
PROCEDURE Begin*(title: ARRAY OF SHORTCHAR);
PROCEDURE OpenStack*(gap: INTSHORT);          (* vertical, gap px or -1 = default *)
PROCEDURE OpenRow*(gap: INTSHORT);            (* horizontal *)
PROCEDURE OpenSplitH*(dividerSize: INTSHORT); (* horizontal split, 2 panes follow *)
PROCEDURE OpenSplitV*(dividerSize: INTSHORT); (* vertical split *)
PROCEDURE OpenSplitPane*(size: INTSHORT);     (* fractional 0-100, or -1 = auto *)
PROCEDURE OpenTabs*(selectedId: ARRAY OF SHORTCHAR);
PROCEDURE AddTab*(label, id: ARRAY OF SHORTCHAR); (* opens tab body frame *)
PROCEDURE CloseContainer*;
PROCEDURE GetSpec*(VAR buf: ARRAY OF SHORTCHAR): INTSHORT;  (* 1=ok, 0=overflow *)
```

### Leaf widget procedures

```
PROCEDURE AddButton*(id, label, event: ARRAY OF SHORTCHAR);
PROCEDURE AddText*(text: ARRAY OF SHORTCHAR);
(* textarea — Windows RichEdit control; slow-path only; for user-editable multi-line text *)
PROCEDURE AddTextarea*(id, label, value: ARRAY OF SHORTCHAR; readonly: INTSHORT);
(* text-grid — hardware-accelerated monospace coloured-text surface;
             each cell holds a UTF-32 codepoint + foreground + background colour;
             the host renders the whole grid to the GPU every frame via a
             glyph atlas shader — no JSON, no Windows controls, no GDI;
             designed for code editors, terminals, log viewers *)
PROCEDURE AddTextGrid*(id, event: ARRAY OF SHORTCHAR; cols, rows: INTSHORT);
PROCEDURE AddInput*(id, label, value, event: ARRAY OF SHORTCHAR);
PROCEDURE AddCheckbox*(id, label: ARRAY OF SHORTCHAR; checked: INTSHORT; event: ARRAY OF SHORTCHAR);
PROCEDURE AddSelect*(id, label, value: ARRAY OF SHORTCHAR);
PROCEDURE AddOption*(value, label: ARRAY OF SHORTCHAR);  (* append to last AddSelect *)
PROCEDURE AddListBox*(id, label, value: ARRAY OF SHORTCHAR; event: ARRAY OF SHORTCHAR);
PROCEDURE AddRadioGroup*(id, label, value: ARRAY OF SHORTCHAR; event: ARRAY OF SHORTCHAR);
(* AddOption works for all three multi-choice widgets *)
```

### textarea vs text-grid

| Widget | Spec type | Backing | When to use |
|---|---|---|---|
| `AddTextarea` | `textarea` | Windows RichEdit control | User-editable rich text; value lives in JSON spec; updated via WinView.Render on the slow event path |
| `AddTextGrid` | `text-grid` | Hardware-accelerated monospace grid: each cell = codepoint + fg/bg colour; rendered via glyph atlas shader every frame | Code editors, terminals, log viewers, any display needing per-cell colour at 60 fps; no JSON round-trip |

### Design notes

- `OpenSplitH`/`OpenSplitV` require exactly two `OpenSplitPane` children before
  `CloseContainer`.
- `OpenTabs` requires `AddTab` + content children + `CloseContainer` per tab, then
  `CloseContainer` for the tabs widget itself.
- All boolean-flavoured parameters use `INTSHORT` (0 = false, non-zero = true).
  This maps cleanly to `i32` in Rust shims.
- No `UpdateTextarea` and no `PatchUi`.  The C++ spec_bind runtime already diffs
  old vs new JSON and sends a minimal patch automatically.  CP always rebuilds the
  full spec and calls `HostWindows.PublishUi`; the host handles the rest.

---

## HostWindows — host primitives

```
PROCEDURE PublishUi*(json: ARRAY OF SHORTCHAR);
PROCEDURE RequestClose*;
PROCEDURE RequestPresent*;
PROCEDURE WaitNamedEvent*(VAR name:    ARRAY OF SHORTCHAR;
                          VAR payload: ARRAY OF SHORTCHAR;
                          timeoutMs:   INTEGER): INTSHORT;
  (* Returns 1 on event delivered, 0 on timeout.
     Pass timeoutMs = -1 to block indefinitely.
     timeoutMs is INTEGER (i64) to avoid truncation of large values.
     name receives the event id string.
     payload receives the raw JSON payload from the host.
     CP code does not need to parse payload unless it needs widget values;
     use WinPayload helpers for that. *)
```

The HostWindows surface will grow `OpenChildWindow` / `CloseChildWindow` to
support an MDI-frame mode in which one top-level window hosts multiple
document windows as Win32 MDI children. See [mdi_design.md](mdi_design.md)
for the full design and phased plan; this section will be updated when
Phase 4 of that design lands the new exports.

### Rust ABI rules (fixed, must not break)

- CP fixed arrays (`ARRAY N OF T`) and open arrays (`ARRAY OF T`) are both
  passed as a **bare pointer** — no length word.  Rust shims must not declare a
  length parameter.
- CP `VAR` formal → pointer to the variable.
- CP `INTEGER` = 64-bit `i64`.  Rust shims must use `i64` for INTEGER params.
- CP `INTSHORT` = 32-bit `i32`.  Use for parameters and return values where a
  32-bit value is natural (gap sizes, readonly flags, 0/1 results).  This avoids
  silent truncation in either direction.
- Return type `INTEGER` → Rust `i64`; return type `INTSHORT` → Rust `i32`.
- `BOOLEAN` → not yet used across the ABI; use `INTSHORT` (0/1) for now.

---

## WinPayload — event payload helpers

A small pure-CP module that extracts named string fields from the JSON payload
without the rest of CP needing to know JSON syntax.  Backed by a Rust shim
that does the actual JSON parsing.

```
(* Extract a string field by key from a JSON payload.
   Returns 1 if found, 0 if absent or not a string. *)
PROCEDURE GetStr*(payload, key: ARRAY OF SHORTCHAR;
                  VAR out: ARRAY OF SHORTCHAR): INTSHORT;

(* Extract an integer field. *)
PROCEDURE GetInt*(payload, key: ARRAY OF SHORTCHAR;
                  VAR out: INTEGER): INTSHORT;

(* Extract a boolean field as 0/1. *)
PROCEDURE GetBool*(payload, key: ARRAY OF SHORTCHAR;
                   VAR out: INTSHORT): INTSHORT;
```

Typical use in an input event handler:
```
PROCEDURE OnSearchChanged*(name, payload: ARRAY OF SHORTCHAR);
  VAR text: ARRAY 512 OF SHORTCHAR;
BEGIN
  IF WinPayload.GetStr(payload, "value", text) # 0 THEN
    Search.SetQuery(text);
    WinView.Render
  END
END OnSearchChanged;
```

---

## WinView — render dispatch and publish

Owns the spec buffer.  Application code registers a single **render procedure**
that calls `WinSpec.*` to describe the current state of the window.  `WinView.Render`
calls that procedure, reads the spec into its buffer, then calls `HostWindows.PublishUi`.

```
MODULE WinView;

TYPE
  RenderProc = PROCEDURE;           (* render proc calls WinSpec.* to build layout *)

PROCEDURE SetRenderer*(p: RenderProc);
  (* Register the procedure that builds the window spec.
     Must be called before the first Render. *)

PROCEDURE Render*;
  (* Call the registered render proc, capture the spec, publish to host.
     The C++ diff engine sends only what changed since the last publish.
     Safe to call from any CP code after state changes. *)

PROCEDURE SetTitle*(title: ARRAY OF SHORTCHAR);
  (* Change the window title; takes effect on next Render. *)
```

### Spec buffer

`WinView` owns `VAR spec: ARRAY SpecMax OF SHORTCHAR` internally.  No other module
holds or passes a spec buffer.  `SpecMax = 16384` is sufficient for all practical
layouts.

### Example render procedure

```
PROCEDURE BuildWindow;
BEGIN
  WinSpec.Begin(WinView.title);
  WinSpec.OpenStack(-1);
    WinSpec.OpenRow(-1);
      WinSpec.AddButton("run_factorial", "Factorial 20", "run_factorial");
      WinSpec.AddButton("clear_log",     "Clear",        "clear_log");
    WinSpec.CloseContainer;
    WinSpec.AddTextarea("log", "Log", Log.text, 1);
  WinSpec.CloseContainer
END BuildWindow;
```

`Log.text` is an exported module VAR (`text*: ARRAY 4096 OF SHORTCHAR`), so the
render procedure reads it directly — no getter function, no JSON, no buffer copies.

---

## WinLoop — event dispatch

Provides a **handler registration table** so any module can subscribe to the
events it cares about.  The event loop itself is one blocking call.

```
MODULE WinLoop;

TYPE
  Handler = PROCEDURE (name, payload: ARRAY OF SHORTCHAR);

CONST
  MaxHandlers = 64;

PROCEDURE Register*(event: ARRAY OF SHORTCHAR; h: Handler);
  (* Register h to be called when an event with the given name arrives.
     Multiple modules can register for the same event name; all are called
     in registration order.
     "__close_requested" and "__host_stopping" are reserved; register with
     OnClose for those. *)

PROCEDURE OnClose*(h: Handler);
  (* Register a handler called on __close_requested or __host_stopping. *)

PROCEDURE Run*;
  (* Block until a close/stop event is dispatched.
     On each event: find all registered handlers for that event name and call
     them in order.  Unrecognised events are silently dropped.
     After Run returns, CP code should exit cleanly. *)
```

### Dispatch algorithm (inside Run)

```
LOOP
  ok := HostWindows.WaitNamedEvent(name, payload, -1);
  IF ok # 0 THEN
    IF StrEq(name, "__close_requested") OR StrEq(name, "__host_stopping") THEN
      (* call close handlers, EXIT *)
    ELSE
      i := 0;
      WHILE i < count DO
        IF StrEq(table[i].event, name) THEN table[i].handler(name, payload) END;
        INC(i)
      END
    END
  END
END
```

No giant application-level CASE or IF chain.  Each module registers its own handler
at startup and is entirely responsible for its own response.

---

## Application module pattern

### App.cp — wire-up only

```
MODULE App;
IMPORT WinLoop, WinView, Factorial, Log;

PROCEDURE BuildWindow;
BEGIN
  (* described above — pure WinSpec calls *)
END BuildWindow;

PROCEDURE Run*;
BEGIN
  WinView.SetRenderer(BuildWindow);
  WinLoop.OnClose(App.OnClose);
  WinLoop.Register("run_factorial", Factorial.OnRun);
  WinLoop.Register("clear_log",     Log.OnClear);
  WinView.Render;
  WinLoop.Run
END Run;

PROCEDURE OnClose*(name, payload: ARRAY OF SHORTCHAR);
BEGIN
  (* cleanup if needed *)
END OnClose;

END App.
```

### Factorial.cp — handles its own event

```
MODULE Factorial;
IMPORT Log, WinView;

PROCEDURE Value*(n: INTEGER): INTEGER;
  VAR i, r: INTEGER;
BEGIN r := 1; i := 2; WHILE i <= n DO r := r * i; INC(i) END; RETURN r END Value;

PROCEDURE OnRun*(name, payload: ARRAY OF SHORTCHAR);
  VAR r: INTEGER;
BEGIN
  r := Value(20);
  Log.String("20! = "); Log.Int(r, 0); Log.Ln;
  WinView.Render   (* push updated log text to window *)
END OnRun;

END Factorial.
```

### Log.cp — text buffer, exported VAR

```
MODULE Log;
CONST TextMax = 4096;
VAR text*: ARRAY TextMax OF SHORTCHAR;  (* exported: WinView render reads this directly *)
    textLen: INTEGER;

PROCEDURE Open*;
PROCEDURE Clear*;
PROCEDURE String*(s: ARRAY OF SHORTCHAR);
PROCEDURE Ln*;
PROCEDURE Int*(n, width: INTEGER);

PROCEDURE OnClear*(name, payload: ARRAY OF SHORTCHAR);
BEGIN Clear END OnClear;

END Log.
```

`Log` is a pure data module with no GUI dependencies.  The view layer reads `Log.text` during `WinView.Render`.

---

## Module dependency graph

```
App ──────────────────────────────┐
 │                                │
 ├── WinView ── WinSpec            │
 │         └── HostWindows        │
 │                                │
 ├── WinLoop ── HostWindows       │
 │                                │
 ├── Factorial ── Log             │
 │            └── WinView         │
 │                                │
 └── Log  (no GUI deps)           │
                                  ▼
                          WinPayload ── HostWindows (Rust)
```

No cycles.  `Log` has zero GUI dependencies — it can be used from non-GUI code too.

---

## Summary of Rust work required

| Shim | Action |
|---|---|
| `WinSpec.*` existing | Keep Begin/OpenStack/OpenRow/CloseContainer/AddButton/AddText/AddTextarea/GetSpec — fix `i32` params to `i64` where they are CP INTEGER |
| `WinSpec.UpdateTextarea` | **Remove** |
| `HostWindows.PatchUi` | **Remove** |
| `HostWindows.WaitNamedEvent` timeout | Change `i32` → `i64` |
| `WinSpec.AddInput` | **Add** |
| `WinSpec.AddCheckbox` | **Add** |
| `WinSpec.AddSelect` + `AddOption` | **Add** |
| `WinSpec.AddListBox` | **Add** |
| `WinSpec.AddRadioGroup` | **Add** |
| `WinSpec.AddTextGrid` | **Add** — `text-grid` widget (cols, rows) for fast-path text display |
| `WinSpec.OpenSplitH/V/Pane` | **Add** |
| `WinSpec.OpenTabs` + `AddTab` | **Add** |
| `WinPayload.GetStr/GetInt/GetBool` | **Add** (new module shim) |
| `WinLoop` | Pure CP — no new Rust needed |
| `WinView` | Pure CP — no new Rust needed |
| `WinFrame.PostPaneMsg` | **Add** — forward pane-scoped messages into host-owned channels |
| `WinFrame.PollPaneMsg` | **Add** — drain pane-scoped messages on the owning side |
| `WinFrame.FrameIndex/ElapsedMs/DeltaMs` | **Add** — read frame timing from stored frame_view |
| `WinFrame.ResolvePaneId` | **Add** — `resolve_pane_id_utf8` wrapper (init-time) |
| `WinFrame.PaneLayout` | **Add** — `frame_get_pane_layout` wrapper |
| `WinFrame.RequestPresent` | **Add** — `frame_request_present` wrapper |
| `HostFrame.TextGridWriteCell` | **Add** — single-cell helper over `frame_text_grid_write_cells` |
| `HostFrame.TextGridClearRegion` | **Add** — `frame_text_grid_clear_region` wrapper |
| `HostFrame.RgbaUpload` | **Add** — `frame_rgba_upload` (CP pixel buffer → pane) |
| `HostFrame.RgbaGpuCopy` | **Add** — `frame_rgba_gpu_copy` wrapper |
| `HostFrame.RegisterAsset` | **Add** — `frame_register_rgba_asset_owned` wrapper |
| `HostFrame.BlitAsset` | **Add** — `frame_asset_blit_to_pane` wrapper |
| `HostFrame.DefineSprite` | **Add** — `frame_define_sprite` wrapper |
| `HostFrame.RenderSprites` | **Add** — `frame_render_sprites` wrapper |
| `HostFrame.IndexedUpload` | **Add** — `frame_indexed_graphics_upload` wrapper |
| `HostFrame.IndexedFillRect` | **Add** — `frame_indexed_fill_rect` wrapper |
| `HostFrame.IndexedDrawLine` | **Add** — `frame_indexed_draw_line` wrapper |
| `HostFrame.DrawLine` | **Add** — `frame_draw_line` wrapper (vector) |
| `HostFrame.FillRect` | **Add** — `frame_fill_rect` wrapper |
| `HostFrame.StrokeRect` | **Add** — `frame_stroke_rect` wrapper |
| `HostFrame.FillCircle` | **Add** — `frame_fill_circle` wrapper |
| `HostFrame.StrokeCircle` | **Add** — `frame_stroke_circle` wrapper |
| `HostFrame.DrawArc` | **Add** — `frame_draw_arc` wrapper |
| `HostFrame.DrawText` | **Add** — `frame_draw_text_utf8` wrapper |

---

## High-speed per-frame path

The slow event path (JSON spec → PublishUi → C++ diff) has enough latency for UI
reactions but is too slow for smooth animation, text-editor cursor blink, real-time
graphs, or games.  The host runtime still runs a dedicated frame tick at
`target_frame_ms` (typically 16 ms, ~60 fps) on the D3D11 main thread, but that
tick is a **host-side drain point**, not a direct call into CP.  The CP thread
records draw/update commands and posts them to the UI thread; the UI thread drains
those commands during `on_frame` and applies them to text-grid / surface panes.

### Principle: channels, not callbacks

- UI-thread work must remain host-native: D3D11, Win32, spec_bind, pane uploads,
  vector rasterisation, text-grid updates.
- Language-thread work must remain on the CP thread: event dispatch, model mutation,
  controller logic, document bookkeeping, command generation.
- A frame pane may have **multiple response channels** back to the CP side
  (status, completion, hit-test results, diagnostics, selection updates), but those
  responses are delivered as queue messages/events and then handled by `WinLoop` or
  another CP-side dispatcher.
- No design in this stack should require the UI thread to call a CP procedure pointer
  directly.  If the UI thread needs something from CP, it sends a message and returns.

### Thread model

```
  D3D11 main thread                    CP background thread
  ─────────────────                    ────────────────────
  spec_bind_runtime_run()
    ├─ on_event() ─────── EVENT_QUEUE ──→ WaitNamedEvent unblocks
    │                                       WinLoop dispatches handlers
    │                                       handler mutates model state
    │                                       (WinView.Render can be called here
    │                                        to push an updated JSON spec)
    │
    ├─ on_frame() drains FRAME_COMMAND_QUEUE
    │      → applies HostFrame / text-grid / vector / sprite work natively
    │      → may emit FRAME_STATUS_QUEUE messages (done / visible / hit-test / etc.)
    │
    └────────────────────────────────────→ CP records frame commands / repaint intents
                                            posts them to frame command queues
                                            consumes frame status/events via WinLoop
```

The D3D11 thread never calls a CP procedure pointer directly.  The UI thread only
executes host-native rendering code.  CP communicates desired drawing through
commands; the host may communicate results or observations back through response
messages.  This preserves thread ownership cleanly and matches the broader NewCP
host model.

### Widget types for the frame path

| Widget spec type | Declared via | Backing surface | Per-frame draw APIs |
|---|---|---|---|
| `text-grid` | `WinSpec.AddTextGrid` | Hardware-accelerated monospace grid (cols × rows cells); each cell stores a UTF-32 codepoint, foreground colour, and background colour; the host uploads the cell buffer to the GPU and renders it via a glyph-atlas shader every frame | `TextGridWriteCell`, `TextGridClearRegion` |
| `surface` | `WinSpec.AddSurface` | General MVC drawing surface for BlackBox-style views, controllers, overlays, embedded content, and document text.  Text on this pane is DirectWrite-backed and not cell-based. | `WinBatch` surface commands, routed by pane id |

`text-grid` is the correct surface for a CP code editor or terminal.  It is
entirely separate from `textarea` (a Windows RichEdit control) and from `surface`
(the general BlackBox-oriented MVC pane).  The cell buffer lives in GPU-visible memory; calling
`TextGridWriteCell` queues a single-cell update that the host applies in the same
frame — no JSON diff, no Windows message, no GDI.

The fast panes are not second-class or deprecated.  They are the correct place
for workloads that want tight, specialized rendering contracts: monospaced text
editing, games, sprite systems, indexed graphics, direct pixel pipelines, and
similar high-throughput features.  They may later appear inside MVC as wrapped or
embedded special-purpose views, but that does not change the role of `surface`.

`surface` is the generic pane for ordinary MVC GUI work: layout-owned drawing,
controller feedback, nested clips, scroll offsets, semantic view repaint, and
document-style text.  It is intentionally not the same as `rgba-pane`,
`indexed-graphics`, or `text-grid`, because those specialized panes remain
available for throughput-driven workloads with narrower rendering contracts.

So the rule is:

- general-purpose MVC rendering targets `surface`
- specialized high-speed rendering targets the existing fast panes
- MVC integration for those fast panes, when needed, is a wrapper/view design
  problem, not a reason to collapse the pane types together

The text split is deliberate:

- `text-grid` is a very fast monospaced cell surface for code editors and terminals.
- `surface` is the broader MVC document surface and must use DirectWrite-backed
  text layout, measurement, hit-testing, and rendering.

`textarea` is a **Windows RichEdit control** on the slow JSON spec path.  It is
not a pane and cannot be resolved for per-frame drawing.

Both `text-grid` and `surface` panes are resolved at startup with
`WinFrame.ResolvePaneId(nodeId, paneId)` — store the returned `paneId` and reuse
it every frame.

### Content buffer modes

| Mode | CP constant | Behaviour |
|---|---|---|
| `FRAME` | `WinFrame.BufFrame = 0` | Scratch buffer — host clears it before each frame.  Safe to draw from scratch every call. |
| `PERSISTENT` | `WinFrame.BufPersistent = 1` | Retained between frames.  Only re-draw the region you changed.  Best for editors and graphs that update partially. |

---

## WinFrame — frame command / status bridge

Pure CP module.  Resolves pane ids and provides the bridge between CP-side model /
controller logic and the UI-thread frame executor.  The final design is queue-based:
CP posts frame commands or pane messages, and the UI thread drains them during
`on_frame`.  `frame_view` remains a host-side concept; CP should not depend on being
called from the UI thread.

```
MODULE WinFrame;

CONST
  BufFrame      = 0;   (* SUPERTERMINAL_RGBA_CONTENT_BUFFER_FRAME *)
  BufPersistent = 1;   (* SUPERTERMINAL_RGBA_CONTENT_BUFFER_PERSISTENT *)

(* Frame timing — reported by the host, readable from CP as metadata *)
PROCEDURE FrameIndex*(): INTEGER;
PROCEDURE ElapsedMs*(): INTEGER;    (* ms since runtime start *)
PROCEDURE DeltaMs*(): INTEGER;      (* ms since previous frame *)

(* Pane resolution — call at startup, not during the host frame drain *)
PROCEDURE ResolvePaneId*(nodeId: ARRAY OF SHORTCHAR; VAR paneId: INTEGER): INTSHORT;
  (* Resolves the widget node id (e.g. "mySurface") to an opaque pane integer.
     Returns 1 on success, 0 if not yet laid out.  Call after WinView.Render.
  Store paneId as a module VAR and use it when posting frame commands. *)

PROCEDURE PaneLayout*(paneId: INTEGER;
                      VAR x, y, width, height: INTSHORT): INTSHORT;
  (* Fill pixel rect and visible flag of a previously resolved pane.
     Returns 1 if layout is valid this frame, 0 if the pane is hidden. *)

PROCEDURE PostPaneMsg*(paneId: INTEGER;
           kind, detail: ARRAY OF SHORTCHAR): INTSHORT;
  (* Cross-thread pane mailbox.  Use for invalidation hints, small control messages,
  status replies, and other pane-scoped coordination.  This is not permission for
  the UI thread to run CP code; it is a message channel only. *)

PROCEDURE PollPaneMsg*(paneId: INTEGER;
                       VAR kind: ARRAY OF SHORTCHAR;
                       VAR detail: ARRAY OF SHORTCHAR): INTSHORT;
  (* Drain pane-scoped messages on whichever side owns the consumer. *)


Direct CP frame callbacks on the UI thread were an intermediate experiment and are
not the desired final architecture.  The final MVC-friendly design is command-driven.
PROCEDURE RequestPresent*;
  (* Force a Present this frame even if auto_request_present is off.
     Usually not needed; call only when skipping draw but still need to flush. *)

END WinFrame.
```

### Two levels of messages

The latest design separates CP-side semantics from UI-side rendering more
strictly than the earlier drafts.

On the CP thread, models, controllers, and logical views may exchange whatever
fine-grained messages they need: `TextModels.UpdateMsg`, cursor motion,
micro-view invalidation, selection changes, and so on.  Those messages are part
of the language-side MVC system.

Across the pane boundary, only two kinds of traffic should exist:

1. **Semantic pane notifications**
  Small messages such as `"text"`, `"cursor"`, `"selection"`, or
  `"rebuild"` with compact details.  These are useful for invalidation,
  scheduling, and coarse coordination.

2. **Flattened render batches**
  Host-consumable drawing instructions for a specific pane.  These are not
  view callbacks.  They are the display-list result of CP-side view
  composition.

The rule is that micro-views and nested logical views may be arbitrarily rich on
the CP side, but they do not each get their own UI-thread protocol.  They
flatten into one pane batch.

### Target pane render-command vocabulary

`kind/detail` is sufficient for scheduling and invalidation, but it is not the
final fast-path payload.  The next stable target is a typed pane render batch
with a small, fixed vocabulary that the UI thread can execute directly.

At the architectural level, the pane batch should support at least:

| Command family | Purpose | Typical producers |
|---|---|---|
| `begin_batch` / `end_batch` | Delimit one pane update and carry sequence / version metadata | any root view |
| `set_clip_rect` | Establish clipping for a pane region or nested micro-view | text layout, embedded views |
| `clear_rect` | Clear scratch or retained regions before redraw | editors, graphs, surface views |
| `write_cells` | Update a rectangular text-grid region | text editors, terminals |
| `draw_text_run` | Draw DirectWrite-backed proportional or positioned text runs | rich text, headings, inline widgets, documents |
| `fill_rect` / `stroke_rect` | Basic box painting | selections, backgrounds, borders |
| `draw_line` / `draw_arc` / `fill_circle` | Vector primitives | graphs, diagrams, handles |
| `sprite_batch` / `blit_asset` | Repeated image or sprite instances | embedded icons, games, previews |
| `set_scroll_offset` | Apply host-side scroll / translation without recomputing all geometry | editors, large documents |
| `caret_overlay` / `selection_overlay` | High-frequency editor overlays | text controllers, text views |

This vocabulary is intentionally pane-oriented rather than widget-oriented.  The
UI thread should not need to know whether a command came from a paragraph view,
an inline button, or a graph widget.  It only needs a pane id plus flat drawing
operations.

### Immediate transport shape

The current inbox API remains useful, but its role should now be treated as:

- `PostPaneMsg(kind, detail)` for semantic invalidation and scheduling.
- the existing typed `WinBatch` payload for actual pane draw commands.

That means the practical near-term flow is:

1. CP model mutation triggers `WinDoc.Notify`.
2. Root CP view observes the change and decides whether it needs incremental
  redraw or full rebuild.
3. CP view flattens the affected pane content into a render batch.
4. CP submits a typed `WinBatch` for the pane and may also post a pane-scoped
  notification when semantic coordination is needed.
5. The host frame drain consumes that batch and issues `HostFrame.*` work.

The key architectural point is that the UI thread executes host-native drawing,
not CP view logic.

For retained panes, the host should treat batches as **replaceable per pane**.
If pane `P` already has a pending batch with sequence `N`, and CP submits a new
batch for the same pane with sequence `N+1`, the newer batch should supersede the
older one before frame execution.  This avoids rendering stale intermediate states
when the CP side produces updates faster than the next frame drain.

### Compiler note: pane IDs

`SuperTerminalPaneId` is a `uint64_t` in C.  CP `INTEGER` is `i64` — the same
width.  Pass `VAR paneId: INTEGER` and the Rust shim writes the raw 64-bit value.
Use CP `INTEGER` throughout for pane IDs.

---

## HostFrame — frame-time drawing primitives

CP definition module backed by Rust shims.  These procedures are executed by the
host-side frame drain on the UI thread after consuming queued frame commands.
Coordinates are in pixels from the top-left of the pane.  Colours are passed as
four separate `REAL` values (0.0–1.0 each: r, g, b, a).

For `surface`, text must be resolved through DirectWrite-backed layout objects,
not through the `text-grid` glyph atlas.  Drawing, measurement, caret placement,
selection geometry, and hit-testing must all agree on the same host-side layout.
The current host bridge already exports `DrawTextRun`, `MeasureTextRun`,
`CharIndexAtPoint`, and `PointAtCharIndex`; the remaining work is to replace the
placeholder atlas-based implementation under those exports with the DirectWrite
text engine and font manager.

```
MODULE HostFrame;
IMPORT WinFrame;

(* ── Text grid ──────────────────────────────────────────────── *)

(* Write a single UTF-32 codepoint at (col, row) with colour.
   Colours as packed ARGB integers for ABI simplicity. *)
PROCEDURE TextGridWriteCell*(paneId: INTEGER;
                             row, col: INTSHORT;
                             codepoint: INTEGER;
                             fg, bg: INTEGER): INTSHORT;

PROCEDURE TextGridClearRegion*(paneId: INTEGER;
                               row, col, width, height: INTSHORT;
                               fillCodepoint: INTEGER;
                               fg, bg: INTEGER): INTSHORT;

(* ── RGBA pixel upload ──────────────────────────────────────── *)

(* Upload a CPU-side BGRA8 pixel buffer to the pane.
   pixels must stay alive until this call returns (Rust copies immediately).
   width, height, sourcePitch in pixels/bytes.
   bufMode: WinFrame.BufFrame or WinFrame.BufPersistent. *)
PROCEDURE RgbaUpload*(paneId: INTEGER;
                      VAR pixels: ARRAY OF SHORTCHAR;
                      width, height, sourcePitch: INTSHORT;
                      bufMode: INTSHORT): INTSHORT;

(* GPU-to-GPU blit between two panes (or same pane, different regions). *)
PROCEDURE RgbaGpuCopy*(srcPaneId: INTEGER; srcX, srcY: INTSHORT;
                       dstPaneId: INTEGER; dstX, dstY: INTSHORT;
                       regionW, regionH: INTSHORT): INTSHORT;

(* ── RGBA assets (register once, blit many) ─────────────────── *)

(* Register a BGRA8 image as a named asset.  Returns assetId (> 0 on success).
   The pixel buffer is consumed; caller must not free it. *)
PROCEDURE RegisterAsset*(VAR pixels: ARRAY OF SHORTCHAR;
                         width, height, sourcePitch: INTSHORT;
                         VAR assetId: INTEGER): INTSHORT;

PROCEDURE BlitAsset*(assetId: INTEGER;
                     srcX, srcY, regionW, regionH: INTSHORT;
                     paneId: INTEGER;
                     dstX, dstY: INTSHORT): INTSHORT;

(* ── Indexed (palette) graphics ─────────────────────────────── *)

(* Upload an 8-bit indexed pixel buffer.
   palette: ARRAY 256 OF INTEGER packed ARGB.  Colour 0 is transparent. *)
PROCEDURE IndexedUpload*(paneId: INTEGER;
                         VAR pixels: ARRAY OF SHORTCHAR;
                         bufW, bufH, screenW, screenH: INTSHORT;
                         VAR palette: ARRAY OF INTEGER): INTSHORT;

PROCEDURE IndexedFillRect*(paneId: INTEGER;
                           x, y, width, height: INTSHORT;
                           paletteIndex: INTSHORT): INTSHORT;

PROCEDURE IndexedDrawLine*(paneId: INTEGER;
                           x0, y0, x1, y1: INTSHORT;
                           paletteIndex: INTSHORT): INTSHORT;

(* ── Sprites ────────────────────────────────────────────────── *)

(* Define a sprite into pane's bank.  spriteId is client-chosen (> 0).
   pixels: R8_UINT strip (frameW * frameCount wide x frameH tall).
   palette: 16-entry ARGB array; colour 0 is transparent. *)
PROCEDURE DefineSprite*(paneId: INTEGER; spriteId: INTSHORT;
                        frameW, frameH, frameCount, framesPerTick: INTSHORT;
                        VAR pixels: ARRAY OF SHORTCHAR;
                        VAR palette: ARRAY OF INTEGER): INTSHORT;

(* Render a batch of sprite instances.  spriteTick drives animation;
   pass WinFrame.FrameIndex() for steady animation.
   instances: flat array of packed instance records (see WinFrame docs). *)
PROCEDURE RenderSprites*(paneId: INTEGER;
                         spriteTick: INTEGER;
                         targetW, targetH: INTSHORT;
                         VAR instances: ARRAY OF SHORTCHAR;
                         count: INTSHORT): INTSHORT;

(* ── Vector drawing ─────────────────────────────────────────── *)
(* All coordinate and colour parameters use REAL (f32).
   bufMode: WinFrame.BufFrame or WinFrame.BufPersistent.
   blendMode: 0 = opaque, 1 = alpha-over.
   clearBefore: 1 to clear the surface before drawing (only first draw call
   each frame should pass 1; subsequent calls in the same frame pass 0). *)

PROCEDURE DrawLine*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, halfThickness: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE FillRect*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    x0, y0, x1, y1, cornerRadius: REAL;
                    r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeRect*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      x0, y0, x1, y1, halfThickness, cornerRadius: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE FillCircle*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                      clearR, clearG, clearB, clearA: REAL;
                      cx, cy, radius: REAL;
                      r, g, b, a: REAL): INTSHORT;

PROCEDURE StrokeCircle*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                        clearR, clearG, clearB, clearA: REAL;
                        cx, cy, radius, halfThickness: REAL;
                        r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawArc*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                   clearR, clearG, clearB, clearA: REAL;
                   cx, cy, radius, halfThickness, rotationRad, halfApertureRad: REAL;
                   r, g, b, a: REAL): INTSHORT;

PROCEDURE DrawText*(paneId: INTEGER; bufMode, blendMode, clearBefore: INTSHORT;
                    clearR, clearG, clearB, clearA: REAL;
                    text: ARRAY OF SHORTCHAR;
                    originX, originY: REAL;
                    r, g, b, a: REAL): INTSHORT;

END HostFrame.
```

### Compiler requirement: REAL

Vector drawing procedures use `REAL` (single-precision float, f32 in Rust).
NewCP must support `REAL` as a parameter and variable type for these shims.
Add to the compiler requirements table.

---

## Usage patterns

### Text editor / log display (text-grid path)

Place a `text-grid` widget in the spec with a given id using `WinSpec.AddTextGrid`.
Resolve the pane id once after the first `WinView.Render`.
When the model changes, enqueue targeted text-grid update commands for the UI thread;
use `PERSISTENT` mode to avoid repainting the whole buffer every frame.

```
MODULE Editor;
IMPORT WinFrame, HostFrame;
CONST Cols = 80; Rows = 40;
VAR pane: INTEGER; ready: INTEGER;

PROCEDURE Init*(nodeId: ARRAY OF SHORTCHAR);
  VAR ok: INTSHORT;
BEGIN
  ok := WinFrame.ResolvePaneId(nodeId, pane);
  IF ok # 0 THEN ready := 1 END
END Init;

PROCEDURE QueueHelloUpdate;
  (* pseudocode: record two text-grid write commands for pane *)
BEGIN
END QueueHelloUpdate;

END Editor.
```

Register in App.Run:
```
WinView.Render;   (* publish spec so pane is created *)
Editor.Init("myEditor");
WinLoop.Run
```

### Real-time graph (vector path)

```
MODULE Graph;
IMPORT WinFrame, HostFrame;
VAR pane: INTEGER;

PROCEDURE OnFrame;
  VAR i: INTEGER; x0, y0, x1, y1: REAL; dummy: INTSHORT;
BEGIN
  (* clear once per frame, then draw N line segments *)
  dummy := HostFrame.FillRect(pane, WinFrame.BufFrame, 0, 1,
                               0.1, 0.1, 0.1, 1.0,  (* clear colour *)
                               0.0, 0.0, 800.0, 400.0, 0.0,
                               0.1, 0.1, 0.1, 1.0);
  (* subsequent calls pass clearBefore=0 *)
  i := 0;
  WHILE i < DataLen - 1 DO
    x0 := i * 10.0; y0 := 200.0 - Data[i] * 2.0;
    x1 := (i+1) * 10.0; y1 := 200.0 - Data[i+1] * 2.0;
    dummy := HostFrame.DrawLine(pane, WinFrame.BufFrame, 0, 0,
                                 0.0, 0.0, 0.0, 0.0,
                                 x0, y0, x1, y1, 1.5,
                                 0.2, 0.8, 0.4, 1.0);
    INC(i)
  END
END OnFrame;

END Graph.
```

### Game / animation (sprite path)

```
MODULE Game;
IMPORT WinFrame, HostFrame;
CONST ShipId = 1;
VAR pane: INTEGER; defined: INTEGER;

PROCEDURE InitSprites;
  (* called once — loads pixels from a module VAR buffer *)
  VAR dummy: INTSHORT;
BEGIN
  dummy := HostFrame.DefineSprite(pane, ShipId, 16, 16, 4, 2,
                                   ShipPixels, ShipPalette);
  defined := 1
END InitSprites;

PROCEDURE OnFrame;
  (* pack instances into a flat SHORTCHAR buffer and call RenderSprites *)
END OnFrame;

END Game.
```

---

## MVC extension — pane channels, observable models, and frame inbox

`WinView.Render` remains the slow-path spec publication entry point.  To support
document-style MVC — where multiple independent panes each observe their own
model — pane-scoped channels are needed for the fast path.

### 1 — Per-pane frame channels in `WinFrame`

Each pane gets one or more command / status channels, not a language callback bound
to the UI thread.  A model/controller module on the CP thread can target a specific
pane by id, and the UI thread can send pane-scoped responses back without ever
executing CP directly.

```
PROCEDURE PostPaneMsg*(paneId: INTEGER; kind, detail: ARRAY OF SHORTCHAR): INTSHORT;
PROCEDURE PollPaneMsg*(paneId: INTEGER; VAR kind, detail: ARRAY OF SHORTCHAR): INTSHORT;
(* Additional dedicated command/status queues may be added later for richer frame
   traffic, but they follow the same ownership rule: channel yes, callback no. *)
```

This is the direct analogue of BlackBox targeted repaint, adapted to a split-thread
host: the CP thread decides what changed, emits a targeted command/invalidation, and
the UI thread executes the redraw locally.

Rust shim additions:

| Shim | Action |
|---|---|
| `WinFrame.PostPaneMsg` | Forward pane-scoped messages into host-owned channels |
| `WinFrame.PollPaneMsg` | Drain pane-scoped messages on the owning side |

Additional frame command queues may be layered on top of the same principle without
changing the ownership model.

---

### 2 — Observable model convention (`WinDoc`)

BlackBox's `Models.Broadcast(model, msg)` notifies all views in the same domain
when the model changes.  The wingui equivalent does not need a full domain/store
object graph.  A lightweight pure-CP module suffices:

```
MODULE WinDoc;

(* A document identity is a short string id, agreed between model and views. *)

TYPE Observer = PROCEDURE (docId, kind, detail: ARRAY OF SHORTCHAR);
  (* kind:   what changed — e.g. "text", "selection", "scroll"
     detail: optional hint — e.g. "12,45" for a line range, "" for full redraw *)

PROCEDURE AddObserver*(docId: ARRAY OF SHORTCHAR; p: Observer);
  (* Register p to be called whenever Notify(docId, kind, detail) is called.
     Multiple panes can observe the same document. *)

PROCEDURE RemoveObserver*(docId: ARRAY OF SHORTCHAR; p: Observer);

PROCEDURE Notify*(docId, kind, detail: ARRAY OF SHORTCHAR);
  (* Called by model modules after mutation.  Calls all registered observers
     synchronously on the calling thread. *)

END WinDoc.
```

A model module (e.g. `TextDoc`, `DataModel`, `Log`) holds its `docId` string and
calls `WinDoc.Notify` after each mutation.  Frame renderers (or slow-path render
procs) call `WinDoc.AddObserver` at startup.  Pure CP — no new Rust required.

BlackBox analogy:

| BlackBox | WinDoc |
|---|---|
| `Models.Model` with domain | module VAR with docId string |
| `Models.Broadcast(model, UpdateMsg{beg,end,delta})` | `WinDoc.Notify(docId, "text", "12,45")` |
| `Views.View.HandleModelMsg(msg)` | `Observer` procedure registered per pane |
| Multiple views of same model | Multiple observers for same docId |

---

### 3 — Per-frame pane inbox (cross-thread channel)

**The thread problem.**  `WinDoc.Notify` is synchronous — it is called on the
CP background (event) thread.  The frame executor runs on the D3D11 main thread
inside `on_frame`.  A direct call from `Notify` into CP code on the UI thread is
forbidden, and a direct call from `Notify` into D3D11-owned state would race.

BlackBox avoids this because everything (model broadcast, view repaint, controller)
runs on the single UI thread.  wingui separates them by design.  To be faithful
to the BlackBox model — where a model change triggers a targeted repaint of only
the affected region — we need a **lock-free cross-thread channel** from the CP
event thread to the frame thread.

**Design: pane inbox queue in wingui.dll**

```
┌────────────────────────────────────────────────────────────────┐
│ CP event thread                  D3D11 / frame thread          │
│                                                                │
│  WinDoc.Notify                    on_frame fires               │
│    → WinDoc observers called        → UI thread drains         │
│      → WinFrame.PostPaneMsg           pane / frame commands    │
│          (writes to inbox)            and executes redraw       │
│                                      → may emit status /       │
│                                        response messages        │
└────────────────────────────────────────────────────────────────┘
```

New CP procedures backed by Rust shims over a MPSC ring buffer in `wingui.dll`:

```
(* Post a message to a pane's inbox from any thread (typically the CP event thread).
   kind and detail are short identifying strings — not JSON.
   Returns 1 on success, 0 if inbox is full (drop and repaint conservatively). *)
PROCEDURE PostPaneMsg*(paneId: INTEGER;
                       kind, detail: ARRAY OF SHORTCHAR): INTSHORT;

(* Drain the next message from paneId's inbox.
  Valid only inside the host-side frame drain / pane execution step.
   Returns 1 if a message was dequeued, 0 if inbox is empty. *)
PROCEDURE PollPaneMsg*(paneId: INTEGER;
                       VAR kind:   ARRAY OF SHORTCHAR;
                       VAR detail: ARRAY OF SHORTCHAR): INTSHORT;
```

`WinDoc.Notify` becomes:

```
PROCEDURE Notify*(docId, kind, detail: ARRAY OF SHORTCHAR);
  (* 1. Call any slow-path observers (WinView.Render etc.) registered for docId.
     2. For each frame-path pane registered for docId, call PostPaneMsg. *)
```

The `Observer` registered by a frame-path pane simply calls `PostPaneMsg`:

```
PROCEDURE OnTextChanged*(docId, kind, detail: ARRAY OF SHORTCHAR);
  VAR dummy: INTSHORT;
BEGIN
  dummy := WinFrame.PostPaneMsg(TextEditor.pane, kind, detail)
END OnTextChanged;
```

On the UI side, the host drain step interprets the message and redraws the dirty
region.  If the operation needs to report something back to CP, it emits a normal
response/event message rather than calling into language code directly.

```
(* UI thread pseudocode *)
loop:
  if poll_pane_msg(paneId, kind, detail) = 0 then break;
  if kind = "text" then redraw_line_range(paneId, detail);
  if kind = "cursor" then redraw_cursor_cell(paneId, detail);
  if needs_reply then emit_ui_event("frame-status", ...);
```

This is the wingui equivalent of BlackBox's targeted repaint: the `UpdateMsg`
carries `{beg, end, delta}` so a text view repaints only the affected lines;
here `detail` carries the same hint as a plain string, and the pane renderer
decides how much to redraw.

**C++ / wingui.dll requirements**

| Component | What to add |
|---|---|
| `spec_bind.h` | `wingui_spec_bind_post_pane_msg(pane_id, kind, payload)` — no frame_view; callable from any thread |
| `spec_bind.h` | `wingui_spec_bind_frame_poll_pane_msg(frame_view, pane_id, kind_buf, payload_buf) -> i32` — frame-thread only |
| `spec_bind.cpp` | One MPSC ring buffer per pane (fixed-size, e.g. 64 entries); allocated when pane is resolved; freed when pane is destroyed |
| Rust shims | `WinFrame.PostPaneMsg` → `wingui_spec_bind_post_pane_msg` |
| Rust shims | `WinFrame.PollPaneMsg` → `wingui_spec_bind_frame_poll_pane_msg` |

The ring buffer is the only C++ addition.  It is lock-free (atomic head/tail),
bounded, and allocated per pane at resolve time — no heap traffic per message.

---

### How the three pieces fit together

```
  Model module (CP event thread)
  ──────────────────────────────
  TextDoc.Insert(pos, text)
    WinDoc.Notify("doc1", "text", "12,15")  ← broadcast

  WinDoc dispatch table
  ─────────────────────
    slow-path observer → WinView.Render (re-publishes JSON spec; for textarea etc.)
    frame-path observer → WinFrame.PostPaneMsg(editorPane, "text", "12,15")

  frame thread — on_frame
  ───────────────────────
    host drains pane/frame command queues
      PollPaneMsg → ("text", "12,15")
      → TextGridWriteCell for rows 12-15 only
      → no other panes touched
      → optional status/event back to CP thread
```

BlackBox analogy summary:

| BlackBox | wingui equivalent |
|---|---|
| `Models.Broadcast(model, msg)` | `WinDoc.Notify(docId, kind, detail)` |
| `Views.View.HandleModelMsg(msg)` | `WinDoc.Observer` proc |
| `Views.UpdateIn(v, l,t,r,b, rebuild)` | `WinFrame.PostPaneMsg(paneId, kind, detail)` |
| `Views.View.Restore(f, l,t,r,b)` | host-side frame executor drains pane/frame commands |
| `Models.Domain` (document boundary) | `docId` string (agreed by model + views) |
| Single UI thread — no channel needed | CP thread + D3D11 thread → `PostPaneMsg`/`PollPaneMsg` ring buffer |

The only structural divergence from BlackBox is the ring buffer channel, and
that is forced by the deliberate thread separation (CP thread isolated from D3D11
main thread).  Everything else maps directly.

---

## Updated summary of Rust / C++ work required

Add to the previous table:

| Shim / component | Action |
|---|---|
| `WinFrame.PostPaneMsg` | **Add** Rust shim → `wingui_spec_bind_post_pane_msg` |
| `WinFrame.PollPaneMsg` | **Add** Rust shim → `wingui_spec_bind_frame_poll_pane_msg` |
| `wingui_spec_bind_post_pane_msg` | **Add** to `spec_bind.h` / `spec_bind.cpp` |
| `wingui_spec_bind_frame_poll_pane_msg` | **Add** to `spec_bind.h` / `spec_bind.cpp` |
| Per-pane MPSC ring buffer | **Add** in `spec_bind.cpp` — allocated at pane resolve, lock-free |
| Frame command queue | **Add** host-owned queue for explicit draw/update commands from CP to UI |
| Frame status/event queue | **Add** optional host→CP reply channel for completions, hit-tests, visibility, etc. |
| `WinDoc` | Pure CP module — no Rust needed |
