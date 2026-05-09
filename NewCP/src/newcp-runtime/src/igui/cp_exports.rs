//! CP-callable shims for the `iGui` module. Phase 2 surface:
//! `NextEvent`, `Quit`. Drives the `channels` mailbox and posts
//! WM_CLOSE to the frame.

#![cfg(windows)]

use std::sync::OnceLock;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

use super::channels::{self, kind, IGuiEvent};
use crate::{
    ExportDirectory, ExportEntry, HostedModuleArtifact, NativeExportBinding, NativeModuleArtifact,
};

/// HWND of the iGui frame, set by `window::run` once the window
/// exists. Used by `iGui.Quit` to post WM_CLOSE.
pub static FRAME_HWND: OnceLock<isize> = OnceLock::new();

/// `iGui.NextEvent(VAR kind, childId, timeMs, p1, p2, p3, p4: INTEGER;
///                 timeoutMs: INTEGER): INTSHORT`.
///
/// Returns 1 if an event was delivered, 0 on timeout.
///
/// Field semantics by kind (all values written to the corresponding
/// VAR pointer):
///
/// | kind         | childId          | timeMs    | p1        | p2          | p3          | p4              |
/// |--------------|------------------|-----------|-----------|-------------|-------------|-----------------|
/// | EvKey        | child window id  | GetMsgTime| vkey      | scancode    | mods        | down(1)/up(0)\|repeat<<16 |
/// | EvChar       | child window id  | GetMsgTime| codepoint | mods        | 0           | 0               |
/// | EvMouse      | child window id  | GetMsgTime| x         | y           | mods\|button<<8\|op<<16 | wheel_delta\|wheel_lines<<16 |
/// | EvFocus      | child window id  | 0         | gained    | 0           | 0           | 0               |
/// | EvResize     | child window id  | 0         | width     | height      | 0           | 0               |
/// | EvClose      | child window id  | 0         | 0         | 0           | 0           | 0               |
/// | EvFrameClose | 0                | 0         | 0         | 0           | 0           | 0               |
/// | EvThemeChange| 0                | 0         | 0         | 0           | 0           | 0               |
/// | EvDpiChange  | child window id  | 0         | dpi_x×100 | dpi_y×100   | 0           | 0               |
/// | EvMenu       | 0                | 0         | menu_id   | item_id     | 0           | 0               |
#[unsafe(export_name = "iGui.NextEvent")]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn igui_next_event(
    out_kind: *mut i64,
    out_child: *mut i64,
    out_time: *mut i64,
    out_p1: *mut i64,
    out_p2: *mut i64,
    out_p3: *mut i64,
    out_p4: *mut i64,
    timeout_ms: i64,
) -> i32 {
    let Some(ev) = channels::next_event(timeout_ms) else {
        return 0;
    };
    write_event(ev, out_kind, out_child, out_time, out_p1, out_p2, out_p3, out_p4);
    1
}

/// `iGui.Quit`. Posts WM_CLOSE to the frame; the GUI thread tears down
/// in its own time.
#[unsafe(export_name = "iGui.Quit")]
pub extern "C" fn igui_quit() {
    if let Some(&hwnd_raw) = FRAME_HWND.get() {
        let hwnd = HWND(hwnd_raw as *mut _);
        let _ = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    }
}

/// `iGui.OpenChild(title: ARRAY OF SHORTCHAR; VAR childId: INTEGER): INTSHORT`.
///
/// Reads the null-terminated SHORTCHAR title, creates an MDI child via
/// `SendMessageW(WM_MDICREATE)` on the GUI thread, writes the new
/// child id to `*out_child`. Returns 1 on success, 0 on failure.
#[unsafe(export_name = "iGui.OpenChild")]
pub extern "C" fn igui_open_child(title: *const u8, out_child: *mut i64) -> i32 {
    if title.is_null() || out_child.is_null() {
        return 0;
    }
    let title_str = unsafe { read_cp_shortstr(title) };
    match super::window::open_child(&title_str) {
        Some(id) => {
            unsafe { *out_child = id };
            1
        }
        None => 0,
    }
}

/// `iGui.CloseChild(childId: INTEGER): INTSHORT`. Returns 1 on success,
/// 0 if the child id is unknown.
#[unsafe(export_name = "iGui.CloseChild")]
pub extern "C" fn igui_close_child(child_id: i64) -> i32 {
    if super::window::close_child(child_id) {
        1
    } else {
        0
    }
}

/// `iGui.SetTitle(childId: INTEGER; title: ARRAY OF SHORTCHAR)`.
#[unsafe(export_name = "iGui.SetTitle")]
pub extern "C" fn igui_set_title(child_id: i64, title: *const u8) {
    if title.is_null() {
        return;
    }
    let title_str = unsafe { read_cp_shortstr(title) };
    super::window::set_child_title(child_id, &title_str);
}

/// CP `ARRAY OF SHORTCHAR` is passed as a bare pointer to a sequence
/// of bytes terminated by `0X`. This helper reads up to 4096 bytes,
/// stops at the first NUL, and returns the lossy UTF-8 decoding.
unsafe fn read_cp_shortstr(ptr: *const u8) -> String {
    const MAX: usize = 4096;
    let mut len = 0usize;
    while len < MAX {
        let b = unsafe { *ptr.add(len) };
        if b == 0 {
            break;
        }
        len += 1;
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(slice).into_owned()
}

#[allow(clippy::too_many_arguments)]
#[allow(unused_assignments)] // initial defaults overwritten by every match arm
fn write_event(
    ev: IGuiEvent,
    out_kind: *mut i64,
    out_child: *mut i64,
    out_time: *mut i64,
    out_p1: *mut i64,
    out_p2: *mut i64,
    out_p3: *mut i64,
    out_p4: *mut i64,
) {
    let mut k = kind::NONE;
    let mut child = 0i64;
    let mut t = 0i64;
    let mut p1 = 0i64;
    let mut p2 = 0i64;
    let mut p3 = 0i64;
    let mut p4 = 0i64;

    match ev {
        IGuiEvent::Key {
            child_id,
            vkey,
            scancode,
            mods,
            repeat,
            down,
            time_ms,
        } => {
            k = kind::KEY;
            child = child_id;
            t = time_ms;
            p1 = vkey;
            p2 = scancode;
            p3 = mods;
            p4 = (if down { 1 } else { 0 }) | (repeat << 16);
        }
        IGuiEvent::Char {
            child_id,
            codepoint,
            mods,
            time_ms,
        } => {
            k = kind::CHAR;
            child = child_id;
            t = time_ms;
            p1 = codepoint;
            p2 = mods;
        }
        IGuiEvent::Mouse {
            child_id,
            x,
            y,
            op,
            button,
            mods,
            wheel_delta,
            wheel_lines,
            time_ms,
        } => {
            k = kind::MOUSE;
            child = child_id;
            t = time_ms;
            p1 = x;
            p2 = y;
            p3 = mods | (button << 8) | (op << 16);
            p4 = (wheel_delta & 0xFFFF) | (wheel_lines << 16);
        }
        IGuiEvent::Focus { child_id, gained } => {
            k = kind::FOCUS;
            child = child_id;
            p1 = if gained { 1 } else { 0 };
        }
        IGuiEvent::Resize {
            child_id,
            width,
            height,
        } => {
            k = kind::RESIZE;
            child = child_id;
            p1 = width;
            p2 = height;
        }
        IGuiEvent::Close { child_id } => {
            k = kind::CLOSE;
            child = child_id;
        }
        IGuiEvent::FrameClose => {
            k = kind::FRAME_CLOSE;
        }
        IGuiEvent::ThemeChange => {
            k = kind::THEME_CHANGE;
        }
        IGuiEvent::DpiChange {
            child_id,
            dpi_x,
            dpi_y,
        } => {
            k = kind::DPI_CHANGE;
            child = child_id;
            p1 = dpi_x;
            p2 = dpi_y;
        }
        IGuiEvent::Menu { menu_id, item_id } => {
            k = kind::MENU;
            p1 = menu_id;
            p2 = item_id;
        }
    }

    unsafe {
        if !out_kind.is_null() {
            *out_kind = k;
        }
        if !out_child.is_null() {
            *out_child = child;
        }
        if !out_time.is_null() {
            *out_time = t;
        }
        if !out_p1.is_null() {
            *out_p1 = p1;
        }
        if !out_p2.is_null() {
            *out_p2 = p2;
        }
        if !out_p3.is_null() {
            *out_p3 = p3;
        }
        if !out_p4.is_null() {
            *out_p4 = p4;
        }
    }
}

pub fn native_module_artifact() -> NativeModuleArtifact {
    NativeModuleArtifact::new(
        HostedModuleArtifact::new(
            "iGui",
            vec![],
            ExportDirectory::new(vec![
                ExportEntry::procedure("NextEvent"),
                ExportEntry::procedure("Quit"),
                ExportEntry::procedure("OpenChild"),
                ExportEntry::procedure("CloseChild"),
                ExportEntry::procedure("SetTitle"),
            ]),
            "iGui.bootstrap",
            "Integrated GUI: MDI frame, Direct2D surfaces, typed event mailbox",
            vec![],
        ),
        vec![
            NativeExportBinding::procedure("NextEvent", igui_next_event as *const () as usize),
            NativeExportBinding::procedure("Quit", igui_quit as *const () as usize),
            NativeExportBinding::procedure("OpenChild", igui_open_child as *const () as usize),
            NativeExportBinding::procedure("CloseChild", igui_close_child as *const () as usize),
            NativeExportBinding::procedure("SetTitle", igui_set_title as *const () as usize),
        ],
    )
}
