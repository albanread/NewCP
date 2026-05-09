# wingui MVC Update — Implementation Plan & Tracker

Tracks the changes needed in `wingui.dll` (C++), `newcp-runtime` (Rust shims),
and the CP module stubs to support the per-pane MVC design described in
`MVC_summary.md` and `cp_wingui.md`.

---

## Overview of changes

| Layer | Component | What changes |
|---|---|---|
| C++ — `spec_bind.h` | Header | Add two new function declarations: `post_pane_msg`, `frame_poll_pane_msg` |
| C++ — `spec_bind.cpp` | Implementation | Add `PaneInbox` SPSC ring buffer; add per-pane inbox map to runtime; implement two new functions |
| Rust — `wingui_ffi.rs` | Types | Add `SuperTerminalPaneLayout` struct |
| Rust — `wingui_spec_ffi.rs` | FFI bindings | Add frame-time function declarations + new pane inbox declarations |
| Rust — `wingui_host.rs` | Shims | Add full `WinFrame.*` shim module; update `on_frame` dispatcher; register `WinFrame` module artifact |

---

## Task tracker

### Task 1 — `spec_bind.h`: pane inbox declarations

**File**: `multiwingui/include/wingui/spec_bind.h`

**Status**: [x] done

Add at the end of the extern "C" block, before the closing `}`:

```c
// ---------------------------------------------------------------------------
// Pane inbox — per-pane SPSC ring buffer (CP event thread → D3D11 frame thread)
// ---------------------------------------------------------------------------

/// Post a (kind, detail) message to pane pane_id's inbox.
/// Safe to call from any thread (typically the CP event thread).
/// Returns 1 on success, 0 if the inbox is full (message dropped).
WINGUI_API int32_t WINGUI_CALL wingui_spec_bind_post_pane_msg(
    WinguiSpecBindRuntime* runtime,
    SuperTerminalPaneId pane_id,
    const char* kind_utf8,
    const char* detail_utf8);

/// Drain one message from pane pane_id's inbox.
/// Valid only during the host frame drain on the frame thread.
/// Returns 1 if a message was dequeued into kind_out/detail_out, 0 if empty.
WINGUI_API int32_t WINGUI_CALL wingui_spec_bind_frame_poll_pane_msg(
    const WinguiSpecBindFrameView* frame_view,
    SuperTerminalPaneId pane_id,
    char* kind_out,
    uint32_t kind_cap,
    char* detail_out,
    uint32_t detail_cap);
```

---

### Task 2 — `spec_bind.cpp`: PaneInbox ring buffer + implementations

**File**: `multiwingui/src/spec_bind.cpp`

**Status**: [x] done

1. Add `#include <atomic>` and `#include <unordered_map>` to includes.
2. Add `PaneMsg` and `PaneInbox` structs inside the anonymous namespace.
3. Add `pane_inbox_mutex` + `pane_inboxes` map to `WinguiSpecBindRuntime`.
4. Implement `wingui_spec_bind_post_pane_msg`.
5. Implement `wingui_spec_bind_frame_poll_pane_msg`.

Design: SPSC lock-free ring buffer with 64 slots of (kind:32, detail:128) bytes.
Map key is `pane_id.value` (uint64). Inbox is allocated lazily on first `post`.

---

### Task 3 — `wingui_ffi.rs`: add `SuperTerminalPaneLayout`

**File**: `NewCP/src/newcp-runtime/src/wingui_ffi.rs`

**Status**: [x] done

Add:
```rust
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct SuperTerminalPaneLayout {
    pub x:           i32,
    pub y:           i32,
    pub width:       i32,
    pub height:      i32,
    pub visible:     i32,
    pub columns:     u32,
    pub rows:        u32,
    pub cell_width:  f32,
    pub cell_height: f32,
}
```

---

### Task 4 — `wingui_spec_ffi.rs`: frame FFI + pane inbox FFI

**File**: `NewCP/src/newcp-runtime/src/wingui_spec_ffi.rs`

**Status**: [x] done

Add declarations for:
- `wingui_spec_bind_frame_index` → `u64`
- `wingui_spec_bind_frame_elapsed_ms` → `u64`
- `wingui_spec_bind_frame_delta_ms` → `u64`
- `wingui_spec_bind_frame_bind_pane` → `i32`
- `wingui_spec_bind_frame_get_pane_layout` → `i32`
- `wingui_spec_bind_frame_request_present` → `i32`
- `wingui_spec_bind_post_pane_msg` → `i32`  (new)
- `wingui_spec_bind_frame_poll_pane_msg` → `i32`  (new)

---

### Task 5 — `wingui_host.rs`: WinFrame shims + updated on_frame

**File**: `NewCP/src/newcp-runtime/src/wingui_host.rs`

**Status**: [x] done

Changes:
1. Add thread-local `FRAME_VIEW: RefCell<*const WinguiSpecBindFrameView>` — valid during host frame drain only.
2. Replace no-op `on_frame` with a host-side drain that exposes frame state to `WinFrame.*` shims.
3. Remove callback-era `WinFrame.SetRenderer` / pane-renderer exports from the host and CP stub.
6. Add all `WinFrame.*` `#[unsafe(export_name = ...)]` shim functions.
7. Add `winframe_module_artifact()` function.
8. Register WinFrame artifact in the module artifact table.

WinFrame exports:
- `FrameIndex() -> i64`
- `ElapsedMs() -> i64`
- `DeltaMs() -> i64`
- `ResolvePaneId(node_id_ptr: *const u8, pane_id_ptr: *mut i64) -> i32`
- `PaneLayout(pane_id: i64, x: *mut i32, y: *mut i32, w: *mut i32, h: *mut i32) -> i32`
- `RequestPresent()`
- `PostPaneMsg(pane_id: i64, kind_ptr: *const u8, detail_ptr: *const u8) -> i32`
- `PollPaneMsg(pane_id: i64, kind_ptr: *mut u8, detail_ptr: *mut u8) -> i32`

---

### Task 6 — Build verification

**Status**: [x] done

- `cargo build --features gui --bin newcp-driver` from `NewCP/`: done.
- Build the wingui.dll solution and confirm the new functions link: done.

---

### Task 7 — typed pane render batch format

**Files**: design first in `cp_wingui.md` / `MVC_summary.md`, then code in `wingui_host.rs` and CP helper modules

**Status**: [~] partial

Define the first concrete fast-path batch format so pane rendering no longer relies
only on ad hoc `kind/detail` strings.

Minimum target:

1. Batch envelope with pane id, sequence number, and flags.
2. Fixed command vocabulary for clip rects, text-grid updates, text runs,
   vector primitives, sprite batches, and caret / selection overlays.
3. Clear ownership rule: CP builds the batch, UI drains and executes it.
4. One root view produces one pane batch, even if many micro-views contributed.

Current prototype implemented:

- New `WinBatch` host module and CP stub.
- Batch envelope with `paneId`, `sequence`, and `flags`.
- Typed commands implemented end-to-end for text-grid cells, geometry, overlays,
  clips, offsets, scrolling, and text-path placeholders.
- Host-side queue drained during `on_frame`, executing through existing `HostFrame.*` helpers.
- Per-pane pending-batch replacement: newer sequence numbers supersede stale work for the same pane.
- New declarative `surface` pane kind scaffolded through `WinSpec.AddSurface` so general MVC views can target a distinct pane type before specialized high-speed panes arrive.

Still missing for this task:

- batch consumption from real CP views,
- sprite batches and richer asset/image commands,
- explicit response / completion semantics for submitted batches.

---

### Task 8 — DirectWrite surface text engine

**Files**: `multiwingui/include/wingui/wingui.h`, `multiwingui/src/wingui.cpp`, `multiwingui/include/wingui/spec_bind.h`, `multiwingui/src/spec_bind.cpp`, `NewCP/docs/display_primitives.md`, `NewCP/docs/cp_wingui.md`

**Status**: [~] partial

Focused design and execution tracking for the next phase now live in:

- `NewCP/docs/surface_design.md`
- `NewCP/docs/surface_tracker.md`

`surface` text must be separated cleanly from `text-grid` text.

This is part of a broader pane split that should remain explicit in the design:

- `surface` is the richer, purpose-built target for general MVC rendering.
- `text-grid`, `rgba-pane`, `indexed-graphics`, and similar fast panes remain
    specialized rendering paths for workloads that need tighter performance-focused
    contracts.
- those specialized panes may later be exposed to MVC through wrapper views or
    embedded pane abstractions, but they are not substitutes for the general
    `surface` primitive model.

Requirements:

1. Keep `text-grid` as the specialized monospaced glyph-cell path for very fast
    code-editor and terminal workloads.
2. Extend `surface` text to use DirectWrite-backed layout and rendering.
3. Define `DrawTextRun`, `MeasureTextRun`, `CharIndexAtPoint`, and
    `PointAtCharIndex` in terms of the same host layout objects.
4. Support arbitrary font families, size, weight, style, stretch, fallback,
    and proportional positioning.
5. Ensure selection and caret geometry comes from the same layout engine used
    for drawing.

Expected host additions:

- DirectWrite text layout creation for `surface` runs.
- Rendering path that can draw DirectWrite-shaped runs into a `surface` pane.
- Replacement of the current atlas-based `surface` text approximation that is
    still sitting underneath the present `DrawTextRun` / measurement / hit-test exports.

---

### Task 9 — wingui font manager

**Files**: likely new font-manager implementation in `multiwingui`, with public API additions in `wingui.h` and `spec_bind.h`

**Status**: [ ] not started

`wingui` should own a reusable font manager component.

Requirements:

1. Load and cache font families / faces / styles independently of any one pane.
2. Expose reusable metrics for fonts and text runs.
3. Measure text runs without requiring a specific `surface` instance to own the
    font data.
4. Support retained layout caching for document-oriented MVC panes.
5. Provide the measurement and hit-test foundation needed by `surface`
    controller overlays and document layout.

---

## Ring buffer design

```
PaneMsg { kind: [u8; 32], detail: [u8; 128] }

PaneInbox {
    write_pos: AtomicU32  // producer advances
    read_pos:  AtomicU32  // consumer advances
    slots: [PaneMsg; 64]
}

post(kind, detail):
    w = write_pos.load(Relaxed)
    r = read_pos.load(Acquire)
    if w - r >= 64: return false  // full, drop
    slots[w % 64] = { kind, detail }
    write_pos.store(w + 1, Release)
    return true

poll(kind_out, detail_out):
    r = read_pos.load(Relaxed)
    w = write_pos.load(Acquire)
    if r == w: return false  // empty
    *kind_out = slots[r % 64].kind
    *detail_out = slots[r % 64].detail
    read_pos.store(r + 1, Release)
    return true
```

The SPSC property holds because:
- CP event thread is serialised (one handler at a time via `WaitNamedEvent`)
- D3D11 frame thread is the only consumer (single consumer)

---

## Pane inbox lifecycle

1. **Inbox created**: lazily on first `post_pane_msg` call for a pane_id.
   Protected by `inbox_map_mutex` for map insertion only.
2. **Inbox used**: after insertion, pointer is stable (stored in `unique_ptr`);
   post/poll use only the atomic ring buffer — no mutex contention per message.
3. **Inbox destroyed**: on `wingui_spec_bind_runtime_destroy` (runtime owns the map).
   No explicit per-pane destroy needed; if a pane is removed from the spec,
   unread messages are simply discarded when the runtime is destroyed.

---

## Thread safety summary

| Operation | Thread | Synchronization |
|---|---|---|
| `post_pane_msg` | CP event thread | `inbox_map_mutex` for inbox creation; atomic write for message post |
| `frame_poll_pane_msg` | D3D11 frame thread | `inbox_map_mutex` for inbox lookup; atomic read for message poll |
| host frame drain | D3D11 frame thread | frame-thread only after `FRAME_VIEW` is installed |
| `FRAME_VIEW` thread-local | D3D11 frame thread only | Thread-local, no sharing |
| `runtime_ptr()` / `RUNTIME` | Any | `OnceLock` (write-once, then read-only) |
