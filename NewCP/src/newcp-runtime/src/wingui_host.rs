/// wingui_host.rs
///
/// Safe Rust wrappers around wingui's spec_bind APIs.
///
/// Primary entry point: `SpecBindRuntime` - owns `WinguiSpecBindRuntime*`.
/// JIT-visible shims ("HostWindows.*", "WinSpec.*", "WinFrame.*") are registered via
/// `native_module_artifact()` / `winspec_module_artifact()` / `winframe_module_artifact()`.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString, c_void};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, OnceLock, PoisonError};

use serde_json::Value;

// ---------------------------------------------------------------------------
// Structured tracing
// ---------------------------------------------------------------------------
//
// All surface/MVC pipeline events are tagged `[wingui-trace]` so they can be
// grepped or filtered out wholesale. This is intentionally always-on during
// development; set `NEWCP_WINGUI_TRACE=0` to silence.
//
// Format convention: key=value pairs, space separated, after a `kind=...`
// classifier. The kind names are stable enough that downstream tooling (or a
// human running `findstr`) can rely on them:
//
//   submit, submit_dropped_stale,
//   drain_begin, drain_batch, drain_end,
//   on_frame, publish_ui, bind_fail,
//   reset_composition, present_request
//
// Per-frame chatter is gated separately by NEWCP_WINGUI_TRACE_FRAMES so the
// always-on mode stays useful without 60Hz spam.

fn trace_enabled() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(|| match std::env::var("NEWCP_WINGUI_TRACE") {
        Ok(v) => !matches!(v.trim(), "0" | "false" | "off" | ""),
        Err(_) => true,
    })
}

fn trace_frames_enabled() -> bool {
    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(|| match std::env::var("NEWCP_WINGUI_TRACE_FRAMES") {
        Ok(v) => matches!(v.trim(), "1" | "true" | "on"),
        Err(_) => false,
    })
}

macro_rules! wingui_trace {
    ($($arg:tt)*) => {
        if trace_enabled() {
            eprintln!("[wingui-trace] {}", format_args!($($arg)*));
        }
    };
}

/// Track the most recent `on_frame` index so other trace points can reference
/// it without plumbing the index through every call.
static LAST_FRAME_INDEX: AtomicU64 = AtomicU64::new(0);
/// Bumped each time `host_publish_ui` runs, so we can correlate spec
/// republish events with surface presenter recreations on the C++ side.
static PUBLISH_UI_COUNT: AtomicU64 = AtomicU64::new(0);
/// Set true once we've logged the first on_frame so we can spam-trace the
/// startup frames specifically.
static FIRST_FRAME_LOGGED: AtomicBool = AtomicBool::new(false);

use crate::wingui_ffi::SuperTerminalRunResult;
use crate::wingui_spec_ffi::{
    WinguiSpecBindEventView, WinguiSpecBindFrameView, WinguiSpecBindRunDesc, WinguiSpecBindRuntime,
    WinguiSpecBindPaneRef,
};
use crate::{
    ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact,
};

// ---------------------------------------------------------------------------
// Event queue
// ---------------------------------------------------------------------------

struct GuiEvent {
    name:    String,
    payload: String,
}

struct EventQueue {
    queue: Mutex<VecDeque<GuiEvent>>,
    ready: Condvar,
}

static EVENT_QUEUE: EventQueue = EventQueue {
    queue: Mutex::new(VecDeque::new()),
    ready: Condvar::new(),
};

/// Recover from mutex poisoning by logging once and returning the inner guard.
///
/// The wingui host runs both producer (UI thread `on_event`) and consumer
/// (CP worker thread `host_wait_named_event`) under the same process. A
/// poisoned mutex means one of those threads panicked while holding the
/// lock — propagating with `expect` would tear down the surviving thread
/// too, which is the worst outcome for a render path. Recovering the
/// inner value lets the system limp on; callers that cannot tolerate a
/// torn queue should still inspect contents themselves.
fn recover_lock<'a, T>(
    result: Result<MutexGuard<'a, T>, PoisonError<MutexGuard<'a, T>>>,
    site: &str,
) -> MutexGuard<'a, T> {
    match result {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("[wingui_host] mutex poisoned at {site}; recovering inner state");
            poisoned.into_inner()
        }
    }
}

unsafe extern "system" fn on_event(
    _user_data: *mut c_void,
    _runtime: *mut WinguiSpecBindRuntime,
    view: *const WinguiSpecBindEventView,
) {
    if view.is_null() {
        eprintln!("[wingui_event] on_event called with null view");
        return;
    }
    let v = unsafe { &*view };
    let name = if v.event_name_utf8.is_null() { String::new() }
               else { unsafe { CStr::from_ptr(v.event_name_utf8) }.to_string_lossy().into_owned() };
    let payload = if v.payload_json_utf8.is_null() { String::new() }
                  else { unsafe { CStr::from_ptr(v.payload_json_utf8) }.to_string_lossy().into_owned() };
    let source = if v.source_utf8.is_null() { String::new() }
                 else { unsafe { CStr::from_ptr(v.source_utf8) }.to_string_lossy().into_owned() };
    eprintln!("[wingui_event] name={:?} source={:?} payload={:?}", name, source, payload);
    let mut q = recover_lock(EVENT_QUEUE.queue.lock(), "EVENT_QUEUE on_event");
    q.push_back(GuiEvent { name, payload });
    EVENT_QUEUE.ready.notify_one();
}

// ---------------------------------------------------------------------------
// WinFrame — frame-state access for the host-side frame drain
// ---------------------------------------------------------------------------

// Thread-local pointer to the current WinguiSpecBindFrameView.
// Valid only for the duration of an on_frame call on the D3D11 main thread.
thread_local! {
    static FRAME_VIEW: RefCell<*const WinguiSpecBindFrameView> =
        RefCell::new(std::ptr::null());
}

#[derive(Debug, Clone)]
struct PaneRenderBatch {
    pane_id: u64,
    sequence: u64,
    flags: u32,
    commands: Vec<PaneRenderCommand>,
}

#[derive(Debug, Copy, Clone)]
struct SurfaceRect {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl SurfaceRect {
    fn normalize(self) -> Self {
        Self {
            x0: self.x0.min(self.x1),
            y0: self.y0.min(self.y1),
            x1: self.x0.max(self.x1),
            y1: self.y0.max(self.y1),
        }
    }

    fn translate(self, dx: f64, dy: f64) -> Self {
        Self {
            x0: self.x0 + dx,
            y0: self.y0 + dy,
            x1: self.x1 + dx,
            y1: self.y1 + dy,
        }
    }

    fn width(self) -> f64 {
        self.x1 - self.x0
    }

    fn height(self) -> f64 {
        self.y1 - self.y0
    }

    fn intersects(self, other: Self) -> bool {
        self.x0 < other.x1 && self.x1 > other.x0 && self.y0 < other.y1 && self.y1 > other.y0
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let rect = Self {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        };
        (rect.width() > 0.0 && rect.height() > 0.0).then_some(rect)
    }
}

#[derive(Debug, Clone)]
enum PaneRenderCommand {
    Clear {
        buf_mode: i32,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    PushClipRect {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    },
    PopClipRect,
    PushOffset {
        dx: f64,
        dy: f64,
    },
    PopOffset,
    TextCell {
        row: i32,
        column: i32,
        codepoint: i64,
        fg: i64,
        bg: i64,
    },
    DrawLine {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        half_thickness: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    DrawText {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        text: String,
        origin_x: f64,
        origin_y: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    FillRect {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        corner_radius: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    StrokeRect {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        half_thickness: f64,
        corner_radius: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    FillCircle {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        cx: f64,
        cy: f64,
        radius: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    FillOval {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    StrokeCircle {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        cx: f64,
        cy: f64,
        radius: f64,
        half_thickness: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    StrokeOval {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        half_thickness: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    DrawArc {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        cx: f64,
        cy: f64,
        radius: f64,
        half_thickness: f64,
        rotation_rad: f64,
        half_aperture_rad: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    DrawPath {
        buf_mode: i32,
        blend_mode: i32,
        clear_before: i32,
        clear_r: f64,
        clear_g: f64,
        clear_b: f64,
        clear_a: f64,
        points_xy: Vec<f32>,
        closed: i32,
        half_thickness: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    MarkRect {
        mode: i32,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    },
    Caret {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    SelectionRange {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    FocusRing {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        half_thickness: f64,
        corner_radius: f64,
        color_r: f64,
        color_g: f64,
        color_b: f64,
        color_a: f64,
    },
    ScrollRect {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        dx: f64,
        dy: f64,
    },
    PresentHint,
    InstallChildViewBounds {
        child_id: i32,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    },
}

thread_local! {
    static CURRENT_PANE_BATCH: RefCell<Option<PaneRenderBatch>> =
        const { RefCell::new(None) };
}

fn pending_pane_batches() -> &'static Mutex<HashMap<u64, PaneRenderBatch>> {
    static PENDING_PANE_BATCHES: OnceLock<Mutex<HashMap<u64, PaneRenderBatch>>> = OnceLock::new();
    PENDING_PANE_BATCHES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn bind_frame_pane_layout(fv_ptr: *const WinguiSpecBindFrameView, pane_id: u64) -> Option<crate::wingui_ffi::SuperTerminalPaneLayout> {
    let pane_ref = bind_frame_pane(fv_ptr, pane_id as i64)?;
    let mut layout = crate::wingui_ffi::SuperTerminalPaneLayout {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        visible: 0,
        columns: 0,
        rows: 0,
        cell_width: 0.0,
        cell_height: 0.0,
    };
    let ok = unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_get_pane_layout(fv_ptr, pane_ref, &mut layout) };
    (ok != 0 && layout.visible != 0).then_some(layout)
}

fn hostframe_rgba_gpu_copy(
    pane_id: i64,
    dst_x: f64,
    dst_y: f64,
    src_x: f64,
    src_y: f64,
    width: f64,
    height: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_rgba_gpu_copy(
                fv_ptr,
                pane_ref,
                dst_x.max(0.0) as u32,
                dst_y.max(0.0) as u32,
                pane_ref,
                src_x.max(0.0) as u32,
                src_y.max(0.0) as u32,
                width.max(0.0) as u32,
                height.max(0.0) as u32,
            )
        }
    })
}

/// Drain queued pane render batches and apply each command to the host frame.
///
/// Called from `on_frame` while `FRAME_VIEW` is set to a live frame view.
/// Each batch's commands are dispatched through [`PaneRenderCommand::apply`],
/// keeping this function as a thin orchestrator.
fn drain_pane_render_batches() {
    let pending = take_pending_batches();
    if pending.is_empty() {
        // Optional per-frame trace for empty drains — useful for confirming
        // on_frame is firing while CP isn't submitting anything (the most
        // suspicious "vanishes off the window" mode).
        if trace_frames_enabled() {
            wingui_trace!(
                "kind=drain_empty frame_index={} publish_ui_count={}",
                LAST_FRAME_INDEX.load(Ordering::Relaxed),
                PUBLISH_UI_COUNT.load(Ordering::Relaxed),
            );
        }
        return;
    }

    let frame_index = LAST_FRAME_INDEX.load(Ordering::Relaxed);
    wingui_trace!("kind=drain_begin frame_index={} batches={}", frame_index, pending.len());

    for batch in pending {
        eprintln!(
            "[WinBatch] drain pane={} seq={} flags={} commands={}",
            batch.pane_id, batch.sequence, batch.flags, batch.commands.len(),
        );
        let _ = hostframe_surface_reset_composition(batch.pane_id as i64);
        let pane_layout = current_pane_layout(batch.pane_id);
        // Layout is the most informative single signal: if it ever reports
        // visible=0 or width/height<=0 we'd never see content even if every
        // FFI call succeeds. Always log it on drain.
        match pane_layout {
            Some(l) => wingui_trace!(
                "kind=drain_batch pane={} seq={} cmds={} flags={} layout=(x={} y={} w={} h={} vis={} cols={} rows={})",
                batch.pane_id, batch.sequence, batch.commands.len(), batch.flags,
                l.x, l.y, l.width, l.height, l.visible, l.columns, l.rows,
            ),
            None => wingui_trace!(
                "kind=drain_batch pane={} seq={} cmds={} flags={} layout=none frame_view_null={}",
                batch.pane_id, batch.sequence, batch.commands.len(), batch.flags,
                FRAME_VIEW.with(|fv| fv.borrow().is_null()),
            ),
        }
        let pane_id_i64 = batch.pane_id as i64;
        for command in batch.commands {
            command.apply(pane_id_i64, pane_layout);
        }
    }

    wingui_trace!("kind=drain_end frame_index={}", frame_index);
}

/// Drain the queued batches under the global mutex.
///
/// Returns batches sorted by `(sequence, pane_id)` so frames render in a
/// deterministic order. On poison the queue is recovered and cleared so we
/// don't replay state from a thread that crashed mid-update.
fn take_pending_batches() -> Vec<PaneRenderBatch> {
    let mut guard = match pending_pane_batches().lock() {
        Ok(g) => g,
        Err(poisoned) => {
            wingui_trace!("kind=pending_batches_poisoned action=drop");
            eprintln!(
                "[wingui_host] pending pane batches mutex poisoned; dropping queued batches",
            );
            let mut g = poisoned.into_inner();
            g.clear();
            return Vec::new();
        }
    };
    let mut pending: Vec<_> = guard.drain().map(|(_, batch)| batch).collect();
    pending.sort_by_key(|batch| (batch.sequence, batch.pane_id));
    pending
}

/// Resolve the current frame's layout for `pane_id`, if a frame view is live.
fn current_pane_layout(
    pane_id: u64,
) -> Option<crate::wingui_ffi::SuperTerminalPaneLayout> {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        if fv_ptr.is_null() {
            None
        } else {
            bind_frame_pane_layout(fv_ptr, pane_id)
        }
    })
}

impl PaneRenderCommand {
    /// Dispatch this command to the host frame against `pane_id`.
    ///
    /// `pane_layout` is required only by `Clear`, which sizes its fill to the
    /// pane rect; if the layout is unknown the clear is silently skipped to
    /// match the original drain behavior.
    fn apply(
        self,
        pane_id: i64,
        pane_layout: Option<crate::wingui_ffi::SuperTerminalPaneLayout>,
    ) {
        match self {
            Self::Clear { buf_mode, color_r, color_g, color_b, color_a } => {
                let Some(layout) = pane_layout else { return };
                let _ = hostframe_fill_rect(
                    pane_id,
                    buf_mode, 0, 1,
                    color_r, color_g, color_b, color_a,
                    0.0, 0.0,
                    layout.width.max(0) as f64,
                    layout.height.max(0) as f64,
                    0.0,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::PushClipRect { x0, y0, x1, y1 } => {
                let _ = hostframe_surface_push_clip_rect(pane_id, x0, y0, x1, y1);
            }
            Self::PopClipRect => {
                let _ = hostframe_surface_pop_clip_rect(pane_id);
            }
            Self::PushOffset { dx, dy } => {
                let _ = hostframe_surface_push_offset(pane_id, dx, dy);
            }
            Self::PopOffset => {
                let _ = hostframe_surface_pop_offset(pane_id);
            }
            Self::TextCell { row, column, codepoint, fg, bg } => {
                let _ = hostframe_text_grid_write_cell(pane_id, row, column, codepoint, fg, bg);
            }
            Self::FillRect {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                x0, y0, x1, y1, corner_radius,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_fill_rect(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    x0, y0, x1, y1, corner_radius,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::StrokeRect {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                x0, y0, x1, y1, half_thickness, corner_radius,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_stroke_rect(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    x0, y0, x1, y1, half_thickness, corner_radius,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::DrawLine {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                x0, y0, x1, y1, half_thickness,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_draw_line(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    x0, y0, x1, y1, half_thickness,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::FillCircle {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                cx, cy, radius,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_fill_circle(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    cx, cy, radius,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::FillOval {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                x0, y0, x1, y1,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_fill_oval(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    x0, y0, x1, y1,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::StrokeCircle {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                cx, cy, radius, half_thickness,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_stroke_circle(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    cx, cy, radius, half_thickness,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::StrokeOval {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                x0, y0, x1, y1, half_thickness,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_stroke_oval(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    x0, y0, x1, y1, half_thickness,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::DrawArc {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                cx, cy, radius, half_thickness, rotation_rad, half_aperture_rad,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_draw_arc(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    cx, cy, radius, half_thickness, rotation_rad, half_aperture_rad,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::DrawPath {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                points_xy, closed, half_thickness,
                color_r, color_g, color_b, color_a,
            } => {
                if points_xy.len() < 4 {
                    return;
                }
                let _ = hostframe_draw_path(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    points_xy.as_ptr(),
                    (points_xy.len() / 2) as i32,
                    closed, half_thickness,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::DrawText {
                buf_mode, blend_mode, clear_before,
                clear_r, clear_g, clear_b, clear_a,
                text, origin_x, origin_y,
                color_r, color_g, color_b, color_a,
            } => {
                let text = CString::new(text).unwrap_or_default();
                let _ = hostframe_draw_text(
                    pane_id,
                    buf_mode, blend_mode, clear_before,
                    clear_r, clear_g, clear_b, clear_a,
                    text.as_ptr() as *const u8,
                    origin_x, origin_y,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::MarkRect { mode, x0, y0, x1, y1 } => {
                let _ = hostframe_mark_rect(
                    pane_id, 1, 0, 0, 0.0, 0.0, 0.0, 0.0, mode, x0, y0, x1, y1,
                );
            }
            Self::Caret { x0, y0, x1, y1, color_r, color_g, color_b, color_a } => {
                let _ = hostframe_caret(
                    pane_id, 1, 0, 0, 0.0, 0.0, 0.0, 0.0,
                    x0, y0, x1, y1,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::SelectionRange { x0, y0, x1, y1, color_r, color_g, color_b, color_a } => {
                let _ = hostframe_selection_range(
                    pane_id, 1, 0, 0, 0.0, 0.0, 0.0, 0.0,
                    x0, y0, x1, y1,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::FocusRing {
                x0, y0, x1, y1, half_thickness, corner_radius,
                color_r, color_g, color_b, color_a,
            } => {
                let _ = hostframe_focus_ring(
                    pane_id, 1, 0, 0, 0.0, 0.0, 0.0, 0.0,
                    x0, y0, x1, y1, half_thickness, corner_radius,
                    color_r, color_g, color_b, color_a,
                );
            }
            Self::ScrollRect { x0, y0, x1, y1, dx, dy } => {
                let src = SurfaceRect { x0, y0, x1, y1 }.normalize();
                if src.width() <= 0.0 || src.height() <= 0.0 {
                    return;
                }
                let _ = hostframe_rgba_gpu_copy(
                    pane_id,
                    src.x0 + dx, src.y0 + dy,
                    src.x0, src.y0,
                    src.width(), src.height(),
                );
            }
            Self::PresentHint => {
                winframe_request_present();
            }
            Self::InstallChildViewBounds { child_id, x0, y0, x1, y1 } => {
                let _ = hostframe_surface_install_child_view_bounds(
                    pane_id, child_id, x0, y0, x1, y1,
                );
            }
        }
    }
}

unsafe extern "system" fn on_frame(
    _user_data: *mut c_void,
    _runtime: *mut WinguiSpecBindRuntime,
    frame_view: *const WinguiSpecBindFrameView,
) {
    // Store frame_view for the duration of this call so WinFrame.* shims can use it.
    FRAME_VIEW.with(|fv| *fv.borrow_mut() = frame_view);

    let frame_index = if frame_view.is_null() {
        0
    } else {
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_index(frame_view) as u64 }
    };
    LAST_FRAME_INDEX.store(frame_index, Ordering::Relaxed);

    if !FIRST_FRAME_LOGGED.swap(true, Ordering::Relaxed) {
        wingui_trace!(
            "kind=on_frame_first frame_index={} frame_view_null={}",
            frame_index, frame_view.is_null(),
        );
    } else if trace_frames_enabled() {
        wingui_trace!(
            "kind=on_frame frame_index={} frame_view_null={}",
            frame_index, frame_view.is_null(),
        );
    }

    drain_pane_render_batches();

    // Clear frame_view before returning so stale use is caught as a null deref.
    FRAME_VIEW.with(|fv| *fv.borrow_mut() = std::ptr::null());
}

// ---------------------------------------------------------------------------
// Global runtime pointer
// ---------------------------------------------------------------------------

static RUNTIME: OnceLock<usize> = OnceLock::new();

fn runtime_ptr() -> *mut WinguiSpecBindRuntime {
    RUNTIME.get().copied().unwrap_or(0) as *mut WinguiSpecBindRuntime
}

// ---------------------------------------------------------------------------
// SpecBindRuntime RAII wrapper
// ---------------------------------------------------------------------------

pub struct SpecBindRuntime {
    ptr: *mut WinguiSpecBindRuntime,
}

unsafe impl Send for SpecBindRuntime {}

impl SpecBindRuntime {
    pub fn new() -> Option<Self> {
        let mut ptr: *mut WinguiSpecBindRuntime = std::ptr::null_mut();
        let ret = unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_create(&mut ptr) };
        eprintln!("[wingui_host] wingui_spec_bind_runtime_create: ret={} ptr={:?}", ret, ptr);
        if ret != 0 && !ptr.is_null() {
            // Load a minimal placeholder spec so that `run` can open the window
            // before the CP worker calls PublishUi with the real layout.
            let placeholder = c"{\"type\":\"window\",\"title\":\"NewCP\",\"children\":[{\"type\":\"textarea\",\"id\":\"log\",\"text\":\"Starting...\"}]}";
            unsafe {
                crate::wingui_spec_ffi::wingui_spec_bind_runtime_load_spec_json(
                    ptr, placeholder.as_ptr() as *const std::os::raw::c_char,
                );
                crate::wingui_spec_ffi::wingui_spec_bind_runtime_set_default_handler(
                    ptr, Some(on_event), std::ptr::null_mut(),
                );
                crate::wingui_spec_ffi::wingui_spec_bind_runtime_set_frame_handler(
                    ptr, Some(on_frame), std::ptr::null_mut(),
                );
            }
            eprintln!("[wingui_host] runtime created, handlers registered, placeholder spec loaded");
            Some(Self { ptr })
        } else {
            eprintln!("[wingui_host] wingui_spec_bind_runtime_create failed (ret={})", ret);
            None
        }
    }

    pub fn load_spec_json(&mut self, json: &str) -> bool {
        let c = CString::new(json).unwrap_or_default();
        let ret = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_runtime_load_spec_json(self.ptr, c.as_ptr())
        };
        if ret == 0 { eprintln!("[wingui_host] load_spec_json failed"); }
        ret != 0
    }

    pub fn request_stop(&self, exit_code: i32) {
        eprintln!("[wingui_host] request_stop called with exit_code={}", exit_code);
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_request_stop(self.ptr, exit_code); }
    }

    pub fn as_ptr(&self) -> *mut WinguiSpecBindRuntime { self.ptr }

    pub fn run(&self, config: &HostConfig) -> i32 {
        let _ = RUNTIME.set(self.ptr as usize);
        let title = CString::new(config.title.as_str()).unwrap_or_default();
        let font = config.font_family.as_deref()
            .map(|s| CString::new(s).unwrap_or_default());
        let font_ptr = font.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
        // Resolve shader path: use config override, else auto-detect <exe_dir>/shaders.
        let shader_path_str = config.shader_path.clone().or_else(|| {
            std::env::current_exe().ok()
                .and_then(|p| p.parent().map(|d| d.join("shaders")))
                .map(|p| p.to_string_lossy().into_owned())
        });
        let shader_cstr = shader_path_str.as_deref()
            .map(|s| CString::new(s).unwrap_or_default());
        let shader_ptr = shader_cstr.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null());
        let shader_path_display = shader_path_str.as_deref().unwrap_or("<none>");
        eprintln!("[wingui_host] run desc: title={:?} cols={} rows={} font_px={} shader_path={:?}",
            config.title, config.columns, config.rows, config.font_pixel_height, shader_path_display);
        let desc = WinguiSpecBindRunDesc {
            title_utf8: title.as_ptr(),
            columns: config.columns,
            rows: config.rows,
            flags: 0,
            command_queue_capacity: config.command_queue_capacity,
            event_queue_capacity: config.event_queue_capacity,
            font_family_utf8: font_ptr,
            font_pixel_height: config.font_pixel_height,
            dpi_scale: config.dpi_scale,
            text_shader_path_utf8: shader_ptr,
            target_frame_ms: 16,
            auto_request_present: 1,
        };
        let mut result = SuperTerminalRunResult { exit_code: 0, host_error_code: 0, message_utf8: [0; 256] };
        eprintln!("[wingui_host] calling spec_bind_runtime_run...");
        let ret = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_runtime_run(self.ptr, &desc, &mut result)
        };
        let msg = unsafe { CStr::from_ptr(result.message_utf8.as_ptr()) };
        eprintln!("[wingui_host] spec_bind_runtime_run returned: ret={} exit_code={} host_error={} msg={}",
            ret, result.exit_code, result.host_error_code, msg.to_string_lossy());
        result.exit_code
    }
}

impl Drop for SpecBindRuntime {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_destroy(self.ptr) };
        }
    }
}

// ---------------------------------------------------------------------------
// HostConfig
// ---------------------------------------------------------------------------

pub struct HostConfig {
    pub title: String,
    pub columns: u32,
    pub rows: u32,
    pub command_queue_capacity: u32,
    pub event_queue_capacity: u32,
    pub font_family: Option<String>,
    pub font_pixel_height: i32,
    pub dpi_scale: f32,
    /// Path to the directory containing wingui HLSL shader files.
    /// If `None`, auto-detected as `<exe_dir>/shaders`.
    pub shader_path: Option<String>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            title: "NewCP".to_string(),
            columns: 120, rows: 40,
            command_queue_capacity: 256, event_queue_capacity: 256,
            font_family: None, font_pixel_height: 18, dpi_scale: 1.0,
            shader_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// WinPayload helpers
// ---------------------------------------------------------------------------

const WINPAYLOAD_STR_CAP: usize = 4096;

fn copy_shortchar_string(dst_ptr: *mut u8, cap: usize, value: &str) {
    if dst_ptr.is_null() || cap == 0 {
        return;
    }
    let bytes = value.as_bytes();
    let copy_len = (cap - 1).min(bytes.len());
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst_ptr, copy_len);
        *dst_ptr.add(copy_len) = 0;
    }
}

fn clear_shortchar_string(dst_ptr: *mut u8, cap: usize) {
    if dst_ptr.is_null() || cap == 0 {
        return;
    }
    unsafe { *dst_ptr = 0; }
}

fn parse_payload_value(payload_ptr: *const u8) -> Option<Value> {
    if payload_ptr.is_null() {
        return None;
    }
    let payload = unsafe { CStr::from_ptr(payload_ptr as *const std::os::raw::c_char) }
        .to_string_lossy();
    serde_json::from_str::<Value>(&payload).ok()
}

fn parse_payload_key(key_ptr: *const u8) -> Option<String> {
    if key_ptr.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(key_ptr as *const std::os::raw::c_char) }
            .to_string_lossy()
            .into_owned(),
    )
}

fn packed_argb_to_rgba(colour: i64) -> [u8; 4] {
    let raw = colour as u32;
    let alpha = ((raw >> 24) & 0xff) as u8;
    [
        ((raw >> 16) & 0xff) as u8,
        ((raw >> 8) & 0xff) as u8,
        (raw & 0xff) as u8,
        if alpha == 0 { 0xff } else { alpha },
    ]
}

fn bind_frame_pane(frame_view: *const WinguiSpecBindFrameView, pane_id: i64) -> Option<WinguiSpecBindPaneRef> {
    if frame_view.is_null() {
        wingui_trace!("kind=bind_fail pane={} reason=frame_view_null", pane_id);
        return None;
    }
    let mut pane_ref = WinguiSpecBindPaneRef {
        window_id: crate::wingui_ffi::SuperTerminalWindowId { value: 0 },
        pane_id: crate::wingui_ffi::SuperTerminalPaneId { value: pane_id as u64 },
        buffer_index: 0,
        active_buffer_index: 0,
    };
    let ok = unsafe {
        crate::wingui_spec_ffi::wingui_spec_bind_frame_bind_pane(
            frame_view,
            crate::wingui_ffi::SuperTerminalPaneId { value: pane_id as u64 },
            &mut pane_ref,
        )
    };
    if ok == 0 {
        // Silent bind failures are the most common way for a hostframe call
        // to return 0 with no other signal. Trace once per failure so we can
        // tell whether a vanish is "we never bound the pane" vs "we drew but
        // it didn't appear".
        wingui_trace!("kind=bind_fail pane={} reason=spec_bind_returned_zero", pane_id);
        None
    } else {
        Some(pane_ref)
    }
}

fn clear_rgba(clear_r: f64, clear_g: f64, clear_b: f64, clear_a: f64) -> [f32; 4] {
    [clear_r as f32, clear_g as f32, clear_b as f32, clear_a as f32]
}

/// Parse the JSON payload, look up `key`, and run `extract` to decode the
/// expected value type. Returns `None` if any step fails.
fn winpayload_get_field<T>(
    payload_ptr: *const u8,
    key_ptr: *const u8,
    extract: impl FnOnce(&Value) -> Option<T>,
) -> Option<T> {
    let payload = parse_payload_value(payload_ptr)?;
    let key = parse_payload_key(key_ptr)?;
    payload.get(&key).and_then(extract)
}

#[unsafe(export_name = "WinPayload.GetStr")]
pub extern "C" fn winpayload_get_str(
    payload_ptr: *const u8,
    key_ptr: *const u8,
    out_ptr: *mut u8,
) -> i32 {
    clear_shortchar_string(out_ptr, WINPAYLOAD_STR_CAP);
    let Some(value) = winpayload_get_field(payload_ptr, key_ptr, |v| {
        v.as_str().map(str::to_owned)
    }) else {
        return 0;
    };
    copy_shortchar_string(out_ptr, WINPAYLOAD_STR_CAP, &value);
    1
}

#[unsafe(export_name = "WinPayload.GetInt")]
pub extern "C" fn winpayload_get_int(
    payload_ptr: *const u8,
    key_ptr: *const u8,
    out_ptr: *mut i64,
) -> i32 {
    if !out_ptr.is_null() {
        unsafe { *out_ptr = 0; }
    }
    let Some(value) = winpayload_get_field(payload_ptr, key_ptr, Value::as_i64) else {
        return 0;
    };
    if !out_ptr.is_null() {
        unsafe { *out_ptr = value; }
    }
    1
}

#[unsafe(export_name = "WinPayload.GetBool")]
pub extern "C" fn winpayload_get_bool(
    payload_ptr: *const u8,
    key_ptr: *const u8,
    out_ptr: *mut i32,
) -> i32 {
    if !out_ptr.is_null() {
        unsafe { *out_ptr = 0; }
    }
    let Some(value) = winpayload_get_field(payload_ptr, key_ptr, Value::as_bool) else {
        return 0;
    };
    if !out_ptr.is_null() {
        unsafe { *out_ptr = if value { 1 } else { 0 }; }
    }
    1
}

// ---------------------------------------------------------------------------
// HostWindows shims
// ---------------------------------------------------------------------------

#[unsafe(export_name = "HostWindows.RequestPresent")]
pub extern "C" fn host_request_present() {}

#[unsafe(export_name = "HostWindows.RequestClose")]
pub extern "C" fn host_request_close() {
    eprintln!("[wingui_host] HostWindows.RequestClose called");
    let r = runtime_ptr();
    if r.is_null() { eprintln!("[wingui_host] RequestClose: runtime ptr is null"); return; }
    unsafe { crate::wingui_spec_ffi::wingui_spec_bind_runtime_request_stop(r, 0) };
}

#[unsafe(export_name = "HostWindows.PublishUi")]
pub extern "C" fn host_publish_ui(json_ptr: *const u8) {
    // Spec republish is the prime suspect for the surface-pane "vanish" bug:
    // if the C++ smart-diff destroys and re-creates the surface node's HWND
    // the host presenter is rebuilt and the buffer content is lost. CP draws
    // once, so it never repaints. Increment a counter so other trace points
    // (drain, presenter_create on the C++ side) can be correlated against
    // exact PublishUi events.
    let publish_id = PUBLISH_UI_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    let r = runtime_ptr();
    if r.is_null() {
        wingui_trace!("kind=publish_ui id={} state=runtime_null", publish_id);
        eprintln!("[wingui_host] PublishUi: runtime ptr is null — window not open yet");
        return;
    }
    if json_ptr.is_null() {
        wingui_trace!("kind=publish_ui id={} state=json_null", publish_id);
        eprintln!("[wingui_host] PublishUi: json_ptr is null");
        return;
    }
    let (json_preview, json_len) = unsafe {
        let s = CStr::from_ptr(json_ptr as *const std::os::raw::c_char).to_string_lossy();
        let len = s.len();
        let preview = if len > 200 { format!("{}...", &s[..200]) } else { s.into_owned() };
        (preview, len)
    };
    wingui_trace!(
        "kind=publish_ui id={} state=submit json_len={} frame_index={}",
        publish_id, json_len, LAST_FRAME_INDEX.load(Ordering::Relaxed),
    );
    eprintln!("[wingui_host] PublishUi: json={}", json_preview);
    let ret = unsafe {
        crate::wingui_spec_ffi::wingui_spec_bind_runtime_load_spec_json(
            r, json_ptr as *const std::os::raw::c_char,
        )
    };
    wingui_trace!("kind=publish_ui id={} state=done ret={}", publish_id, ret);
    eprintln!("[wingui_host] PublishUi: load_spec_json returned {}", ret);
}

// ---------------------------------------------------------------------------
// WinFrame shims
// ---------------------------------------------------------------------------

/// WinFrame.FrameIndex — monotonically increasing frame counter.
/// Valid only inside a frame proc; returns 0 otherwise.
#[unsafe(export_name = "WinFrame.FrameIndex")]
pub extern "C" fn winframe_frame_index() -> i64 {
    FRAME_VIEW.with(|fv| {
        let ptr = *fv.borrow();
        if ptr.is_null() { return 0; }
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_index(ptr) as i64 }
    })
}

/// WinFrame.ElapsedMs — milliseconds since runtime start.
#[unsafe(export_name = "WinFrame.ElapsedMs")]
pub extern "C" fn winframe_elapsed_ms() -> i64 {
    FRAME_VIEW.with(|fv| {
        let ptr = *fv.borrow();
        if ptr.is_null() { return 0; }
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_elapsed_ms(ptr) as i64 }
    })
}

/// WinFrame.DeltaMs — milliseconds since the previous frame.
#[unsafe(export_name = "WinFrame.DeltaMs")]
pub extern "C" fn winframe_delta_ms() -> i64 {
    FRAME_VIEW.with(|fv| {
        let ptr = *fv.borrow();
        if ptr.is_null() { return 0; }
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_delta_ms(ptr) as i64 }
    })
}

/// WinFrame.ResolvePaneId — resolve widget node id → opaque pane INTEGER.
/// CP signature: PROCEDURE ResolvePaneId*(nodeId: ARRAY OF SHORTCHAR;
///                                        VAR paneId: INTEGER): INTSHORT
/// Call at startup after WinView.Render, not from inside a frame proc.
#[unsafe(export_name = "WinFrame.ResolvePaneId")]
pub extern "C" fn winframe_resolve_pane_id(node_id_ptr: *const u8, pane_id_ptr: *mut i64) -> i32 {
    if node_id_ptr.is_null() || pane_id_ptr.is_null() { return 0; }
    let r = runtime_ptr();
    if r.is_null() {
        eprintln!("[WinFrame.ResolvePaneId] runtime ptr is null");
        return 0;
    }
    let mut pane_id = crate::wingui_ffi::SuperTerminalPaneId { value: 0 };
    let ret = unsafe {
        crate::wingui_spec_ffi::wingui_spec_bind_runtime_resolve_pane_id_utf8(
            r,
            node_id_ptr as *const std::os::raw::c_char,
            &mut pane_id,
        )
    };
    if ret != 0 {
        unsafe { *pane_id_ptr = pane_id.value as i64; }
        eprintln!("[WinFrame.ResolvePaneId] pane_id={}", pane_id.value);
        1
    } else {
        eprintln!("[WinFrame.ResolvePaneId] resolve failed");
        0
    }
}

/// WinFrame.PaneLayout — get pixel rect of a pane this frame.
/// CP signature: PROCEDURE PaneLayout*(paneId: INTEGER;
///                VAR x, y, width, height: INTSHORT): INTSHORT
/// Valid only inside a frame proc.
#[unsafe(export_name = "WinFrame.PaneLayout")]
pub extern "C" fn winframe_pane_layout(
    pane_id: i64,
    x_ptr: *mut i32, y_ptr: *mut i32,
    w_ptr: *mut i32, h_ptr: *mut i32,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let mut layout = crate::wingui_ffi::SuperTerminalPaneLayout::default();
        let r2 = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_get_pane_layout(fv_ptr, pane_ref, &mut layout)
        };
        if r2 == 0 { return 0; }
        unsafe {
            if !x_ptr.is_null() { *x_ptr = layout.x; }
            if !y_ptr.is_null() { *y_ptr = layout.y; }
            if !w_ptr.is_null() { *w_ptr = layout.width; }
            if !h_ptr.is_null() { *h_ptr = layout.height; }
        }
        1
    })
}

/// WinFrame.RequestPresent — force a present this frame.
#[unsafe(export_name = "WinFrame.RequestPresent")]
pub extern "C" fn winframe_request_present() {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        if fv_ptr.is_null() { return; }
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_request_present(fv_ptr); }
    });
}

#[unsafe(export_name = "HostFrame.TextGridWriteCell")]
pub extern "C" fn hostframe_text_grid_write_cell(
    pane_id: i64,
    row: i32,
    column: i32,
    codepoint: i64,
    fg: i64,
    bg: i64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let cell = crate::wingui_ffi::SuperTerminalTextGridCell {
            row: row as u32,
            column: column as u32,
            codepoint: codepoint as u32,
            foreground: packed_argb_to_rgba(fg),
            background: packed_argb_to_rgba(bg),
        };
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_text_grid_write_cells(
                fv_ptr,
                pane_ref,
                &cell,
                1,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.TextGridClearRegion")]
pub extern "C" fn hostframe_text_grid_clear_region(
    pane_id: i64,
    row: i32,
    column: i32,
    width: i32,
    height: i32,
    fill_codepoint: i64,
    fg: i64,
    bg: i64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_text_grid_clear_region(
                fv_ptr,
                pane_ref,
                row as u32,
                column as u32,
                width as u32,
                height as u32,
                fill_codepoint as u32,
                packed_argb_to_rgba(fg),
                packed_argb_to_rgba(bg),
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.DrawLine")]
pub extern "C" fn hostframe_draw_line(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_draw_line(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                half_thickness as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.FillRect")]
pub extern "C" fn hostframe_fill_rect(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    corner_radius: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_fill_rect(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                corner_radius as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.StrokeRect")]
pub extern "C" fn hostframe_stroke_rect(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    half_thickness: f64,
    corner_radius: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_stroke_rect(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                half_thickness as f32,
                corner_radius as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.MarkRect")]
pub extern "C" fn hostframe_mark_rect(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    mode: i32,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_mark_rect(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                mode,
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.Caret")]
pub extern "C" fn hostframe_caret(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_caret(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.SelectionRange")]
pub extern "C" fn hostframe_selection_range(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_selection_range(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.FocusRing")]
pub extern "C" fn hostframe_focus_ring(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    half_thickness: f64,
    corner_radius: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_focus_ring(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                half_thickness as f32,
                corner_radius as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.SurfacePushClipRect")]
pub extern "C" fn hostframe_surface_push_clip_rect(
    pane_id: i64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_surface_push_clip_rect(
                fv_ptr,
                pane_ref,
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.SurfacePopClipRect")]
pub extern "C" fn hostframe_surface_pop_clip_rect(pane_id: i64) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_surface_pop_clip_rect(fv_ptr, pane_ref) }
    })
}

#[unsafe(export_name = "HostFrame.SurfacePushOffset")]
pub extern "C" fn hostframe_surface_push_offset(pane_id: i64, dx: f64, dy: f64) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_surface_push_offset(
                fv_ptr,
                pane_ref,
                dx as f32,
                dy as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.SurfacePopOffset")]
pub extern "C" fn hostframe_surface_pop_offset(pane_id: i64) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_surface_pop_offset(fv_ptr, pane_ref) }
    })
}

#[unsafe(export_name = "HostFrame.SurfaceResetComposition")]
pub extern "C" fn hostframe_surface_reset_composition(pane_id: i64) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe { crate::wingui_spec_ffi::wingui_spec_bind_frame_surface_reset_composition(fv_ptr, pane_ref) }
    })
}

#[unsafe(export_name = "HostFrame.SurfaceInstallChildViewBounds")]
pub extern "C" fn hostframe_surface_install_child_view_bounds(
    pane_id: i64,
    child_id: i32,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_surface_install_child_view_bounds(
                fv_ptr,
                pane_ref,
                child_id,
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.FillCircle")]
pub extern "C" fn hostframe_fill_circle(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    cx: f64,
    cy: f64,
    radius: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_fill_circle(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                cx as f32,
                cy as f32,
                radius as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.FillOval")]
pub extern "C" fn hostframe_fill_oval(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_fill_oval(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.StrokeCircle")]
pub extern "C" fn hostframe_stroke_circle(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    cx: f64,
    cy: f64,
    radius: f64,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_stroke_circle(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                cx as f32,
                cy as f32,
                radius as f32,
                half_thickness as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.StrokeOval")]
pub extern "C" fn hostframe_stroke_oval(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_stroke_oval(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                x0 as f32,
                y0 as f32,
                x1 as f32,
                y1 as f32,
                half_thickness as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.DrawArc")]
pub extern "C" fn hostframe_draw_arc(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    cx: f64,
    cy: f64,
    radius: f64,
    half_thickness: f64,
    rotation_rad: f64,
    half_aperture_rad: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_draw_arc(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                cx as f32,
                cy as f32,
                radius as f32,
                half_thickness as f32,
                rotation_rad as f32,
                half_aperture_rad as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.DrawPath")]
pub extern "C" fn hostframe_draw_path(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    points_ptr: *const f32,
    point_count: i32,
    closed: i32,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    if points_ptr.is_null() || point_count < 2 {
        return 0;
    }
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_draw_path(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                points_ptr,
                point_count as u32,
                closed,
                half_thickness as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.DrawText")]
pub extern "C" fn hostframe_draw_text(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    text_ptr: *const u8,
    origin_x: f64,
    origin_y: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    if text_ptr.is_null() {
        return 0;
    }
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let Some(pane_ref) = bind_frame_pane(fv_ptr, pane_id) else {
            return 0;
        };
        let clear = clear_rgba(clear_r, clear_g, clear_b, clear_a);
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_draw_text_utf8(
                fv_ptr,
                pane_ref,
                buf_mode as u32,
                blend_mode as u32,
                clear_before,
                clear.as_ptr(),
                text_ptr as *const std::os::raw::c_char,
                origin_x as f32,
                origin_y as f32,
                color_r as f32,
                color_g as f32,
                color_b as f32,
                color_a as f32,
            )
        }
    })
}

#[unsafe(export_name = "HostFrame.DrawTextRun")]
pub extern "C" fn hostframe_draw_text_run(
    pane_id: i64,
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    text_ptr: *const u8,
    origin_x: f64,
    origin_y: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    hostframe_draw_text(
        pane_id,
        buf_mode,
        blend_mode,
        clear_before,
        clear_r,
        clear_g,
        clear_b,
        clear_a,
        text_ptr,
        origin_x,
        origin_y,
        color_r,
        color_g,
        color_b,
        color_a,
    )
}

#[unsafe(export_name = "HostFrame.MeasureTextRun")]
pub extern "C" fn hostframe_measure_text_run(
    text_ptr: *const u8,
    width_ptr: *mut f64,
    height_ptr: *mut f64,
    char_count_ptr: *mut i64,
) -> i32 {
    if text_ptr.is_null() || width_ptr.is_null() || height_ptr.is_null() || char_count_ptr.is_null() {
        return 0;
    }
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let mut char_count = 0u32;
        let ok = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_measure_text_utf8(
                fv_ptr,
                text_ptr as *const std::os::raw::c_char,
                &mut width,
                &mut height,
                &mut char_count,
            )
        };
        if ok != 0 {
            unsafe {
                *width_ptr = width as f64;
                *height_ptr = height as f64;
                *char_count_ptr = char_count as i64;
            }
        }
        ok
    })
}

#[unsafe(export_name = "HostFrame.CharIndexAtPoint")]
pub extern "C" fn hostframe_char_index_at_point(
    text_ptr: *const u8,
    origin_x: f64,
    origin_y: f64,
    x: f64,
    y: f64,
    char_index_ptr: *mut i64,
) -> i32 {
    if text_ptr.is_null() || char_index_ptr.is_null() {
        return 0;
    }
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let mut char_index = 0u32;
        let ok = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_char_index_at_point_utf8(
                fv_ptr,
                text_ptr as *const std::os::raw::c_char,
                origin_x as f32,
                origin_y as f32,
                x as f32,
                y as f32,
                &mut char_index,
            )
        };
        if ok != 0 {
            unsafe {
                *char_index_ptr = char_index as i64;
            }
        }
        ok
    })
}

#[unsafe(export_name = "HostFrame.PointAtCharIndex")]
pub extern "C" fn hostframe_point_at_char_index(
    text_ptr: *const u8,
    origin_x: f64,
    origin_y: f64,
    char_index: i64,
    x_ptr: *mut f64,
    y_ptr: *mut f64,
) -> i32 {
    if text_ptr.is_null() || x_ptr.is_null() || y_ptr.is_null() {
        return 0;
    }
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        let mut x = 0.0f32;
        let mut y = 0.0f32;
        let ok = unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_point_at_char_index_utf8(
                fv_ptr,
                text_ptr as *const std::os::raw::c_char,
                origin_x as f32,
                origin_y as f32,
                char_index.max(0) as u32,
                &mut x,
                &mut y,
            )
        };
        if ok != 0 {
            unsafe {
                *x_ptr = x as f64;
                *y_ptr = y as f64;
            }
        }
        ok
    })
}

/// WinFrame.PostPaneMsg — post (kind, detail) to pane's inbox from any thread.
/// CP signature: PROCEDURE PostPaneMsg*(paneId: INTEGER;
///                 kind, detail: ARRAY OF SHORTCHAR): INTSHORT
#[unsafe(export_name = "WinFrame.PostPaneMsg")]
pub extern "C" fn winframe_post_pane_msg(
    pane_id:    i64,
    kind_ptr:   *const u8,
    detail_ptr: *const u8,
) -> i32 {
    let r = runtime_ptr();
    if r.is_null() { return 0; }
    if kind_ptr.is_null() { return 0; }
    static EMPTY: &[u8] = b"\0";
    unsafe {
        crate::wingui_spec_ffi::wingui_spec_bind_post_pane_msg(
            r,
            crate::wingui_ffi::SuperTerminalPaneId { value: pane_id as u64 },
            kind_ptr as *const std::os::raw::c_char,
            if detail_ptr.is_null() { EMPTY.as_ptr() as *const std::os::raw::c_char }
            else { detail_ptr as *const std::os::raw::c_char },
        )
    }
}

/// WinFrame.PollPaneMsg — drain one message from pane's inbox (frame thread only).
/// CP signature: PROCEDURE PollPaneMsg*(paneId: INTEGER;
///                 VAR kind: ARRAY OF SHORTCHAR;
///                 VAR detail: ARRAY OF SHORTCHAR): INTSHORT
/// kind and detail buffers are assumed to be at least 64 and 128 bytes respectively
/// (matching CP ARRAY 64 OF SHORTCHAR and ARRAY 128 OF SHORTCHAR fixed arrays).
#[unsafe(export_name = "WinFrame.PollPaneMsg")]
pub extern "C" fn winframe_poll_pane_msg(
    pane_id:    i64,
    kind_ptr:   *mut u8,
    detail_ptr: *mut u8,
) -> i32 {
    FRAME_VIEW.with(|fv| {
        let fv_ptr = *fv.borrow();
        if fv_ptr.is_null() { return 0; }
        unsafe {
            crate::wingui_spec_ffi::wingui_spec_bind_frame_poll_pane_msg(
                fv_ptr,
                crate::wingui_ffi::SuperTerminalPaneId { value: pane_id as u64 },
                kind_ptr   as *mut std::os::raw::c_char, 64,
                detail_ptr as *mut std::os::raw::c_char, 128,
            )
        }
    })
}

// ---------------------------------------------------------------------------
// WinBatch shims
// ---------------------------------------------------------------------------

#[unsafe(export_name = "WinBatch.Begin")]
pub extern "C" fn winbatch_begin(pane_id: i64, sequence: i64, flags: i32) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        *current.borrow_mut() = Some(PaneRenderBatch {
            pane_id: pane_id as u64,
            sequence: sequence as u64,
            flags: flags as u32,
            commands: Vec::new(),
        });
    });
    1
}

#[unsafe(export_name = "WinBatch.Clear")]
pub extern "C" fn winbatch_clear(buf_mode: i32, color_r: f64, color_g: f64, color_b: f64, color_a: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::Clear { buf_mode, color_r, color_g, color_b, color_a });
        1
    })
}

#[unsafe(export_name = "WinBatch.PushClipRect")]
pub extern "C" fn winbatch_push_clip_rect(x0: f64, y0: f64, x1: f64, y1: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::PushClipRect { x0, y0, x1, y1 });
        1
    })
}

#[unsafe(export_name = "WinBatch.PopClipRect")]
pub extern "C" fn winbatch_pop_clip_rect() -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::PopClipRect);
        1
    })
}

#[unsafe(export_name = "WinBatch.PushOffset")]
pub extern "C" fn winbatch_push_offset(dx: f64, dy: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::PushOffset { dx, dy });
        1
    })
}

#[unsafe(export_name = "WinBatch.PopOffset")]
pub extern "C" fn winbatch_pop_offset() -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::PopOffset);
        1
    })
}

#[unsafe(export_name = "WinBatch.TextCell")]
pub extern "C" fn winbatch_text_cell(
    row: i32,
    column: i32,
    codepoint: i64,
    fg: i64,
    bg: i64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::TextCell {
            row,
            column,
            codepoint,
            fg,
            bg,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.FillRect")]
pub extern "C" fn winbatch_fill_rect(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    corner_radius: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::FillRect {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            x0,
            y0,
            x1,
            y1,
            corner_radius,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.StrokeRect")]
pub extern "C" fn winbatch_stroke_rect(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    half_thickness: f64,
    corner_radius: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::StrokeRect {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            x0,
            y0,
            x1,
            y1,
            half_thickness,
            corner_radius,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.DrawLine")]
pub extern "C" fn winbatch_draw_line(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::DrawLine {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            x0,
            y0,
            x1,
            y1,
            half_thickness,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.FillCircle")]
pub extern "C" fn winbatch_fill_circle(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    cx: f64,
    cy: f64,
    radius: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::FillCircle {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            cx,
            cy,
            radius,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.FillOval")]
pub extern "C" fn winbatch_fill_oval(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::FillOval {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            x0,
            y0,
            x1,
            y1,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.StrokeCircle")]
pub extern "C" fn winbatch_stroke_circle(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    cx: f64,
    cy: f64,
    radius: f64,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::StrokeCircle {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            cx,
            cy,
            radius,
            half_thickness,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.StrokeOval")]
pub extern "C" fn winbatch_stroke_oval(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::StrokeOval {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            x0,
            y0,
            x1,
            y1,
            half_thickness,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.DrawArc")]
pub extern "C" fn winbatch_draw_arc(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    cx: f64,
    cy: f64,
    radius: f64,
    half_thickness: f64,
    rotation_rad: f64,
    half_aperture_rad: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::DrawArc {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            cx,
            cy,
            radius,
            half_thickness,
            rotation_rad,
            half_aperture_rad,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.DrawPath")]
pub extern "C" fn winbatch_draw_path(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    points_ptr: *const f64,
    point_count: i32,
    closed: i32,
    half_thickness: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    if points_ptr.is_null() || point_count < 2 {
        return 0;
    }
    let point_count = point_count as usize;
    let source = unsafe { std::slice::from_raw_parts(points_ptr, point_count * 2) };
    let mut points_xy = Vec::with_capacity(source.len());
    points_xy.extend(source.iter().map(|value| *value as f32));
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::DrawPath {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            points_xy,
            closed,
            half_thickness,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.DrawText")]
pub extern "C" fn winbatch_draw_text(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    text_ptr: *const u8,
    origin_x: f64,
    origin_y: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    if text_ptr.is_null() {
        return 0;
    }
    let text = unsafe { CStr::from_ptr(text_ptr as *const std::os::raw::c_char) }
        .to_string_lossy()
        .into_owned();
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else {
            return 0;
        };
        batch.commands.push(PaneRenderCommand::DrawText {
            buf_mode,
            blend_mode,
            clear_before,
            clear_r,
            clear_g,
            clear_b,
            clear_a,
            text,
            origin_x,
            origin_y,
            color_r,
            color_g,
            color_b,
            color_a,
        });
        1
    })
}

#[unsafe(export_name = "WinBatch.DrawTextRun")]
pub extern "C" fn winbatch_draw_text_run(
    buf_mode: i32,
    blend_mode: i32,
    clear_before: i32,
    clear_r: f64,
    clear_g: f64,
    clear_b: f64,
    clear_a: f64,
    text_ptr: *const u8,
    origin_x: f64,
    origin_y: f64,
    color_r: f64,
    color_g: f64,
    color_b: f64,
    color_a: f64,
) -> i32 {
    winbatch_draw_text(
        buf_mode,
        blend_mode,
        clear_before,
        clear_r,
        clear_g,
        clear_b,
        clear_a,
        text_ptr,
        origin_x,
        origin_y,
        color_r,
        color_g,
        color_b,
        color_a,
    )
}

#[unsafe(export_name = "WinBatch.MarkRect")]
pub extern "C" fn winbatch_mark_rect(mode: i32, x0: f64, y0: f64, x1: f64, y1: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::MarkRect { mode, x0, y0, x1, y1 });
        1
    })
}

#[unsafe(export_name = "WinBatch.Caret")]
pub extern "C" fn winbatch_caret(x0: f64, y0: f64, x1: f64, y1: f64, color_r: f64, color_g: f64, color_b: f64, color_a: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::Caret { x0, y0, x1, y1, color_r, color_g, color_b, color_a });
        1
    })
}

#[unsafe(export_name = "WinBatch.SelectionRange")]
pub extern "C" fn winbatch_selection_range(x0: f64, y0: f64, x1: f64, y1: f64, color_r: f64, color_g: f64, color_b: f64, color_a: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::SelectionRange { x0, y0, x1, y1, color_r, color_g, color_b, color_a });
        1
    })
}

#[unsafe(export_name = "WinBatch.FocusRing")]
pub extern "C" fn winbatch_focus_ring(x0: f64, y0: f64, x1: f64, y1: f64, half_thickness: f64, corner_radius: f64, color_r: f64, color_g: f64, color_b: f64, color_a: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::FocusRing { x0, y0, x1, y1, half_thickness, corner_radius, color_r, color_g, color_b, color_a });
        1
    })
}

#[unsafe(export_name = "WinBatch.ScrollRect")]
pub extern "C" fn winbatch_scroll_rect(x0: f64, y0: f64, x1: f64, y1: f64, dx: f64, dy: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::ScrollRect { x0, y0, x1, y1, dx, dy });
        1
    })
}

#[unsafe(export_name = "WinBatch.PresentHint")]
pub extern "C" fn winbatch_present_hint() -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::PresentHint);
        1
    })
}

#[unsafe(export_name = "WinBatch.InstallChildViewBounds")]
pub extern "C" fn winbatch_install_child_view_bounds(child_id: i32, x0: f64, y0: f64, x1: f64, y1: f64) -> i32 {
    CURRENT_PANE_BATCH.with(|current| {
        let mut current = current.borrow_mut();
        let Some(batch) = current.as_mut() else { return 0; };
        batch.commands.push(PaneRenderCommand::InstallChildViewBounds { child_id, x0, y0, x1, y1 });
        1
    })
}

#[unsafe(export_name = "WinBatch.Submit")]
pub extern "C" fn winbatch_submit() -> i32 {
    let Some(batch) = CURRENT_PANE_BATCH.with(|current| current.borrow_mut().take()) else {
        wingui_trace!("kind=submit_no_open_batch");
        return 0;
    };

    let pane_id = batch.pane_id;
    let sequence = batch.sequence;
    let cmd_count = batch.commands.len();

    let mut pending = recover_lock(pending_pane_batches().lock(), "pending pane batches submit");
    if let Some(existing) = pending.get(&pane_id) {
        if existing.sequence > sequence {
            wingui_trace!(
                "kind=submit_dropped_stale pane={} seq={} existing_seq={} cmds={}",
                pane_id, sequence, existing.sequence, cmd_count,
            );
            // Original behavior: log to stderr without trace prefix as well, so the
            // legacy `[WinBatch]` grep keeps matching.
            eprintln!(
                "[WinBatch] drop stale pane={pane_id} seq={sequence} existing_seq={}",
                existing.sequence,
            );
            return 0;
        }
    }
    let replaced = pending.insert(pane_id, batch).is_some();
    wingui_trace!(
        "kind=submit pane={} seq={} cmds={} replaced_pending={}",
        pane_id, sequence, cmd_count, replaced,
    );
    1
}

/// WaitNamedEvent: blocks until an event is available or timeout elapses.
/// Returns 1 on event, 0 on timeout.
/// Signature matches CP: PROCEDURE (VAR name: ARRAY 256 OF SHORTCHAR;
///                                   VAR payload: ARRAY 4096 OF SHORTCHAR;
///                                   timeout: INTEGER): INTEGER
/// Fixed-size VAR arrays are passed as bare pointers (no length word).
#[unsafe(export_name = "HostWindows.WaitNamedEvent")]
pub extern "C" fn host_wait_named_event(
    name_ptr:    *mut u8,
    payload_ptr: *mut u8,
    timeout_ms:  i64,
) -> i32 {
    const NAME_CAP: usize    = 256;
    const PAYLOAD_CAP: usize = 4096;
    if name_ptr.is_null() { return 0; }
    let event = {
        let mut q = recover_lock(EVENT_QUEUE.queue.lock(), "EVENT_QUEUE wait_named_event");
        if timeout_ms < 0 {
            loop {
                if let Some(ev) = q.pop_front() { break ev; }
                q = recover_lock(EVENT_QUEUE.ready.wait(q), "EVENT_QUEUE wait");
            }
        } else {
            let dur = std::time::Duration::from_millis(timeout_ms as u64);
            let (mut q2, _) = match EVENT_QUEUE.ready.wait_timeout(q, dur) {
                Ok(pair) => pair,
                Err(poisoned) => {
                    eprintln!("[wingui_host] mutex poisoned at EVENT_QUEUE wait_timeout; recovering");
                    poisoned.into_inner()
                }
            };
            match q2.pop_front() {
                Some(ev) => ev,
                None => return 0,
            }
        }
    };
    let name_bytes = event.name.as_bytes();
    let name_copy = (NAME_CAP - 1).min(name_bytes.len());
    unsafe {
        std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), name_ptr, name_copy);
        *name_ptr.add(name_copy) = 0;
    }
    if !payload_ptr.is_null() {
        let pay_bytes = event.payload.as_bytes();
        let pay_copy = (PAYLOAD_CAP - 1).min(pay_bytes.len());
        unsafe {
            std::ptr::copy_nonoverlapping(pay_bytes.as_ptr(), payload_ptr, pay_copy);
            *payload_ptr.add(pay_copy) = 0;
        }
    }
    1
}

// ---------------------------------------------------------------------------
// WinSpec builder shims
// ---------------------------------------------------------------------------

struct FrameEntry {
    type_:    &'static str,
    gap:      Option<i32>,
    children: Vec<String>,
}

struct WinSpecBuilder {
    title: String,
    stack: Vec<FrameEntry>,
}

impl WinSpecBuilder {
    fn new() -> Self { Self { title: String::new(), stack: vec![] } }

    fn reset(&mut self, title: &str) {
        self.title = title.to_owned();
        self.stack = vec![FrameEntry { type_: "stack", gap: None, children: vec![] }];
    }

    fn push_leaf(&mut self, json: String) {
        if let Some(frame) = self.stack.last_mut() { frame.children.push(json); }
    }

    fn open(&mut self, type_: &'static str, gap: Option<i32>) {
        self.stack.push(FrameEntry { type_, gap, children: vec![] });
    }

    fn close(&mut self) {
        if self.stack.len() <= 1 { return; }
        let frame = self.stack.pop().unwrap();
        let gap_part = frame.gap.map(|g| format!(",\"gap\":{}", g)).unwrap_or_default();
        let node = format!(
            "{{\"type\":\"{}\"{},\"children\":[{}]}}",
            frame.type_, gap_part, frame.children.join(",")
        );
        self.push_leaf(node);
    }

    fn build(&self) -> String {
        let body_children = self.stack.last()
            .map(|f| f.children.join(",")).unwrap_or_default();
        let title_esc = escape_json(&self.title);
        format!(
            "{{\"type\":\"window\",\"title\":\"{}\",\"body\":{{\"type\":\"stack\",\"children\":[{}]}}}}",
            title_esc, body_children
        )
    }
}

thread_local! {
    static SPEC: RefCell<WinSpecBuilder> = RefCell::new(WinSpecBuilder::new());
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
     .replace('\n', "\\n").replace('\r', "\\r")
}

#[unsafe(export_name = "WinSpec.Begin")]
pub extern "C" fn winspec_begin(title_ptr: *const u8) {
    if title_ptr.is_null() { return; }
    let title = unsafe { CStr::from_ptr(title_ptr as *const _) }.to_string_lossy();
    eprintln!("[WinSpec.Begin] title={:?}", title);
    SPEC.with(|s| s.borrow_mut().reset(&title));
}

#[unsafe(export_name = "WinSpec.OpenStack")]
pub extern "C" fn winspec_open_stack(gap: i32) {
    eprintln!("[WinSpec.OpenStack] gap={}", gap);
    SPEC.with(|s| s.borrow_mut().open("stack", if gap < 0 { None } else { Some(gap) }));
}

#[unsafe(export_name = "WinSpec.OpenRow")]
pub extern "C" fn winspec_open_row(gap: i32) {
    eprintln!("[WinSpec.OpenRow] gap={}", gap);
    SPEC.with(|s| s.borrow_mut().open("row", if gap < 0 { None } else { Some(gap) }));
}

#[unsafe(export_name = "WinSpec.CloseContainer")]
pub extern "C" fn winspec_close_container() {
    eprintln!("[WinSpec.CloseContainer]");
    SPEC.with(|s| s.borrow_mut().close());
}

#[unsafe(export_name = "WinSpec.AddTextarea")]
pub extern "C" fn winspec_add_textarea(
    id_ptr: *const u8, label_ptr: *const u8, value_ptr: *const u8, readonly: i32,
) {
    let id    = unsafe { CStr::from_ptr(id_ptr    as *const _) }.to_string_lossy();
    let label = unsafe { CStr::from_ptr(label_ptr as *const _) }.to_string_lossy();
    let value = escape_json(&unsafe { CStr::from_ptr(value_ptr as *const _) }.to_string_lossy());
    eprintln!("[WinSpec.AddTextarea] id={:?} label={:?} readonly={}", id, label, readonly);
    let ro = if readonly != 0 { ",\"readonly\":true" } else { "" };
    SPEC.with(|s| s.borrow_mut().push_leaf(format!(
        "{{\"type\":\"textarea\",\"id\":\"{}\",\"label\":\"{}\",\"value\":\"{}\"{}}}",
        id, label, value, ro
    )));
}

#[unsafe(export_name = "WinSpec.AddButton")]
pub extern "C" fn winspec_add_button(id_ptr: *const u8, label_ptr: *const u8, event_ptr: *const u8) {
    let id    = unsafe { CStr::from_ptr(id_ptr    as *const _) }.to_string_lossy();
    let label = unsafe { CStr::from_ptr(label_ptr as *const _) }.to_string_lossy();
    let ev    = unsafe { CStr::from_ptr(event_ptr as *const _) }.to_string_lossy();
    eprintln!("[WinSpec.AddButton] id={:?} event={:?}", id, ev);
    SPEC.with(|s| s.borrow_mut().push_leaf(format!(
        "{{\"type\":\"button\",\"id\":\"{}\",\"text\":\"{}\",\"event\":\"{}\"}}",
        id, label, ev
    )));
}

#[unsafe(export_name = "WinSpec.AddText")]
pub extern "C" fn winspec_add_text(text_ptr: *const u8) {
    let text = escape_json(&unsafe { CStr::from_ptr(text_ptr as *const _) }.to_string_lossy());
    eprintln!("[WinSpec.AddText] text={:?}", text);
    SPEC.with(|s| s.borrow_mut().push_leaf(format!("{{\"type\":\"text\",\"text\":\"{}\"}}", text)));
}

#[unsafe(export_name = "WinSpec.AddTextGrid")]
pub extern "C" fn winspec_add_text_grid(
    id_ptr: *const u8,
    event_ptr: *const u8,
    cols: i32,
    rows: i32,
) {
    let id = unsafe { CStr::from_ptr(id_ptr as *const _) }.to_string_lossy();
    let event = if event_ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(event_ptr as *const _) }.to_string_lossy().into_owned()
    };
    eprintln!("[WinSpec.AddTextGrid] id={:?} event={:?} cols={} rows={}", id, event, cols, rows);
    let event_part = if event.is_empty() {
        String::new()
    } else {
        format!(",\"event\":\"{}\"", escape_json(&event))
    };
    SPEC.with(|s| s.borrow_mut().push_leaf(format!(
        "{{\"type\":\"text-grid\",\"id\":\"{}\",\"columns\":{},\"rows\":{}{}}}",
        escape_json(&id),
        cols.max(1),
        rows.max(1),
        event_part
    )));
}

#[unsafe(export_name = "WinSpec.AddRgbaPane")]
pub extern "C" fn winspec_add_rgba_pane(
    id_ptr: *const u8,
    event_ptr: *const u8,
    width: i32,
    height: i32,
) {
    let id = unsafe { CStr::from_ptr(id_ptr as *const _) }.to_string_lossy();
    let event = if event_ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(event_ptr as *const _) }.to_string_lossy().into_owned()
    };
    eprintln!("[WinSpec.AddRgbaPane] id={:?} event={:?} width={} height={}", id, event, width, height);
    let event_part = if event.is_empty() {
        String::new()
    } else {
        format!(",\"event\":\"{}\"", escape_json(&event))
    };
    SPEC.with(|s| s.borrow_mut().push_leaf(format!(
        "{{\"type\":\"rgba-pane\",\"id\":\"{}\",\"width\":{},\"height\":{}{}}}",
        escape_json(&id),
        width.max(1),
        height.max(1),
        event_part
    )));
}

#[unsafe(export_name = "WinSpec.AddSurface")]
pub extern "C" fn winspec_add_surface(
    id_ptr: *const u8,
    event_ptr: *const u8,
    width: i32,
    height: i32,
) {
    let id = unsafe { CStr::from_ptr(id_ptr as *const _) }.to_string_lossy();
    let event = if event_ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(event_ptr as *const _) }.to_string_lossy().into_owned()
    };
    eprintln!("[WinSpec.AddSurface] id={:?} event={:?} width={} height={}", id, event, width, height);
    let event_part = if event.is_empty() {
        String::new()
    } else {
        format!(",\"event\":\"{}\"", escape_json(&event))
    };
    SPEC.with(|s| s.borrow_mut().push_leaf(format!(
        "{{\"type\":\"surface\",\"id\":\"{}\",\"width\":{},\"height\":{}{}}}",
        escape_json(&id),
        width.max(1),
        height.max(1),
        event_part
    )));
}

#[unsafe(export_name = "WinSpec.GetSpec")]
pub extern "C" fn winspec_get_spec(buf_ptr: *mut u8) -> i32 {
    // VAR spec: ARRAY 10240 OF SHORTCHAR — fixed array, passed as bare pointer.
    const BUF_CAP: usize = 10240;
    if buf_ptr.is_null() { return 0; }
    let json = SPEC.with(|s| s.borrow().build());
    let bytes = json.as_bytes();
    eprintln!("[WinSpec.GetSpec] json_len={} cap={} json={}", bytes.len(), BUF_CAP, &json);
    if bytes.len() + 1 > BUF_CAP { return 0; }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf_ptr, bytes.len());
        *buf_ptr.add(bytes.len()) = 0;
    }
    1
}

// ---------------------------------------------------------------------------
// Module artifacts
// ---------------------------------------------------------------------------

pub fn native_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "HostWindows", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("RequestPresent"),
                ExportEntry::procedure("RequestClose"),
                ExportEntry::procedure("PublishUi"),
                ExportEntry::procedure("WaitNamedEvent"),
            ]),
            "HostWindows.bootstrap",
            "Rust-hosted wingui spec_bind bridge",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("RequestPresent", host_request_present as *const () as usize),
            NativeExportBinding::procedure("RequestClose",   host_request_close   as *const () as usize),
            NativeExportBinding::procedure("PublishUi",      host_publish_ui      as *const () as usize),
            NativeExportBinding::procedure("WaitNamedEvent", host_wait_named_event as *const () as usize),
        ],
    )
}

pub fn winspec_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "WinSpec", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("Begin"),
                ExportEntry::procedure("OpenStack"),
                ExportEntry::procedure("OpenRow"),
                ExportEntry::procedure("CloseContainer"),
                ExportEntry::procedure("AddTextarea"),
                ExportEntry::procedure("AddButton"),
                ExportEntry::procedure("AddText"),
                ExportEntry::procedure("AddTextGrid"),
                ExportEntry::procedure("AddRgbaPane"),
                ExportEntry::procedure("GetSpec"),
            ]),
            "WinSpec.bootstrap",
            "Rust-hosted JSON spec builder for wingui layouts",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("Begin",          winspec_begin           as *const () as usize),
            NativeExportBinding::procedure("OpenStack",      winspec_open_stack      as *const () as usize),
            NativeExportBinding::procedure("OpenRow",        winspec_open_row        as *const () as usize),
            NativeExportBinding::procedure("CloseContainer", winspec_close_container as *const () as usize),
            NativeExportBinding::procedure("AddTextarea",    winspec_add_textarea    as *const () as usize),
            NativeExportBinding::procedure("AddButton",      winspec_add_button      as *const () as usize),
            NativeExportBinding::procedure("AddText",        winspec_add_text        as *const () as usize),
            NativeExportBinding::procedure("AddTextGrid",    winspec_add_text_grid   as *const () as usize),
            NativeExportBinding::procedure("AddRgbaPane",    winspec_add_rgba_pane   as *const () as usize),
            NativeExportBinding::procedure("GetSpec",        winspec_get_spec        as *const () as usize),
        ],
    )
}

pub fn winframe_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "WinFrame", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("FrameIndex"),
                ExportEntry::procedure("ElapsedMs"),
                ExportEntry::procedure("DeltaMs"),
                ExportEntry::procedure("ResolvePaneId"),
                ExportEntry::procedure("PaneLayout"),
                ExportEntry::procedure("RequestPresent"),
                ExportEntry::procedure("PostPaneMsg"),
                ExportEntry::procedure("PollPaneMsg"),
            ]),
            "WinFrame.bootstrap",
            "Rust-hosted frame-state and pane inbox shims for wingui MVC",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("FrameIndex",             winframe_frame_index             as *const () as usize),
            NativeExportBinding::procedure("ElapsedMs",              winframe_elapsed_ms              as *const () as usize),
            NativeExportBinding::procedure("DeltaMs",                winframe_delta_ms                as *const () as usize),
            NativeExportBinding::procedure("ResolvePaneId",          winframe_resolve_pane_id         as *const () as usize),
            NativeExportBinding::procedure("PaneLayout",             winframe_pane_layout             as *const () as usize),
            NativeExportBinding::procedure("RequestPresent",         winframe_request_present         as *const () as usize),
            NativeExportBinding::procedure("PostPaneMsg",            winframe_post_pane_msg           as *const () as usize),
            NativeExportBinding::procedure("PollPaneMsg",            winframe_poll_pane_msg           as *const () as usize),
        ],
    )
}

pub fn winpayload_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "WinPayload", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("GetStr"),
                ExportEntry::procedure("GetInt"),
                ExportEntry::procedure("GetBool"),
            ]),
            "WinPayload.bootstrap",
            "Rust-hosted JSON payload field extraction for wingui events",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("GetStr",  winpayload_get_str  as *const () as usize),
            NativeExportBinding::procedure("GetInt",  winpayload_get_int  as *const () as usize),
            NativeExportBinding::procedure("GetBool", winpayload_get_bool as *const () as usize),
        ],
    )
}

pub fn winbatch_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "WinBatch", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("Begin"),
                ExportEntry::procedure("Clear"),
                ExportEntry::procedure("PushClipRect"),
                ExportEntry::procedure("PopClipRect"),
                ExportEntry::procedure("PushOffset"),
                ExportEntry::procedure("PopOffset"),
                ExportEntry::procedure("TextCell"),
                ExportEntry::procedure("DrawLine"),
                ExportEntry::procedure("DrawText"),
                ExportEntry::procedure("DrawTextRun"),
                ExportEntry::procedure("FillRect"),
                ExportEntry::procedure("StrokeRect"),
                ExportEntry::procedure("FillCircle"),
                ExportEntry::procedure("FillOval"),
                ExportEntry::procedure("StrokeCircle"),
                ExportEntry::procedure("StrokeOval"),
                ExportEntry::procedure("DrawArc"),
                ExportEntry::procedure("DrawPath"),
                ExportEntry::procedure("MarkRect"),
                ExportEntry::procedure("Caret"),
                ExportEntry::procedure("SelectionRange"),
                ExportEntry::procedure("FocusRing"),
                ExportEntry::procedure("ScrollRect"),
                ExportEntry::procedure("PresentHint"),
                ExportEntry::procedure("InstallChildViewBounds"),
                ExportEntry::procedure("Submit"),
            ]),
            "WinBatch.bootstrap",
            "Rust-hosted typed pane batch staging for wingui MVC",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("Begin",    winbatch_begin    as *const () as usize),
            NativeExportBinding::procedure("Clear",    winbatch_clear    as *const () as usize),
            NativeExportBinding::procedure("PushClipRect", winbatch_push_clip_rect as *const () as usize),
            NativeExportBinding::procedure("PopClipRect", winbatch_pop_clip_rect as *const () as usize),
            NativeExportBinding::procedure("PushOffset", winbatch_push_offset as *const () as usize),
            NativeExportBinding::procedure("PopOffset", winbatch_pop_offset as *const () as usize),
            NativeExportBinding::procedure("TextCell", winbatch_text_cell as *const () as usize),
            NativeExportBinding::procedure("DrawLine", winbatch_draw_line as *const () as usize),
            NativeExportBinding::procedure("DrawText", winbatch_draw_text as *const () as usize),
            NativeExportBinding::procedure("DrawTextRun", winbatch_draw_text_run as *const () as usize),
            NativeExportBinding::procedure("FillRect", winbatch_fill_rect as *const () as usize),
            NativeExportBinding::procedure("StrokeRect", winbatch_stroke_rect as *const () as usize),
            NativeExportBinding::procedure("FillCircle", winbatch_fill_circle as *const () as usize),
            NativeExportBinding::procedure("FillOval", winbatch_fill_oval as *const () as usize),
            NativeExportBinding::procedure("StrokeCircle", winbatch_stroke_circle as *const () as usize),
            NativeExportBinding::procedure("StrokeOval", winbatch_stroke_oval as *const () as usize),
            NativeExportBinding::procedure("DrawArc", winbatch_draw_arc as *const () as usize),
            NativeExportBinding::procedure("DrawPath", winbatch_draw_path as *const () as usize),
            NativeExportBinding::procedure("MarkRect", winbatch_mark_rect as *const () as usize),
            NativeExportBinding::procedure("Caret", winbatch_caret as *const () as usize),
            NativeExportBinding::procedure("SelectionRange", winbatch_selection_range as *const () as usize),
            NativeExportBinding::procedure("FocusRing", winbatch_focus_ring as *const () as usize),
            NativeExportBinding::procedure("ScrollRect", winbatch_scroll_rect as *const () as usize),
            NativeExportBinding::procedure("PresentHint", winbatch_present_hint as *const () as usize),
            NativeExportBinding::procedure("InstallChildViewBounds", winbatch_install_child_view_bounds as *const () as usize),
            NativeExportBinding::procedure("Submit",   winbatch_submit   as *const () as usize),
        ],
    )
}

pub fn hostframe_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "HostFrame", vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("TextGridWriteCell"),
                ExportEntry::procedure("TextGridClearRegion"),
                ExportEntry::procedure("DrawLine"),
                ExportEntry::procedure("FillRect"),
                ExportEntry::procedure("StrokeRect"),
                ExportEntry::procedure("FillCircle"),
                ExportEntry::procedure("FillOval"),
                ExportEntry::procedure("StrokeCircle"),
                ExportEntry::procedure("StrokeOval"),
                ExportEntry::procedure("DrawArc"),
                ExportEntry::procedure("DrawText"),
                ExportEntry::procedure("DrawTextRun"),
                ExportEntry::procedure("DrawPath"),
                ExportEntry::procedure("MeasureTextRun"),
                ExportEntry::procedure("CharIndexAtPoint"),
                ExportEntry::procedure("PointAtCharIndex"),
            ]),
            "HostFrame.bootstrap",
            "Rust-hosted frame-time pane helpers for wingui surfaces",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("TextGridWriteCell", hostframe_text_grid_write_cell as *const () as usize),
            NativeExportBinding::procedure("TextGridClearRegion", hostframe_text_grid_clear_region as *const () as usize),
            NativeExportBinding::procedure("DrawLine", hostframe_draw_line as *const () as usize),
            NativeExportBinding::procedure("FillRect", hostframe_fill_rect as *const () as usize),
            NativeExportBinding::procedure("StrokeRect", hostframe_stroke_rect as *const () as usize),
            NativeExportBinding::procedure("FillCircle", hostframe_fill_circle as *const () as usize),
            NativeExportBinding::procedure("FillOval", hostframe_fill_oval as *const () as usize),
            NativeExportBinding::procedure("StrokeCircle", hostframe_stroke_circle as *const () as usize),
            NativeExportBinding::procedure("StrokeOval", hostframe_stroke_oval as *const () as usize),
            NativeExportBinding::procedure("DrawArc", hostframe_draw_arc as *const () as usize),
            NativeExportBinding::procedure("DrawText", hostframe_draw_text as *const () as usize),
            NativeExportBinding::procedure("DrawTextRun", hostframe_draw_text_run as *const () as usize),
            NativeExportBinding::procedure("DrawPath", hostframe_draw_path as *const () as usize),
            NativeExportBinding::procedure("MeasureTextRun", hostframe_measure_text_run as *const () as usize),
            NativeExportBinding::procedure("CharIndexAtPoint", hostframe_char_index_at_point as *const () as usize),
            NativeExportBinding::procedure("PointAtCharIndex", hostframe_point_at_char_index as *const () as usize),
        ],
    )
}
