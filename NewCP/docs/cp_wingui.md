# CP WinGui — Greenfield Design

## Goals

- CP modules never see or construct JSON.  That is purely the Rust layer's concern.
- The declarative builder (`WinSpec`) is stateless: call it, get a spec, done.
- A small framework (`WinView`, `WinLoop`) provides an MVC event backbone so any
  module can register handlers for the events it cares about without a monolithic
  switch statement.
- The whole stack runs on a background CP thread; the Rust main thread drives the
  D3D11 message loop independently.
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
│  WinView   — owns spec buffer, render callback, publish  │
│  WinLoop   — blocking event loop, handler dispatch table │
│  WinFrame  — per-frame callback, pane resolution         │
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
| `WinFrame.SetRenderer` | **Add** — store CP frame proc pointer; call from Rust `on_frame` |
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
graphs, or games.  The host runtime fires a dedicated **frame callback** at
`target_frame_ms` (typically 16 ms, ~60 fps) on the D3D11 main thread.  Inside
that callback CP can write pixels, text cells, sprites, and vector shapes directly
into any `canvas` widget pane — bypassing the JSON diff engine entirely.

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
    └─ on_frame() ──────────────────────→ WinFrame.FrameProc called
                                            HostFrame.* draw calls run
                                            (frame_view stored thread-locally
                                             in Rust for duration of call)
```

The D3D11 thread calls CP's frame procedure directly via the stored function
pointer.  CP must **return quickly** — no blocking, no `WaitNamedEvent`, no slow
allocations.  All drawing is fire-and-forget; ownership of pixel buffers is
transferred to the host.

### Widget types for the frame path

| Widget spec type | Declared via | Backing surface | Per-frame draw APIs |
|---|---|---|---|
| `text-grid` | `WinSpec.AddTextGrid` | Hardware-accelerated monospace grid (cols × rows cells); each cell stores a UTF-32 codepoint, foreground colour, and background colour; the host uploads the cell buffer to the GPU and renders it via a glyph-atlas shader every frame | `TextGridWriteCell`, `TextGridClearRegion` |
| `canvas` | `WinSpec.AddCanvas` (TBD) | RGBA or indexed pixel surface | RGBA upload, indexed upload, sprites, vector draw |

`text-grid` is the correct surface for a CP code editor or terminal.  It is
entirely separate from `textarea` (a Windows RichEdit control) and from `canvas`
(a pixel surface).  The cell buffer lives in GPU-visible memory; calling
`TextGridWriteCell` queues a single-cell update that the host applies in the same
frame — no JSON diff, no Windows message, no GDI.

`textarea` is a **Windows RichEdit control** on the slow JSON spec path.  It is
not a pane and cannot be resolved for per-frame drawing.

Both `text-grid` and `canvas` panes are resolved at startup with
`WinFrame.ResolvePaneId(nodeId, paneId)` — store the returned `paneId` and reuse
it every frame.

### Content buffer modes

| Mode | CP constant | Behaviour |
|---|---|---|
| `FRAME` | `WinFrame.BufFrame = 0` | Scratch buffer — host clears it before each frame.  Safe to draw from scratch every call. |
| `PERSISTENT` | `WinFrame.BufPersistent = 1` | Retained between frames.  Only re-draw the region you changed.  Best for editors and graphs that update partially. |

---

## WinFrame — frame callback dispatch

Pure CP module.  Registers a frame render procedure and exposes frame-level
query helpers.  Backed by Rust shims that store the current `frame_view` pointer
thread-locally for the duration of the frame callback.

```
MODULE WinFrame;

TYPE
  FrameProc = PROCEDURE;

CONST
  BufFrame      = 0;   (* SUPERTERMINAL_RGBA_CONTENT_BUFFER_FRAME *)
  BufPersistent = 1;   (* SUPERTERMINAL_RGBA_CONTENT_BUFFER_PERSISTENT *)

PROCEDURE SetRenderer*(p: FrameProc);
  (* Register p as the per-frame render procedure.
     Rust stores the function pointer; on_frame calls it directly.
     Call once before WinLoop.Run. *)

(* Frame timing — valid only inside the frame procedure *)
PROCEDURE FrameIndex*(): INTEGER;
PROCEDURE ElapsedMs*(): INTEGER;    (* ms since runtime start *)
PROCEDURE DeltaMs*(): INTEGER;      (* ms since previous frame *)

(* Pane resolution — call at startup, not inside the frame procedure *)
PROCEDURE ResolvePaneId*(nodeId: ARRAY OF SHORTCHAR; VAR paneId: INTEGER): INTSHORT;
  (* Resolves the widget node id (e.g. "myCanvas") to an opaque pane integer.
     Returns 1 on success, 0 if not yet laid out.  Call after WinView.Render.
     Store paneId as a module VAR and pass it to HostFrame.* each frame. *)

PROCEDURE PaneLayout*(paneId: INTEGER;
                      VAR x, y, width, height: INTSHORT): INTSHORT;
  (* Fill pixel rect and visible flag of a previously resolved pane.
     Returns 1 if layout is valid this frame, 0 if the pane is hidden. *)

PROCEDURE RequestPresent*;
  (* Force a Present this frame even if auto_request_present is off.
     Usually not needed; call only when skipping draw but still need to flush. *)

END WinFrame.
```

### Compiler note: pane IDs

`SuperTerminalPaneId` is a `uint64_t` in C.  CP `INTEGER` is `i64` — the same
width.  Pass `VAR paneId: INTEGER` and the Rust shim writes the raw 64-bit value.
Use CP `INTEGER` throughout for pane IDs.

---

## HostFrame — frame-time drawing primitives

CP definition module backed by Rust shims.  All procedures are valid only inside
a `WinFrame.FrameProc` (the Rust shim asserts a non-null frame_view).
Coordinates are in pixels from the top-left of the pane.  Colours are passed as
four separate `REAL` values (0.0–1.0 each: r, g, b, a).

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
In the frame procedure, call `TextGridWriteCell` for each changed cell;
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

PROCEDURE OnFrame;
  VAR dummy: INTSHORT;
BEGIN
  IF ready = 0 THEN RETURN END;
  (* write only cells that changed since last frame *)
  dummy := HostFrame.TextGridWriteCell(pane, 0, 0, ORD("H"), 0FFFFFFH, 0);
  dummy := HostFrame.TextGridWriteCell(pane, 0, 1, ORD("i"), 0FFFFFFH, 0)
END OnFrame;

END Editor.
```

Register in App.Run:
```
WinFrame.SetRenderer(Editor.OnFrame);
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

## MVC extension — per-pane renderers, observable models, and frame inbox

The global `WinFrame.SetRenderer` and `WinView.Render` are correct for simple
single-view applications.  To support document-style MVC — where multiple
independent panes each observe their own model — three additions are needed.

### 1 — Per-pane frame renderers in `WinFrame`

Replace the single global frame proc with a per-pane registration table.  The
global `SetRenderer` remains for backwards compatibility (it registers against
pane 0, the full-window fallback).

```
TYPE PaneProc = PROCEDURE (paneId: INTEGER);

PROCEDURE RegisterPaneRenderer*(paneId: INTEGER; p: PaneProc);
  (* Register p as the per-frame draw procedure for paneId.
     During on_frame the Rust dispatcher iterates the table and calls each
     PaneProc in registration order, passing the pane's own ID.
     A module can register for multiple panes; one pane can have one renderer. *)

PROCEDURE UnregisterPaneRenderer*(paneId: INTEGER);
  (* Remove the renderer for paneId — e.g. when a document is closed. *)
```

This is the direct analogue of BlackBox `Views.View.Restore(f, l,t,r,b)`:
each view (pane) owns its own repaint procedure.  The Rust `on_frame` shim
loops over the registry exactly as BlackBox's frame manager loops over the
frame tree to call `Restore` on each dirty view.

Rust shim additions:

| Shim | Action |
|---|---|
| `WinFrame.RegisterPaneRenderer` | Store `(pane_id, fn_ptr)` in a `Vec` in Rust; `on_frame` iterates and calls each |
| `WinFrame.UnregisterPaneRenderer` | Remove entry by `pane_id` |

No changes to `wingui.dll` needed — the Rust layer already owns `on_frame`.

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
CP background (event) thread.  The frame renderers run on the D3D11 main thread
inside `on_frame`.  A direct call from `Notify` into a frame renderer would race
with D3D11.

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
│    → WinDoc observers called        → WinFrame dispatches      │
│      → WinFrame.PostPaneMsg           each PaneProc            │
│          (writes to inbox)              → PaneProc calls        │
│                                           WinFrame.PollPaneMsg  │
│                                           drains inbox          │
│                                           redraws dirty region  │
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
   Valid only inside a PaneProc (frame thread).
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

Then inside the `PaneProc`:

```
PROCEDURE OnFrame*(paneId: INTEGER);
  VAR kind, detail: ARRAY 64 OF SHORTCHAR;
      dummy: INTSHORT;
BEGIN
  LOOP
    IF WinFrame.PollPaneMsg(paneId, kind, detail) = 0 THEN EXIT END;
    IF StrEq(kind, "text") THEN
      (* detail = "firstLine,lastLine" — redraw only those rows *)
      RedrawLineRange(paneId, detail)
    ELSIF StrEq(kind, "cursor") THEN
      RedrawCursorCell(paneId, detail)
    END
  END
END OnFrame;
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
    WinFrame dispatches EditorPane.OnFrame(editorPane)
      PollPaneMsg → ("text", "12,15")
      → TextGridWriteCell for rows 12-15 only
      → no other panes touched
```

BlackBox analogy summary:

| BlackBox | wingui equivalent |
|---|---|
| `Models.Broadcast(model, msg)` | `WinDoc.Notify(docId, kind, detail)` |
| `Views.View.HandleModelMsg(msg)` | `WinDoc.Observer` proc |
| `Views.UpdateIn(v, l,t,r,b, rebuild)` | `WinFrame.PostPaneMsg(paneId, kind, detail)` |
| `Views.View.Restore(f, l,t,r,b)` | `PaneProc(paneId)` registered via `RegisterPaneRenderer` |
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
| `WinFrame.RegisterPaneRenderer` | **Add** — per-pane `(pane_id, fn_ptr)` table in Rust; iterated in `on_frame` |
| `WinFrame.UnregisterPaneRenderer` | **Add** |
| `WinFrame.PostPaneMsg` | **Add** Rust shim → `wingui_spec_bind_post_pane_msg` |
| `WinFrame.PollPaneMsg` | **Add** Rust shim → `wingui_spec_bind_frame_poll_pane_msg` |
| `wingui_spec_bind_post_pane_msg` | **Add** to `spec_bind.h` / `spec_bind.cpp` |
| `wingui_spec_bind_frame_poll_pane_msg` | **Add** to `spec_bind.h` / `spec_bind.cpp` |
| Per-pane MPSC ring buffer | **Add** in `spec_bind.cpp` — allocated at pane resolve, lock-free |
| `WinDoc` | Pure CP module — no Rust needed |
