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

**Status**: [ ] not started | [ ] done

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
/// Valid only inside a frame callback (frame thread).
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

**Status**: [ ] not started | [ ] done

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

**Status**: [ ] not started | [ ] done

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

**Status**: [ ] not started | [ ] done

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

**Status**: [ ] not started | [ ] done

Changes:
1. Add `use std::sync::atomic::{AtomicUsize, Ordering}`.
2. Add `FRAME_RENDERER: AtomicUsize` — stores global frame proc ptr.
3. Add `PANE_RENDERERS: OnceLock<Mutex<Vec<PaneRendererEntry>>>` — per-pane table.
4. Add thread-local `FRAME_VIEW: RefCell<*const WinguiSpecBindFrameView>` — valid during `on_frame` only.
5. Replace no-op `on_frame` with dispatcher that: stores frame_view, calls global renderer, iterates pane table.
6. Add all `WinFrame.*` `#[unsafe(export_name = ...)]` shim functions.
7. Add `winframe_module_artifact()` function.
8. Register WinFrame artifact in the module artifact table.

WinFrame exports:
- `SetRenderer(fn_ptr: usize)`
- `RegisterPaneRenderer(pane_id: i64, fn_ptr: usize)`
- `UnregisterPaneRenderer(pane_id: i64)`
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

Run `cargo build --features gui --bin newcp-driver` from `NewCP/` and confirm zero errors.
Then build the wingui.dll solution and confirm the new functions link.

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
| `on_frame` dispatcher | D3D11 frame thread | `PANE_RENDERERS` mutex for renderer table snapshot |
| `RegisterPaneRenderer` | CP startup thread | `PANE_RENDERERS` mutex |
| `FRAME_VIEW` thread-local | D3D11 frame thread only | Thread-local, no sharing |
| `runtime_ptr()` / `RUNTIME` | Any | `OnceLock` (write-once, then read-only) |
