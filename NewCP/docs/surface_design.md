# Surface Design

## Purpose

This document defines the focused implementation design for the NewCP `surface`
pane.

`surface` is the general-purpose rendering target for BlackBox-style MVC work.
It is the pane used for ordinary document views, proportional text, mixed-style
layout, controller overlays, precision geometry, and nested view composition.

This is a correctness-first renderer.

- `surface` is not `text-grid` with extra commands.
- `surface` is not `rgba-pane` with a pile of software drawing helpers.
- `surface` is not `indexed-graphics` with a more complicated protocol.
- `surface` may later embed or wrap specialized panes, but it is not defined in
  terms of them.

The implementation target is a host-native Direct2D plus DirectWrite renderer,
integrated into the existing D3D11-based wingui runtime.

---

## Design Goals

1. Support the `display_primitives.md` contract for general MVC rendering.
2. Render rich typographic text with DirectWrite-backed shaping, metrics,
   hit-testing, selection, and caret geometry.
3. Render geometry with high precision and explicit control over fills, strokes,
   clips, transforms, and composition.
4. Keep CP/JIT execution off the UI thread.
5. Preserve the architectural split between `surface` and the specialized
   high-speed panes.
6. Provide a reusable font manager that is independent of any one pane instance.
7. Keep the fast-pane renderers available for workloads that want narrower,
   performance-oriented contracts.

---

## Non-Goals

1. Replacing `text-grid`, `rgba-pane`, or `indexed-graphics`.
2. Faking `surface` by routing its commands through the existing fast panes.
3. Forcing all graphics in wingui through Direct2D.
4. Exposing Win32, D2D, or DWrite object pointers directly to CP code.
5. Solving all future retained-layout and document-model problems in the first
   implementation slice.

---

## Core Architectural Decision

`surface` should be implemented as a distinct rendering stack with three layers:

1. A process-wide text and font service.
2. A context-level Direct2D and DirectWrite bridge layered onto the D3D11 device.
3. A per-pane `surface` executor that drains typed batches and renders them into
   pane-owned buffers.

The current shader-based vector path remains useful for specialized high-speed
panes and possibly for a few narrow `surface` optimizations later, but it is not
the primary rendering model for general MVC work.

---

## Rendering Stack

### 1. D3D11 and DXGI Foundation

The current wingui renderer already owns the D3D11 device, device context,
swap chain, and RGBA pane buffers.  `surface` should continue to sit inside that
host environment.

Required foundation changes:

1. Create the D3D11 device with BGRA support so Direct2D can interoperate with
   DXGI-backed textures.
2. Use BGRA-compatible target formats for any pane buffers that Direct2D must
   draw into.
3. Expose or internalize the DXGI surfaces needed to create Direct2D bitmap
   targets for `surface` buffers.

The goal is not to replace D3D11.  The goal is to let D3D11 remain the owning GPU
environment while Direct2D and DirectWrite become the precision drawing layer for
`surface`.

### 2. Direct2D Integration

Each wingui context should own Direct2D objects bound to the same graphics device:

1. `ID2D1Factory`
2. `ID2D1Device`
3. `ID2D1DeviceContext`

The device context is used by the `surface` executor to:

1. clear pane buffers
2. push and pop clips
3. apply translation transforms
4. draw fills and strokes
5. draw text layouts
6. render selection and caret overlays

Direct2D is the right place to implement:

1. `FillRect`
2. `StrokeRect`
3. `DrawLine`
4. `FillOval`
5. `StrokeOval`
6. `DrawArc`
7. `DrawPath`
8. clip and transform stack behavior

### 3. DirectWrite Integration

The text side should be built on DirectWrite, not on the glyph atlas used by
`text-grid`.

Each wingui process should own a DirectWrite factory and the `surface` text path
should use real DirectWrite layout objects for:

1. shaping and font fallback
2. measurement
3. hit-testing
4. caret geometry
5. selection geometry
6. final rendering

This means the following exports must converge on the same underlying text engine:

1. `DrawTextRun`
2. `MeasureTextRun`
3. `CharIndexAtPoint`
4. `PointAtCharIndex`

No approximation split is allowed between draw, measure, and hit-test.

---

## Font Manager Design

## Purpose

The font manager is a host-owned service for `surface` text.

It must not be tied to any one pane instance or one frame.  It provides reusable
font and layout services for all `surface` panes in a process.

## Responsibilities

1. Resolve font family, weight, style, stretch, size, locale, and feature requests.
2. Load system fonts and optional application-provided fonts.
3. Cache text formats and font-face resolution results.
4. Create and cache reusable text layouts when layout reuse is profitable.
5. Provide text metrics for lines and runs.
6. Provide hit-testing and caret placement.
7. Provide fallback behavior for missing glyphs and mixed-script text.

## Suggested Internal Split

1. `WinguiFontManager`
   Process-wide owner of DirectWrite factory access, font collections, custom font
   registration, and reusable format caches.

2. `WinguiTextFormatKey`
   Immutable descriptor for family, weight, style, stretch, size, locale, and
   rendering-affecting text options.

3. `WinguiTextLayoutKey`
   Descriptor for shaped text layout reuse.  Includes the text payload, format,
   width constraints, alignment options, and layout-affecting flags.

4. `WinguiTextLayoutHandle`
   Host-side cached layout object that can answer draw, measure, hit-test, and
   caret/selection queries consistently.

## Cache Policy

The first implementation should keep caching simple:

1. cache text formats aggressively
2. cache short-lived text layouts opportunistically
3. invalidate layout cache entries by key rather than by mutable object updates
4. keep retained document-layout caching as a later optimization layer

Correctness matters more than aggressive caching in the first slice.

---

## Surface Pane Object Model

Each `surface` pane should have host-owned resources roughly like this:

1. pane buffer resources
2. a Direct2D render target view of the active buffer
3. clip stack
4. transform stack
5. optional retained child-view bounds table
6. optional surface-local resource cache for brushes, stroke styles, and path data

Suggested host-side structure names:

1. `WinguiSurfacePaneResources`
2. `WinguiSurfaceRenderer`
3. `WinguiSurfaceCommandExecutor`

The executor is responsible for consuming `WinBatch` / `SurfaceBatch` commands
on the UI thread and translating them into Direct2D and DirectWrite operations.

---

## Primitive Mapping

### Geometry

| Primitive | Primary implementation |
|---|---|
| `FillRect` | Direct2D filled rounded rectangle or rectangle |
| `StrokeRect` | Direct2D stroked rounded rectangle or rectangle |
| `DrawLine` | Direct2D line draw with explicit stroke style |
| `FillOval` | Direct2D filled ellipse |
| `StrokeOval` | Direct2D stroked ellipse |
| `DrawArc` | Direct2D path geometry with arc segment |
| `DrawPath` | Direct2D path geometry built from explicit path commands |

`DrawPath` should evolve beyond the current polyline approximation.  The design
target is a real path command stream with move, line, bezier, arc, close, fill
mode, and stroke options.

### Text

| Primitive | Primary implementation |
|---|---|
| `DrawTextRun` | DirectWrite text layout draw through Direct2D |
| `MeasureTextRun` | DirectWrite layout metrics |
| `CharIndexAtPoint` | DirectWrite hit-test query |
| `PointAtCharIndex` | DirectWrite hit-test / caret query |

The public command name may remain `DrawTextRun`, but the payload must become a
real text run descriptor rather than just `(text, origin, color)`.

### View and Controller Feedback

| Primitive | Primary implementation |
|---|---|
| `MarkRect` | Direct2D fill or effect brush over a rect |
| `Caret` | Direct2D fill rect or caret path |
| `SelectionRange` | selection geometry from text layout, rendered with Direct2D |
| `FocusRing` | Direct2D stroked rounded rectangle or custom path |

### Composition

| Primitive | Primary implementation |
|---|---|
| `PushClipRect` | Direct2D axis-aligned clip push |
| `PopClipRect` | Direct2D clip pop |
| `PushOffset` | transform stack push with translation |
| `PopOffset` | transform stack pop |
| `ScrollRect` | GPU copy when valid; redraw fallback when necessary |

`ScrollRect` should remain a composition optimization.  It does not define the
text or geometry renderer.

---

## Command Model Evolution

The current typed batch bridge is useful, but the `surface` command payloads need
to become richer.

## Immediate Compatibility Layer

The first Direct2D and DirectWrite implementation can keep the current command
names and replace their internals underneath the existing host exports.

This is the lowest-risk path for:

1. `DrawTextRun`
2. `MeasureTextRun`
3. `CharIndexAtPoint`
4. `PointAtCharIndex`

## Next-Step Payload Improvements

After the initial renderer is live, evolve the payloads toward explicit style and
path descriptions.

### Text command expansion

Add host-side descriptors for:

1. font family
2. size
3. weight
4. style
5. stretch
6. locale
7. brush or RGBA color
8. optional width constraints
9. alignment and wrapping mode
10. optional feature flags

### Path command expansion

Replace the current point-list `DrawPath` approximation with a real path stream:

1. move-to
2. line-to
3. quadratic bezier
4. cubic bezier
5. arc segment
6. close path
7. fill mode
8. stroke style

---

## Surface Execution Model

The CP side continues to own semantics.

1. models mutate on the CP thread
2. views flatten display output into `surface` batches on the CP thread
3. the UI thread drains those batches
4. the `surface` executor renders host-natively with Direct2D and DirectWrite

This preserves the MVC split while giving the UI thread the rendering precision
it needs.

---

## Quality Bar

The first acceptable `surface` implementation must satisfy all of the following:

1. proportional text rendering is real, not cell-based
2. text measurement matches text drawing
3. hit-testing matches text drawing
4. caret and selection geometry come from the same layout engine
5. geometry is not approximated through the wrong pane type
6. path rendering supports real clipping and transforms
7. the implementation does not depend on executing CP code on the UI thread

If a proposed implementation does not meet that bar, it is not a real `surface`
implementation.

---

## Recommended Implementation Order

1. D3D11 and DXGI interop changes for Direct2D compatibility
2. process-wide font manager and DirectWrite factory integration
3. context-level Direct2D device and device-context integration
4. `surface` pane resource type and render-target binding
5. replace `DrawTextRun` / `MeasureTextRun` / hit-test exports with DirectWrite
6. replace core geometry commands with Direct2D implementations
7. expand path command model beyond polyline approximation
8. add brush, stroke-style, and layout caching optimizations

---

## Explicit Anti-Goals for Future Drift

1. Do not implement `surface` by aliasing it to `text-grid`.
2. Do not implement `surface` by aliasing it to `rgba-pane`.
3. Do not implement `surface` by routing all drawing through the current shader
   vector renderer and calling that finished.
4. Do not let the convenience of existing panes lower the quality bar for the
   general MVC renderer.
