/// wingui_spec_ffi.rs
///
/// Raw FFI bindings for wingui's spec_bind and spec_builder APIs.
/// spec_bind owns the hosted run loop, window creation, event dispatch and
/// UI reconciliation.  spec_builder validates / normalises JSON specs.
///
/// Header sources:
///   include/wingui/spec_bind.h
///   include/wingui/spec_builder.h

use std::os::raw::c_char;

use crate::wingui_ffi::{
    SuperTerminalNativeUiPatchMetrics, SuperTerminalPaneId, SuperTerminalRunResult,
    SuperTerminalTextGridCell, SuperTerminalWindowDesc, SuperTerminalWindowId,
};

// ---------------------------------------------------------------------------
// Opaque runtime handle
// ---------------------------------------------------------------------------

/// Opaque spec_bind runtime (heap-allocated by wingui).
#[repr(C)]
pub struct WinguiSpecBindRuntime {
    _unused: [u8; 0],
}

/// Opaque frame-view handle (valid only inside a frame callback).
#[repr(C)]
pub struct WinguiSpecBindFrameView {
    _unused: [u8; 0],
}

// ---------------------------------------------------------------------------
// Event and frame view structs
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct WinguiSpecBindEventView {
    pub window_id: SuperTerminalWindowId,
    pub event_name_utf8: *const c_char,
    pub payload_json_utf8: *const c_char,
    pub source_utf8: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WinguiSpecBindPaneRef {
    pub window_id: SuperTerminalWindowId,
    pub pane_id: SuperTerminalPaneId,
    pub buffer_index: u32,
    pub active_buffer_index: u32,
}

// ---------------------------------------------------------------------------
// Callback types
// ---------------------------------------------------------------------------

pub type WinguiSpecBindEventHandlerFn = unsafe extern "system" fn(
    user_data: *mut std::ffi::c_void,
    runtime: *mut WinguiSpecBindRuntime,
    event_view: *const WinguiSpecBindEventView,
);

pub type WinguiSpecBindFrameHandlerFn = unsafe extern "system" fn(
    user_data: *mut std::ffi::c_void,
    runtime: *mut WinguiSpecBindRuntime,
    frame_view: *const WinguiSpecBindFrameView,
);

// ---------------------------------------------------------------------------
// Run descriptor
// ---------------------------------------------------------------------------

#[repr(C)]
pub struct WinguiSpecBindRunDesc {
    pub title_utf8: *const c_char,
    pub columns: u32,
    pub rows: u32,
    pub flags: u32,
    pub command_queue_capacity: u32,
    pub event_queue_capacity: u32,
    pub font_family_utf8: *const c_char,
    pub font_pixel_height: i32,
    pub dpi_scale: f32,
    pub text_shader_path_utf8: *const c_char,
    pub target_frame_ms: u32,
    pub auto_request_present: i32,
}

// ---------------------------------------------------------------------------
// FFI declarations — spec_bind
// ---------------------------------------------------------------------------

unsafe extern "system" {
    /// Create a new spec_bind runtime.  The caller owns it and must call
    /// `wingui_spec_bind_runtime_destroy` when done.
    pub fn wingui_spec_bind_runtime_create(
        out_runtime: *mut *mut WinguiSpecBindRuntime,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_destroy(runtime: *mut WinguiSpecBindRuntime);

    /// Load (or replace) the declarative JSON window spec.
    pub fn wingui_spec_bind_runtime_load_spec_json(
        runtime: *mut WinguiSpecBindRuntime,
        json_utf8: *const c_char,
    ) -> i32;

    /// Copy the current spec JSON into a caller-supplied buffer.
    /// Pass null + 0 to query the required size.
    pub fn wingui_spec_bind_runtime_copy_spec_json(
        runtime: *mut WinguiSpecBindRuntime,
        buffer_utf8: *mut c_char,
        buffer_size: u32,
        out_required_size: *mut u32,
    ) -> i32;

    /// Bind a named event handler (e.g. a button id).
    pub fn wingui_spec_bind_runtime_bind_event(
        runtime: *mut WinguiSpecBindRuntime,
        event_name_utf8: *const c_char,
        handler: Option<WinguiSpecBindEventHandlerFn>,
        user_data: *mut std::ffi::c_void,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_unbind_event(
        runtime: *mut WinguiSpecBindRuntime,
        event_name_utf8: *const c_char,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_clear_bindings(runtime: *mut WinguiSpecBindRuntime);

    /// Catch-all handler for events without a specific binding.
    pub fn wingui_spec_bind_runtime_set_default_handler(
        runtime: *mut WinguiSpecBindRuntime,
        handler: Option<WinguiSpecBindEventHandlerFn>,
        user_data: *mut std::ffi::c_void,
    );

    pub fn wingui_spec_bind_runtime_set_frame_handler(
        runtime: *mut WinguiSpecBindRuntime,
        handler: Option<WinguiSpecBindFrameHandlerFn>,
        user_data: *mut std::ffi::c_void,
    );

    /// Request the host to stop.
    pub fn wingui_spec_bind_runtime_request_stop(
        runtime: *mut WinguiSpecBindRuntime,
        exit_code: i32,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_create_window(
        runtime: *mut WinguiSpecBindRuntime,
        desc: *const SuperTerminalWindowDesc,
        out_window_id: *mut SuperTerminalWindowId,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_close_window(
        runtime: *mut WinguiSpecBindRuntime,
        window_id: SuperTerminalWindowId,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_get_patch_metrics(
        runtime: *mut WinguiSpecBindRuntime,
        out_metrics: *mut SuperTerminalNativeUiPatchMetrics,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_resolve_pane_id_utf8(
        runtime: *mut WinguiSpecBindRuntime,
        node_id_utf8: *const c_char,
        out_pane_id: *mut SuperTerminalPaneId,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_resolve_pane_id_for_window(
        runtime: *mut WinguiSpecBindRuntime,
        window_id: SuperTerminalWindowId,
        node_id_utf8: *const c_char,
        out_pane_id: *mut SuperTerminalPaneId,
    ) -> i32;

    pub fn wingui_spec_bind_runtime_text_grid_write_cells(
        runtime: *mut WinguiSpecBindRuntime,
        pane_id: SuperTerminalPaneId,
        cells: *const SuperTerminalTextGridCell,
        cell_count: u32,
    ) -> i32;

    /// Block the calling thread on the spec_bind event + frame loop.
    /// Returns when the user closes the window or `request_stop` is called.
    pub fn wingui_spec_bind_runtime_run(
        runtime: *mut WinguiSpecBindRuntime,
        desc: *const WinguiSpecBindRunDesc,
        out_result: *mut SuperTerminalRunResult,
    ) -> i32;
}

// ---------------------------------------------------------------------------
// FFI declarations — frame-time helpers (valid inside frame callback only)
// ---------------------------------------------------------------------------

unsafe extern "system" {
    pub fn wingui_spec_bind_frame_index(
        frame_view: *const WinguiSpecBindFrameView,
    ) -> u64;

    pub fn wingui_spec_bind_frame_elapsed_ms(
        frame_view: *const WinguiSpecBindFrameView,
    ) -> u64;

    pub fn wingui_spec_bind_frame_delta_ms(
        frame_view: *const WinguiSpecBindFrameView,
    ) -> u64;

    pub fn wingui_spec_bind_frame_bind_pane(
        frame_view: *const WinguiSpecBindFrameView,
        pane_id: crate::wingui_ffi::SuperTerminalPaneId,
        out_pane: *mut WinguiSpecBindPaneRef,
    ) -> i32;

    pub fn wingui_spec_bind_frame_get_pane_layout(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        out_layout: *mut crate::wingui_ffi::SuperTerminalPaneLayout,
    ) -> i32;

    pub fn wingui_spec_bind_frame_request_present(
        frame_view: *const WinguiSpecBindFrameView,
    ) -> i32;

    pub fn wingui_spec_bind_frame_text_grid_write_cells(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        cells: *const SuperTerminalTextGridCell,
        cell_count: u32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_text_grid_clear_region(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        row: u32,
        column: u32,
        width: u32,
        height: u32,
        fill_codepoint: u32,
        foreground: [u8; 4],
        background: [u8; 4],
    ) -> i32;

    pub fn wingui_spec_bind_frame_rgba_gpu_copy(
        frame_view: *const WinguiSpecBindFrameView,
        dst_pane: WinguiSpecBindPaneRef,
        dst_x: u32,
        dst_y: u32,
        src_pane: WinguiSpecBindPaneRef,
        src_x: u32,
        src_y: u32,
        region_width: u32,
        region_height: u32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_draw_line(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        half_thickness: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_fill_rect(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        corner_radius: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_stroke_rect(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        half_thickness: f32,
        corner_radius: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_fill_circle(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        cx: f32,
        cy: f32,
        radius: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_stroke_circle(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        cx: f32,
        cy: f32,
        radius: f32,
        half_thickness: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_draw_arc(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        cx: f32,
        cy: f32,
        radius: f32,
        half_thickness: f32,
        rotation_rad: f32,
        half_aperture_rad: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_fill_oval(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_stroke_oval(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        half_thickness: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_draw_path(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        points_xy: *const f32,
        point_count: u32,
        closed: i32,
        half_thickness: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_draw_text_utf8(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        text_utf8: *const c_char,
        origin_x: f32,
        origin_y: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_mark_rect(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        mode: i32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_caret(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_selection_range(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_focus_ring(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        content_buffer_mode: u32,
        blend_mode: u32,
        clear_before: i32,
        clear_color_rgba: *const f32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        half_thickness: f32,
        corner_radius: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
        color_a: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_surface_push_clip_rect(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_surface_pop_clip_rect(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
    ) -> i32;

    pub fn wingui_spec_bind_frame_surface_push_offset(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        dx: f32,
        dy: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_surface_pop_offset(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
    ) -> i32;

    pub fn wingui_spec_bind_frame_surface_reset_composition(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
    ) -> i32;

    pub fn wingui_spec_bind_frame_surface_install_child_view_bounds(
        frame_view: *const WinguiSpecBindFrameView,
        pane: WinguiSpecBindPaneRef,
        child_id: i32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_measure_text_utf8(
        frame_view: *const WinguiSpecBindFrameView,
        text_utf8: *const c_char,
        out_width: *mut f32,
        out_height: *mut f32,
        out_char_count: *mut u32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_char_index_at_point_utf8(
        frame_view: *const WinguiSpecBindFrameView,
        text_utf8: *const c_char,
        origin_x: f32,
        origin_y: f32,
        x: f32,
        y: f32,
        out_char_index: *mut u32,
    ) -> i32;

    pub fn wingui_spec_bind_frame_point_at_char_index_utf8(
        frame_view: *const WinguiSpecBindFrameView,
        text_utf8: *const c_char,
        origin_x: f32,
        origin_y: f32,
        char_index: u32,
        out_x: *mut f32,
        out_y: *mut f32,
    ) -> i32;

    // Pane inbox — cross-thread messaging (CP event thread → D3D11 frame thread)

    /// Post a (kind, detail) string pair to pane_id's inbox.
    /// Callable from any thread.  Returns 1 on success, 0 if full/invalid.
    pub fn wingui_spec_bind_post_pane_msg(
        runtime: *mut WinguiSpecBindRuntime,
        pane_id: crate::wingui_ffi::SuperTerminalPaneId,
        kind_utf8: *const c_char,
        detail_utf8: *const c_char,
    ) -> i32;

    /// Drain one message from pane_id's inbox.
    /// Frame thread only.  Returns 1 if a message was dequeued, 0 if empty.
    pub fn wingui_spec_bind_frame_poll_pane_msg(
        frame_view: *const WinguiSpecBindFrameView,
        pane_id: crate::wingui_ffi::SuperTerminalPaneId,
        kind_out: *mut c_char,
        kind_cap: u32,
        detail_out: *mut c_char,
        detail_cap: u32,
    ) -> i32;
}

// ---------------------------------------------------------------------------
// FFI declarations — spec_builder
// ---------------------------------------------------------------------------

unsafe extern "system" {
    /// Validate a JSON window spec.  Returns 0 on success, non-zero on error.
    pub fn wingui_spec_builder_validate_json(json_utf8: *const c_char) -> i32;

    /// Write canonical (stable-key-ordered) JSON into `buffer_utf8`.
    /// Pass null + 0 to query the required size (returned in `out_required_size`).
    pub fn wingui_spec_builder_copy_canonical_json(
        json_utf8: *const c_char,
        buffer_utf8: *mut c_char,
        buffer_size: u32,
        out_required_size: *mut u32,
    ) -> i32;

    /// Write normalised JSON (auto-ids filled, defaults applied) into buffer.
    pub fn wingui_spec_builder_copy_normalized_json(
        json_utf8: *const c_char,
        buffer_utf8: *mut c_char,
        buffer_size: u32,
        out_required_size: *mut u32,
    ) -> i32;

    /// Compute a JSON Patch (RFC 6902) between two specs.
    /// `out_requires_full_publish` is set to 1 when the diff cannot be
    /// applied incrementally and a full republish is needed.
    pub fn wingui_spec_builder_copy_patch_json(
        old_json_utf8: *const c_char,
        new_json_utf8: *const c_char,
        buffer_utf8: *mut c_char,
        buffer_size: u32,
        out_required_size: *mut u32,
        out_requires_full_publish: *mut i32,
        out_patch_op_count: *mut u32,
    ) -> i32;
}
