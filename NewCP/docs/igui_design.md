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

The full primitive set is enumerated in [Surface primitives — full
reference](#surface-primitives--full-reference) below. Summary:

| Family | Commands |
|---|---|
| Lifecycle | `Clear`, `PresentHint` |
| Composition | `PushClipRect`, `PopClipRect`, `PushOffset`, `PopOffset`, `ScrollRect`, `InstallChildViewBounds` |
| Geometry — fills | `FillRect`, `FillOval`, `FillCircle` |
| Geometry — strokes | `StrokeRect`, `StrokeOval`, `StrokeCircle`, `DrawLine`, `DrawArc` |
| Geometry — paths | `DrawPath` |
| Text | `DrawTextRun`, `MeasureTextRun`, `CharIndexAtPoint`, `PointAtCharIndex` |
| Overlays | `MarkRect`, `Caret`, `SelectionRange`, `FocusRing` |

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

## Surface primitives — full reference

These are the primitives the iGui surface executor must implement on top
of Direct2D and DirectWrite. The list is **closed**: nothing here can be
skipped, and no other primitive may be added without amending this doc.

The set is mined from the archived [display_primitives](archive/display_primitives.md),
[surface_design](archive/surface_design.md), [surface_tracker](archive/surface_tracker.md),
and the proven `WinBatch` signatures in [Mod/WinBatch.cp](../Mod/WinBatch.cp).
Several legacy parameters are intentionally dropped:

- `bufMode` (Frame vs. Persistent buffer) — iGui surfaces are always
  persistent retained buffers managed by the executor.
- `clearBefore` + per-call clear color — replaced by an explicit `Clear`
  at batch start.
- `blendMode` — alpha-over is the only mode; opaque drawing is achieved
  by a fully-opaque color.

### Coordinate space and units

- **Pixel space** — coordinates are DIPs (device-independent pixels)
  measured from the top-left of the child window's client area. The
  Direct2D device context maps DIPs to physical pixels using system DPI;
  iGui does no scaling of its own.
- **Color** — linear-RGBA `f32` in `[0.0, 1.0]`. The Direct2D render
  target is BGRA8; conversion happens at draw time.
- **Angles** — radians, measured clockwise from the positive X axis.
- **Half-thickness** — stroke commands take `half_thickness`, not full
  thickness, because Direct2D's stroke is centered on the path. Half-
  thickness `t` produces a stroke of width `2t` extending `t` on each
  side of the path centerline.
- **Corner radius** — `0.0` means a sharp rectangle; non-zero values
  produce Direct2D rounded rectangles with that radius on both axes.

### Composition state

The surface executor maintains three pieces of per-batch state, all
reset to identity at `Begin`:

1. **Clip stack** — LIFO of axis-aligned rects. `PushClipRect` calls
   `PushAxisAlignedClip`; `PopClipRect` calls `PopAxisAlignedClip`.
   Mismatched push/pop aborts the batch with a logged diagnostic.
2. **Offset stack** — LIFO of translation transforms. `PushOffset`
   multiplies a translation onto the current device-context transform;
   `PopOffset` restores. Translations only — arbitrary affine
   transforms are out of scope.
3. **Retained child-view bounds** — `child_id → rect` map, set by
   `InstallChildViewBounds`. Survives across batches until the child
   window closes. Used by future nested-view composition.

### Rust enum

```rust
pub enum SurfaceCmd {
    // ─── Lifecycle ─────────────────────────────────────────────────
    Clear            { color: Rgba },
    PresentHint,

    // ─── Composition ───────────────────────────────────────────────
    PushClipRect     { rect: Rect },
    PopClipRect,
    PushOffset       { dx: f32, dy: f32 },
    PopOffset,
    ScrollRect       { rect: Rect, dx: f32, dy: f32 },
    InstallChildViewBounds { child_id: u32, rect: Rect },

    // ─── Geometry — fills ──────────────────────────────────────────
    FillRect         { rect: Rect, corner_radius: f32, color: Rgba },
    FillOval         { rect: Rect, color: Rgba },
    FillCircle       { center: Point, radius: f32, color: Rgba },

    // ─── Geometry — strokes ────────────────────────────────────────
    StrokeRect       { rect: Rect, corner_radius: f32,
                       half_thickness: f32, color: Rgba },
    StrokeOval       { rect: Rect, half_thickness: f32, color: Rgba },
    StrokeCircle     { center: Point, radius: f32,
                       half_thickness: f32, color: Rgba },
    DrawLine         { p0: Point, p1: Point,
                       half_thickness: f32, color: Rgba },
    DrawArc          { center: Point, radius: f32,
                       rotation_rad: f32, half_aperture_rad: f32,
                       half_thickness: f32, color: Rgba },

    // ─── Geometry — paths ──────────────────────────────────────────
    DrawPath         { commands: Vec<PathCmd>,
                       fill: Option<Rgba>,
                       stroke: Option<StrokeStyle> },

    // ─── Text ──────────────────────────────────────────────────────
    DrawTextRun      { run: TextRun },
    MeasureTextRun   { request_id: u32, run: TextRun },
    CharIndexAtPoint { request_id: u32, run: TextRun, point: Point },
    PointAtCharIndex { request_id: u32, run: TextRun, char_index: u32 },

    // ─── Overlays ──────────────────────────────────────────────────
    MarkRect         { rect: Rect, mode: MarkMode },
    Caret            { rect: Rect, color: Rgba },
    SelectionRange   { rect: Rect, color: Rgba },
    FocusRing        { rect: Rect, corner_radius: f32,
                       half_thickness: f32, color: Rgba },
}

pub struct Rect  { pub x0: f32, pub y0: f32, pub x1: f32, pub y1: f32 }
pub struct Point { pub x: f32, pub y: f32 }
pub struct Rgba  { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

pub enum MarkMode { Highlight, Invert, Dim }

pub enum PathCmd {
    MoveTo  (Point),
    LineTo  (Point),
    QuadTo  { ctrl: Point, end: Point },
    CubicTo { c1: Point, c2: Point, end: Point },
    ArcTo   { radius: Point, rotation_rad: f32,
              large_arc: bool, sweep: bool, end: Point },
    Close,
}

pub struct StrokeStyle {
    pub half_thickness: f32,
    pub line_cap:       LineCap,
    pub line_join:      LineJoin,
    pub miter_limit:    f32,
    pub dash_pattern:   Option<Vec<f32>>,
}
pub enum LineCap  { Flat, Round, Square }
pub enum LineJoin { Miter, Round, Bevel }

pub struct TextRun {
    pub text:      String,
    pub origin:    Point,
    pub family:    String,
    pub size:      f32,                // DIPs
    pub weight:    u16,                // 100..900 (DWRITE_FONT_WEIGHT)
    pub style:     FontStyle,
    pub stretch:   FontStretch,
    pub locale:    String,             // BCP-47, e.g. "en-us"
    pub color:     Rgba,
    pub max_width: Option<f32>,        // None = no wrap
    pub alignment: TextAlign,
    pub trimming:  TextTrimming,
}
pub enum FontStyle    { Normal, Italic, Oblique }
pub enum FontStretch  { UltraCondensed, ExtraCondensed, Condensed,
                        SemiCondensed, Normal, SemiExpanded, Expanded,
                        ExtraExpanded, UltraExpanded }
pub enum TextAlign    { Leading, Trailing, Center, Justified }
pub enum TextTrimming { None, EllipsisChar, EllipsisWord }
```

### Per-command reference

#### Lifecycle

**`Clear { color }`** — fill the entire pane buffer with `color`.
`ID2D1DeviceContext::Clear`. Clip and offset stacks are unaffected.

**`PresentHint`** — request the GUI thread to `Present()` after the
current batch finishes draining. Sets a per-pane "present pending"
flag; `WM_PAINT` honors it. Without this hint the GUI thread may
coalesce presents across batches.

#### Composition

**`PushClipRect { rect }`** — push axis-aligned clip.
`PushAxisAlignedClip(rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE)`.
Subsequent draw commands are clipped to the intersection of all
currently pushed rects.

**`PopClipRect`** — pop one entry. `PopAxisAlignedClip()`. Mismatch
with a prior `PushClipRect` aborts the batch.

**`PushOffset { dx, dy }`** — translate the coordinate space by
`(dx, dy)`. Multiplies `Matrix3x2F::Translation(dx, dy)` onto the
current transform via `SetTransform`. Clip rects pushed *before* an
offset are not translated by it; they were captured in the coordinate
space active at push time.

**`PopOffset`** — restore the transform from before the matching
`PushOffset`.

**`ScrollRect { rect, dx, dy }`** — copy the contents of `rect` to
`rect + (dx, dy)` within the same pane buffer. Source rect is left
undefined; the caller is expected to redraw it. Implementation:
intra-buffer GPU copy; falls back to a temporary bitmap when source
and destination overlap and copy ordering cannot be safe.

**`InstallChildViewBounds { child_id, rect }`** — record the rect at
which a logical child view of CP-side composition will be drawn.
Inserts into the per-pane retained child-bounds map; does not draw
anything yet.

#### Geometry — fills

**`FillRect { rect, corner_radius, color }`** — `FillRectangle` if
`corner_radius == 0.0`, else `FillRoundedRectangle`. Brush is a
cached solid-color brush keyed on `color`.

**`FillOval { rect, color }`** — filled axis-aligned ellipse with the
given bounding box. `FillEllipse` with center `rect.center`,
`radiusX = rect.width/2`, `radiusY = rect.height/2`.

**`FillCircle { center, radius, color }`** — `FillEllipse` with equal
radii. Provided as its own command for clarity at the call site.

#### Geometry — strokes

**`StrokeRect { rect, corner_radius, half_thickness, color }`** —
`DrawRectangle` / `DrawRoundedRectangle` with `strokeWidth =
2.0 * half_thickness`.

**`StrokeOval { rect, half_thickness, color }`** — `DrawEllipse`.

**`StrokeCircle { center, radius, half_thickness, color }`** —
`DrawEllipse` with equal radii.

**`DrawLine { p0, p1, half_thickness, color }`** — `DrawLine`,
default flat caps. For other caps use `DrawPath` with an explicit
`StrokeStyle`.

**`DrawArc { center, radius, rotation_rad, half_aperture_rad,
half_thickness, color }`** — circular arc spanning
`[rotation_rad - half_aperture_rad, rotation_rad + half_aperture_rad]`.
Builds a transient `ID2D1PathGeometry` with `BeginFigure` /
`AddArc` / `EndFigure(OPEN)`, then `DrawGeometry`.

#### Geometry — paths

**`DrawPath { commands, fill, stroke }`** — arbitrary path with
optional fill and optional stroke. Builds an `ID2D1PathGeometry`
from `commands`. If `fill` is set, `FillGeometry`; if `stroke` is
set, build (or look up) an `ID2D1StrokeStyle1` for the cap/join/dash
settings and `DrawGeometry`. Fill happens before stroke when both
are present.

`PathCmd` → `ID2D1GeometrySink` mapping:

| `PathCmd` | Sink call |
|---|---|
| `MoveTo` | `BeginFigure(point, FILLED)` |
| `LineTo` | `AddLine` |
| `QuadTo` | `AddQuadraticBezier` |
| `CubicTo` | `AddBezier` |
| `ArcTo` | `AddArc` with computed `D2D1_ARC_SEGMENT` |
| `Close` | `EndFigure(CLOSED)` |

A path that opens a figure with `MoveTo` and never closes it ends
with `EndFigure(OPEN)`.

#### Text

All four text commands resolve `TextRun` against the same
DirectWrite layout. The executor:

1. Looks up or creates an `IDWriteTextFormat` keyed on
   `(family, size, weight, style, stretch, locale, alignment)`.
2. Looks up or creates an `IDWriteTextLayout` keyed on
   `(text, format, max_width)`.
3. Reuses that layout for whichever of the four operations is
   requested.

This is the only acceptable text path. There is no glyph-atlas
fallback, no separate measurement engine, no estimated metrics.
Draw, measure, and hit-test are answered by the same layout object.

**`DrawTextRun { run }`** — `ID2D1DeviceContext::DrawTextLayout(run.origin, layout, brush)`.

**`MeasureTextRun { request_id, run }`** —
`IDWriteTextLayout::GetMetrics`. The reply is sent as a
`surface-reply` event with `request_id`, carrying
`(width, height, ascent, line_count)`.

**`CharIndexAtPoint { request_id, run, point }`** —
`IDWriteTextLayout::HitTestPoint`. Reply carries
`(char_index, is_inside, is_trailing_hit)`.

**`PointAtCharIndex { request_id, run, char_index }`** —
`IDWriteTextLayout::HitTestTextPosition`. Reply carries
`(x, y, height)`.

The synchronous-query commands sit in normal batches alongside
drawing commands; the GUI thread executes them in batch order and
emits replies into the event mailbox after each query is answered.

#### Overlays

**`MarkRect { rect, mode }`** — view feedback over a rect:
- `Highlight` — fill with system selection color at ~30% alpha.
- `Invert` — XOR composition with white. `D2D1_COMPOSITE_MODE_XOR`
  `FillRectangle` with a white brush.
- `Dim` — fill with system window color at ~50% alpha.

**`Caret { rect, color }`** — `FillRectangle(rect, brush)`.
`rect.x1 - rect.x0` is caret thickness (typically 1 DIP).

**`SelectionRange { rect, color }`** — one rectangle of a multi-rect
text selection. Producers emit one command per visual line.
`FillRectangle(rect, brush)`, `color.a` typically ~0.3.

**`FocusRing { rect, corner_radius, half_thickness, color }`** —
focused-control outline. Same implementation as `StrokeRect`.
Provided as its own command so the platform can swap in a system
focus visual style later without touching call sites.

### Implementation map

| `SurfaceCmd` | Direct2D / DirectWrite call(s) |
|---|---|
| `Clear` | `Clear(color)` |
| `PresentHint` | sets per-pane present-pending flag |
| `PushClipRect` | `PushAxisAlignedClip` |
| `PopClipRect` | `PopAxisAlignedClip` |
| `PushOffset` | `SetTransform(prev * Translation)` |
| `PopOffset` | `SetTransform(prev)` |
| `ScrollRect` | intra-buffer GPU copy / temporary bitmap fallback |
| `InstallChildViewBounds` | host-state map insert |
| `FillRect` | `FillRectangle` / `FillRoundedRectangle` |
| `FillOval` | `FillEllipse` |
| `FillCircle` | `FillEllipse` |
| `StrokeRect` | `DrawRectangle` / `DrawRoundedRectangle` |
| `StrokeOval` | `DrawEllipse` |
| `StrokeCircle` | `DrawEllipse` |
| `DrawLine` | `DrawLine` |
| `DrawArc` | `ID2D1PathGeometry` + `AddArc` + `DrawGeometry` |
| `DrawPath` | `ID2D1PathGeometry` from `PathCmd[]` + optional `FillGeometry` / `DrawGeometry` |
| `DrawTextRun` | `IDWriteTextLayout` + `DrawTextLayout` |
| `MeasureTextRun` | `IDWriteTextLayout::GetMetrics` |
| `CharIndexAtPoint` | `IDWriteTextLayout::HitTestPoint` |
| `PointAtCharIndex` | `IDWriteTextLayout::HitTestTextPosition` |
| `MarkRect` | `FillRectangle` with mode-specific brush / composite |
| `Caret` | `FillRectangle` |
| `SelectionRange` | `FillRectangle` |
| `FocusRing` | `DrawRoundedRectangle` |

### Cached host resources

The executor keeps three caches keyed by hash of their inputs:

1. **Solid-color brush cache** — `Rgba → ID2D1SolidColorBrush`. Hot
   path; nearly every command hits it.
2. **Stroke-style cache** — `(LineCap, LineJoin, miter_limit,
   dash_pattern) → ID2D1StrokeStyle1`. Misses are rare in steady
   state.
3. **Text format / layout cache** — owned by the font manager; keyed
   on the `IDWriteTextFormat` and `IDWriteTextLayout` descriptors
   above.

Cache eviction is LRU with generous limits (1024 brushes, 64 stroke
styles, 256 layouts). A future `iGui.FlushCaches` may be exposed if
memory pressure ever becomes a concern.

### CP-side `Emit*` correspondence

Every `SurfaceCmd` variant has exactly one CP-side `iGui.Emit*`
procedure with the same name minus the `Emit` prefix and the same
parameter list expanded to scalars. The exact signatures live in the
`iGui` DEFINITION module ([CP-side surface](#cp-side-surface--the-igui-module)).
The closed enum here is the authoritative source; the CP signatures
follow from it.

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

  (* batches: build, then submit. One Emit* per SurfaceCmd variant. *)
  PROCEDURE BeginBatch* (childId: INTEGER);
  PROCEDURE SubmitBatch*(): INTSHORT;            (* enqueue current batch, return *)

  (* lifecycle *)
  PROCEDURE EmitClear*       (r, g, b, a: REAL);
  PROCEDURE EmitPresentHint* ;

  (* composition *)
  PROCEDURE EmitPushClipRect* (x0, y0, x1, y1: REAL);
  PROCEDURE EmitPopClipRect*  ;
  PROCEDURE EmitPushOffset*   (dx, dy: REAL);
  PROCEDURE EmitPopOffset*    ;
  PROCEDURE EmitScrollRect*   (x0, y0, x1, y1, dx, dy: REAL);
  PROCEDURE EmitInstallChildViewBounds* (childId: INTSHORT;
                                         x0, y0, x1, y1: REAL);

  (* geometry — fills *)
  PROCEDURE EmitFillRect*    (x0, y0, x1, y1, cornerRadius: REAL;
                              r, g, b, a: REAL);
  PROCEDURE EmitFillOval*    (x0, y0, x1, y1: REAL;
                              r, g, b, a: REAL);
  PROCEDURE EmitFillCircle*  (cx, cy, radius: REAL;
                              r, g, b, a: REAL);

  (* geometry — strokes *)
  PROCEDURE EmitStrokeRect*  (x0, y0, x1, y1, cornerRadius, halfThickness: REAL;
                              r, g, b, a: REAL);
  PROCEDURE EmitStrokeOval*  (x0, y0, x1, y1, halfThickness: REAL;
                              r, g, b, a: REAL);
  PROCEDURE EmitStrokeCircle*(cx, cy, radius, halfThickness: REAL;
                              r, g, b, a: REAL);
  PROCEDURE EmitDrawLine*    (x0, y0, x1, y1, halfThickness: REAL;
                              r, g, b, a: REAL);
  PROCEDURE EmitDrawArc*     (cx, cy, radius,
                              rotationRad, halfApertureRad,
                              halfThickness: REAL;
                              r, g, b, a: REAL);

  (* geometry — paths
     pathCmds: tagged stream of MoveTo / LineTo / QuadTo / CubicTo / ArcTo / Close
               packed as a flat REAL/INTSHORT array; layout fixed in iGui.cp.
     fillMode  / strokeMode: 0 = absent, 1 = present *)
  PROCEDURE EmitDrawPath*    (VAR pathBytes: ARRAY OF SHORTCHAR;
                              pathLen: INTEGER;
                              fillMode: INTSHORT;
                              fillR, fillG, fillB, fillA: REAL;
                              strokeMode: INTSHORT;
                              strokeHalfThick, strokeMiter: REAL;
                              strokeCap, strokeJoin: INTSHORT;
                              VAR strokeDash: ARRAY OF REAL;
                              strokeDashLen: INTSHORT;
                              strokeR, strokeG, strokeB, strokeA: REAL);

  (* text *)
  PROCEDURE EmitDrawTextRun* (text:    ARRAY OF SHORTCHAR;
                              originX, originY, size: REAL;
                              family:  ARRAY OF SHORTCHAR;
                              weight:  INTSHORT;
                              style:   INTSHORT;
                              stretch: INTSHORT;
                              locale:  ARRAY OF SHORTCHAR;
                              maxWidth: REAL;       (* < 0 = no wrap *)
                              alignment, trimming: INTSHORT;
                              r, g, b, a: REAL);

  (* overlays *)
  PROCEDURE EmitMarkRect*     (x0, y0, x1, y1: REAL; mode: INTSHORT);
  PROCEDURE EmitCaret*        (x0, y0, x1, y1: REAL; r, g, b, a: REAL);
  PROCEDURE EmitSelectionRange*(x0, y0, x1, y1: REAL; r, g, b, a: REAL);
  PROCEDURE EmitFocusRing*    (x0, y0, x1, y1, cornerRadius, halfThickness: REAL;
                               r, g, b, a: REAL);

  (* synchronous queries — submit, block, return reply
     Each generates a SurfaceCmd::Measure*/Char*/Point* in the batch with a
     fresh requestId, then blocks the language thread on the matching
     surface-reply event. *)
  PROCEDURE MeasureTextRun*   (childId: INTEGER;
                               text:    ARRAY OF SHORTCHAR;
                               size:    REAL;
                               family:  ARRAY OF SHORTCHAR;
                               weight, style, stretch: INTSHORT;
                               locale:  ARRAY OF SHORTCHAR;
                               maxWidth: REAL;
                               VAR width, height, ascent: REAL;
                               VAR lineCount: INTEGER): INTSHORT;
  PROCEDURE CharIndexAtPoint* (childId: INTEGER;
                               text:    ARRAY OF SHORTCHAR;
                               size:    REAL;
                               family:  ARRAY OF SHORTCHAR;
                               weight, style: INTSHORT;
                               x, y:    REAL;
                               VAR charIndex: INTEGER;
                               VAR isInside, isTrailing: INTSHORT): INTSHORT;
  PROCEDURE PointAtCharIndex* (childId:   INTEGER;
                               text:      ARRAY OF SHORTCHAR;
                               size:      REAL;
                               family:    ARRAY OF SHORTCHAR;
                               weight, style: INTSHORT;
                               charIndex: INTEGER;
                               VAR x, y, height: REAL): INTSHORT;

END iGui.
```

There is intentionally no separate `Console`, `Log`, `WinPayload`, or
`HostFrame` module. Logging is done by drawing into a designated child;
event payloads are typed in the event struct, not in JSON.

The `Emit*` set is **closed**: it has exactly one procedure per
`SurfaceCmd` variant in [the Rust enum above](#rust-enum). Adding a
new primitive means adding both a `SurfaceCmd` variant and the matching
`Emit*` procedure in lockstep; one without the other is broken.

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

### Phase 3 — Child windows + geometry primitives

1. Add MDI child window class, `iGui.OpenChild`, `iGui.CloseChild`,
   `iGui.SetTitle`.
2. Allocate one SPSC pane batch ring per child.
3. Implement the surface executor: drain the pending batch on
   `WM_PAINT`, dispatch `SurfaceCmd` variants.
4. Implement the **lifecycle** group: `Clear`, `PresentHint`.
5. Implement the **geometry — fills** group: `FillRect`, `FillOval`,
   `FillCircle`.
6. Implement the **geometry — strokes** group: `StrokeRect`,
   `StrokeOval`, `StrokeCircle`, `DrawLine`, `DrawArc`.
7. Wire the brush cache and stroke-style cache.
8. Add `iGui.BeginBatch` / matching `iGui.Emit*` procedures /
   `iGui.SubmitBatch`.

Acceptance: a CP module opens two children and paints a recognisable
geometric scene into each (rects, ovals, circles, arcs, lines), end
to end, with no panic and no leaks across child close.

### Phase 4 — Text via DirectWrite

1. Wire the font manager: format cache, family/weight/style/stretch
   resolution, layout cache.
2. Implement `DrawTextRun`.
3. Implement the synchronous `MeasureTextRun`, `CharIndexAtPoint`,
   `PointAtCharIndex` round-trips via the `surface-reply` event with
   the 5-second internal timeout described above.

Acceptance: a CP module renders proportional text and round-trips a
hit-test that matches caret geometry.

### Phase 5 — Composition, overlays, paths

1. Implement the **composition** group: `PushClipRect`, `PopClipRect`,
   `PushOffset`, `PopOffset`, `ScrollRect`, `InstallChildViewBounds`.
2. Implement the **overlays** group: `MarkRect` (Highlight / Invert /
   Dim), `Caret`, `SelectionRange`, `FocusRing`.
3. Implement the **geometry — paths** group: `DrawPath` with the full
   `PathCmd` enum and `StrokeStyle`. Wire the `ID2D1StrokeStyle1`
   cache.
4. Verify mismatched clip / offset push/pop aborts the batch with a
   structured diagnostic.

Acceptance: a CP-side text view exercises selection, caret blinking,
clipped scrolling, and a non-trivial `DrawPath` (e.g. a rounded badge
with cubic curves) without flicker. **At end of Phase 5 every variant
in the closed `SurfaceCmd` enum is implemented in Rust.**

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
