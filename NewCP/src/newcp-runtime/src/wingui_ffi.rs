use std::ffi::c_void;
use std::os::raw::c_char;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SuperTerminalClientContext {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SuperTerminalPaneId {
    pub value: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SuperTerminalWindowId {
    pub value: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SuperTerminalWindowDesc {
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
    pub initial_ui_json_utf8: *const c_char,
}

pub const SUPERTERMINAL_WAIT_INFINITE: u32 = 0xffffffff;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SuperTerminalCommandType {
    Nop = 0,
    CreateWindow = 1,
    CloseWindow = 2,
    NativeUiPublish = 3,
    NativeUiPatch = 4,
    WindowSetTitle = 5,
    TextGridWriteCells = 6,
    TextGridClearRegion = 7,
    RequestPresent = 8,
    RequestClose = 9,
    RgbaUploadOwned = 10,
    FrameSwap = 11,
    RgbaGpuCopy = 12,
    RgbaAssetRegisterOwned = 13,
    RgbaAssetBlitToPane = 14,
    IndexedUploadOwned = 15,
    SpriteDefineOwned = 16,
    SpriteRender = 17,
    VectorDrawOwned = 18,
    IndexedFillRect = 19,
    IndexedDrawLine = 20,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SuperTerminalEventType {
    None = 0,
    Key = 1,
    Char = 2,
    Mouse = 3,
    PaneInput = 4,
    Resize = 5,
    Focus = 6,
    NativeUi = 7,
    CloseRequested = 8,
    HostStopping = 9,
    WindowCreated = 10,
    WindowClosed = 11,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SuperTerminalHostErrorCode {
    None = 0,
    InvalidArgument = 1,
    WindowCreate = 2,
    ContextCreate = 3,
    GlyphAtlasCreate = 4,
    RendererCreate = 5,
    NativeUiAttach = 6,
    ClientStart = 7,
    MessageLoop = 8,
}

#[repr(C)]
pub struct SuperTerminalEvent {
    pub window_id: SuperTerminalWindowId,
    pub event_type: u32, // SuperTerminalEventType tag
    pub sequence: u32,
    // Union covers the largest member (NativeUiEvent: WindowId + 512 byte string).
    // 512 + 8 (window_id) = 520; round up to 528 for alignment.
    pub data: [u8; 528],
}

#[repr(C)]
pub struct SuperTerminalRunResult {
    pub exit_code: i32,
    pub host_error_code: i32, // SuperTerminalHostErrorCode
    pub message_utf8: [c_char; 256],
}

/// A single cell written to a text-grid pane.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SuperTerminalTextGridCell {
    pub row: u32,
    pub column: u32,
    pub codepoint: u32,
    pub foreground: [u8; 4], // WinguiGraphicsColour (RGBA u8 x4)
    pub background: [u8; 4],
}

/// Metrics reported by the native-UI patch reconciler.
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct SuperTerminalNativeUiPatchMetrics {
    pub publish_count: u64,
    pub patch_request_count: u64,
    pub direct_apply_count: u64,
    pub subtree_rebuild_count: u64,
    pub window_rebuild_count: u64,
    pub resize_reject_count: u64,
    pub failed_patch_count: u64,
}

pub type SuperTerminalStartupFn = extern "system" fn(*mut SuperTerminalClientContext, *mut c_void) -> i32;
pub type SuperTerminalShutdownFn = extern "system" fn(*mut c_void);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SuperTerminalAppDesc {
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
    pub initial_ui_json_utf8: *const c_char,
    pub user_data: *mut c_void,
    pub startup: Option<SuperTerminalStartupFn>,
    pub shutdown: Option<SuperTerminalShutdownFn>,
}

#[repr(C)]
pub struct SuperTerminalCommand {
    pub command_type: u32, // SuperTerminalCommandType
    pub sequence: u32,
    pub padding: [u8; 128], // Enough size to cover union types (e.g. SpriteDefineOwned, CreateWindow)
}

unsafe extern "system" {
    pub fn super_terminal_run(
        desc: *const SuperTerminalAppDesc,
        out_result: *mut SuperTerminalRunResult,
    ) -> i32;

    pub fn super_terminal_enqueue(
        ctx: *mut SuperTerminalClientContext,
        command: *const SuperTerminalCommand,
    ) -> i32;

    pub fn super_terminal_wait_event(
        ctx: *mut SuperTerminalClientContext,
        timeout_ms: u32,
        out_event: *mut SuperTerminalEvent,
    ) -> i32;

    pub fn super_terminal_request_stop(
        ctx: *mut SuperTerminalClientContext,
        exit_code: i32,
    ) -> i32;

    pub fn super_terminal_create_window(
        ctx: *mut SuperTerminalClientContext,
        desc: *const SuperTerminalWindowDesc,
        out_window_id: *mut SuperTerminalWindowId,
    ) -> i32;

    pub fn super_terminal_close_window(
        ctx: *mut SuperTerminalClientContext,
        window_id: SuperTerminalWindowId,
    ) -> i32;

    pub fn super_terminal_resolve_pane_id_for_window(
        ctx: *mut SuperTerminalClientContext,
        window_id: SuperTerminalWindowId,
        node_id_utf8: *const c_char,
        out_pane_id: *mut SuperTerminalPaneId,
    ) -> i32;
    
    pub fn super_terminal_publish_ui_json_for_window(
        ctx: *mut SuperTerminalClientContext,
        window_id: SuperTerminalWindowId,
        json_utf8: *const c_char,
    ) -> i32;

    pub fn super_terminal_patch_ui_json_for_window(
        ctx: *mut SuperTerminalClientContext,
        window_id: SuperTerminalWindowId,
        patch_json_utf8: *const c_char,
    ) -> i32;
}

// ---------------------------------------------------------------------------
// Pane layout
// ---------------------------------------------------------------------------

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
