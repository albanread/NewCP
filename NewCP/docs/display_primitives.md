# Display Primitives

This document defines the first general `surface` contract for NewCP GUI work.
The goal is not to collapse everything into `text-grid`, `rgba-pane`, or
`indexed-graphics`. The goal is to provide one MVC-oriented pane type that can
faithfully host BlackBox-style views and controllers on the CP thread while the
UI thread owns execution of the resulting display batches.

This distinction is architectural, not temporary.

- `surface` is the purpose-built general MVC rendering target.
- the existing high-speed panes remain separate specialized rendering substrates.
- those fast panes may later be wrapped by special-purpose MVC views when useful,
  but they are not the definition of general MVC rendering and they do not set
  the primitive contract for ordinary document views.

## Scope

`surface` is the default pane type for ordinary GUI views.

It is tuned for:

- view restore into a pane-owned frame
- document-style text with mixed fonts, styles, sizes, and positioned runs
- controller overlays such as caret, selection, hilite, invert, and drag marks
- nested micro-views flattened into one pane batch
- scroll offsets, clip rectangles, and embedded coordinate spaces
- incremental repaint driven by model/view invalidation rather than full-scene redraw assumptions

It is not the same as the future specialized high-speed panes:

- `text-grid` for dense cell-oriented text
- `rgba-pane` for direct RGBA uploads and graphics pipelines
- `indexed-graphics` for palette/index workflows
- future editing, drawing, and game panes with tighter throughput contracts

Those panes are expected to remain valuable in their own right for fast graphics,
games, monospaced editing, and other throughput-driven workloads outside the
general MVC design.  If MVC later wants to host them, it should do so by wrapping
them as specialized views or embedded surfaces, not by redefining `surface` to be
one of those fast paths.

`text-grid` and `surface` are intentionally different text systems.

- `text-grid` is the specialized fast path for monospaced code editors,
  terminals, and other cell-addressed workloads.
- `surface` is the general MVC document pane.  Its text model is not cell-based.
  It must support arbitrary fonts, style changes inside a run, proportional
  advances, precise positioning, and host-side hit-testing.

## BlackBox-Oriented Primitive Set

The compatibility target comes from the abstract frame contract used by BlackBox
views rather than from raw Win32 drawing calls.

### Geometry

- `FillRect`
- `StrokeRect`
- `DrawLine`
- `FillOval`
- `StrokeOval`
- `DrawArc`
- `DrawPath`

### Text

- `DrawTextRun`
- `MeasureTextRun`
- `CharIndexAtPoint`
- `PointAtCharIndex`

These text primitives are defined in terms of a full host text engine, not a
glyph-cell atlas.

For `surface`, the host text engine must be DirectWrite-backed and must support:

- arbitrary font families, sizes, weights, slant, and stretch
- font fallback and Unicode shaping
- proportional positioning and mixed-style runs
- measurement from real advances and line metrics
- hit-testing from layout objects rather than cell coordinates
- caret and selection geometry derived from the same layout used for drawing

The authoritative implementation model is:

- CP thread: owns document semantics, style runs, and text command generation
- UI thread: owns DirectWrite text layout creation, measurement, hit-testing,
  and final rendering into the `surface`

`DrawTextRun`, `MeasureTextRun`, `CharIndexAtPoint`, and `PointAtCharIndex`
must all be resolved against the same host text layout semantics.  They are not
allowed to diverge into separate approximations.

### Font Manager

`wingui` needs a font manager component for `surface` text.

The font manager is a host-owned service that:

- loads and caches font faces and text formats
- resolves requested family/weight/style/stretch tuples
- exposes reusable font metrics independent of any one pane instance
- measures text runs and line metrics
- supports text layout object caching for retained MVC panes

The font manager is not the same as the `text-grid` glyph atlas.  The glyph
atlas remains a specialized optimization for the `text-grid` pane.

### View and Controller Feedback

- `MarkRect` with modes such as normal highlight, invert, and dim
- `Caret`
- `SelectionRange`
- `FocusRing`

### Composition

- `PushClipRect`
- `PopClipRect`
- `PushOffset`
- `PopOffset`
- `ScrollRect`

### Lifecycle

- `Clear`
- `PresentHint`
- `InstallChildViewBounds`

## Batch Message Format

The `surface` pane receives typed batches from the CP thread.

### Envelope

```text
SurfaceBatch {
    paneId: u64
    sequence: u64
    flags: u32
    commands: [SurfaceCmd]
}
```

Rules:

- `paneId` selects the target pane.
- `sequence` is monotonic per pane.
- newer batches replace stale pending batches for the same pane.
- the UI thread drains and executes batches during frame processing.

### Command Families

```text
SurfaceCmd =
    Clear
  | PushClipRect
  | PopClipRect
  | PushOffset
  | PopOffset
  | FillRect
  | StrokeRect
  | DrawLine
  | FillOval
  | StrokeOval
  | DrawArc
  | DrawPath
  | DrawTextRun
  | MeasureTextRun
  | CharIndexAtPoint
  | PointAtCharIndex
  | MarkRect
  | Caret
  | SelectionRange
  | ScrollRect
```

### Current Implementation Slice

The current codebase already has a typed `WinBatch` bridge and a host-side drain
for the first `surface` command set.

Implemented end-to-end today:

1. batch envelope with `paneId`, `sequence`, and `flags`
2. composition commands: `PushClipRect`, `PopClipRect`, `PushOffset`, `PopOffset`
3. geometry commands: `FillRect`, `StrokeRect`, `DrawLine`, `FillOval`,
   `StrokeOval`, `DrawArc`, `DrawPath`
4. feedback / overlay commands: `MarkRect`, `Caret`, `SelectionRange`, `FocusRing`
5. lifecycle / composition helpers: `Clear`, `ScrollRect`, `PresentHint`,
  `InstallChildViewBounds` with host-owned retained child-bounds state
6. text-path commands: `DrawTextRun`, `MeasureTextRun`,
   `CharIndexAtPoint`, and `PointAtCharIndex`

The remaining gaps are now higher-level renderer concerns rather than basic
command routing. `surface` text already runs through DirectWrite-backed draw,
measure, and hit-test exports, but the reusable host font manager and the final
child-view composition semantics still need to be completed.

## Architectural Rule

Models, controllers, and logical views stay on the CP thread.

The UI thread never runs CP view code directly. It only executes `surface`
batches that were already computed by the CP-side MVC world.