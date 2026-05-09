//! MDI frame window, MDI client child, message pump, and the
//! cross-thread helpers used by `iGui.OpenChild` / `CloseChild` /
//! `SetTitle`.
//!
//! Window-creation operations issued by the language thread are
//! marshalled to the GUI thread via private `WM_USER` messages and
//! `SendMessageW`, which blocks until the WndProc returns. This
//! preserves the iGui rule that all HWND ownership lives on the GUI
//! thread without forcing a typed RPC between the two.

#![cfg(windows)]

use std::ptr;
use std::sync::OnceLock;
use std::sync::Mutex;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_CAPITAL, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefFrameProcW, DispatchMessageW, GetClientRect, GetMessageTime,
    GetMessageW, LoadCursorW, PostQuitMessage, RegisterClassExW, SendMessageW, ShowWindow,
    TranslateMessage, CLIENTCREATESTRUCT, CW_USEDEFAULT, IDC_ARROW, MDICREATESTRUCTW, MSG,
    SW_SHOW, WHEEL_DELTA, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP,
    WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MDICREATE,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETFOCUS, WM_SIZE,
    WM_SYSCOLORCHANGE, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_THEMECHANGED, WM_USER,
    WNDCLASSEXW, WNDCLASS_STYLES, WS_CHILD,
    WS_CLIPCHILDREN, WS_EX_APPWINDOW, WS_HSCROLL, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};

use super::channels::{self, modifier, mouse_op, IGuiEvent};
use super::child::{self, MdiBootstrap, MDI_CHILD_CLASS};
use super::cp_exports::FRAME_HWND;
use super::registry;
use super::renderer;
use super::IGuiError;

const FRAME_CHILD_ID: i64 = 1;
const FRAME_CLASS: PCWSTR = w!("NewCP.iGui.Frame");

// Private messages used to marshal language-thread calls onto the GUI
// thread. lparam is the address of the corresponding *Request struct,
// which the WndProc reads, mutates, and returns 0; the SendMessageW
// caller reads its own request struct on return.
const WM_IGUI_OPEN_CHILD: u32 = WM_USER + 1;
const WM_IGUI_CLOSE_CHILD: u32 = WM_USER + 2;
const WM_IGUI_SET_TITLE: u32 = WM_USER + 3;

/// HWND of the MDI client. Set by `run` after `CreateWindowExW`.
static MDI_CLIENT: Mutex<Option<isize>> = Mutex::new(None);
static GUI_THREAD_ID: OnceLock<u32> = OnceLock::new();

fn mdi_client_hwnd() -> Option<HWND> {
    let raw = MDI_CLIENT.lock().ok()?;
    raw.map(|r| HWND(r as *mut _))
}

pub(crate) fn gui_thread_id() -> Option<u32> {
    GUI_THREAD_ID.get().copied()
}

/// Public entry point. Opens the iGui frame, sets up the MDI client,
/// runs the Win32 message pump until `WM_QUIT`, and returns the quit
/// code. If `worker` is provided, it is spawned on a background
/// thread once the frame is up.
pub fn run<F>(worker: Option<F>) -> Result<i32, IGuiError>
where
    F: FnOnce() + Send + 'static,
{
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let _ = GUI_THREAD_ID.set(unsafe { GetCurrentThreadId() });

    let h_instance = unsafe { GetModuleHandleW(None) }
        .map_err(|e| IGuiError::Win32(format!("GetModuleHandleW failed: {e}")))?
        .into();
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) }
        .map_err(|e| IGuiError::Win32(format!("LoadCursorW failed: {e}")))?;

    // Frame class.
    let frame_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(frame_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: Default::default(),
        hCursor: cursor,
        hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(ptr::null_mut()),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: FRAME_CLASS,
        hIconSm: Default::default(),
    };
    if unsafe { RegisterClassExW(&frame_class) } == 0 {
        return Err(IGuiError::Win32("RegisterClassExW (frame) returned 0".into()));
    }
    child::register_classes()?;

    // Renderer comes up before any window so child WM_NCCREATE can build
    // its swap chain immediately.
    renderer::install()?;

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_APPWINDOW,
            FRAME_CLASS,
            w!("NewCP — iGui"),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1024,
            720,
            None,
            None,
            Some(h_instance),
            None,
        )
    }
    .map_err(|e| IGuiError::Win32(format!("CreateWindowExW (frame) failed: {e}")))?;
    let _ = FRAME_HWND.set(hwnd.0 as isize);

    // MDI client occupies the whole frame body for now (no toolbar /
    // status bar yet).
    let mut frame_rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut frame_rect) }
        .map_err(|e| IGuiError::Win32(format!("GetClientRect (frame) failed: {e}")))?;
    let mut create = CLIENTCREATESTRUCT {
        hWindowMenu: Default::default(),
        idFirstChild: 0xCC00,
    };
    let mdi = unsafe {
        CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
            w!("MDICLIENT"),
            PCWSTR::null(),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_HSCROLL | WS_VSCROLL,
            0,
            0,
            frame_rect.right - frame_rect.left,
            frame_rect.bottom - frame_rect.top,
            Some(hwnd),
            None,
            Some(h_instance),
            Some(&mut create as *mut _ as *mut _),
        )
    }
    .map_err(|e| IGuiError::Win32(format!("CreateWindowExW (MDICLIENT) failed: {e}")))?;
    {
        let mut slot = MDI_CLIENT.lock().expect("MDI_CLIENT mutex poisoned");
        *slot = Some(mdi.0 as isize);
    }

    channels::install();
    super::system_colors::sample();

    let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };

    if let Some(worker) = worker {
        std::thread::Builder::new()
            .name("igui-language".into())
            .spawn(worker)
            .map_err(|e| IGuiError::Win32(format!("spawn language thread: {e}")))?;
    }

    let mut msg = MSG::default();
    let exit_code = unsafe {
        loop {
            let r = GetMessageW(&mut msg, None, 0, 0);
            if r.0 == 0 {
                break msg.wParam.0 as i32;
            }
            if r.0 == -1 {
                break 1;
            }
            // MDI requires TranslateMDISysAccel before TranslateMessage,
            // but for Phase 3a (no menus, no system accelerators) we
            // skip it. Add when menus arrive in Phase 6.
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    };

    Ok(exit_code)
}

unsafe extern "system" fn frame_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mdi = mdi_client_hwnd().unwrap_or_default();

    match msg {
        WM_IGUI_OPEN_CHILD => {
            let req_ptr = lparam.0 as *mut OpenChildRequest;
            if !req_ptr.is_null() {
                let req = unsafe { &mut *req_ptr };
                req.out = handle_open_child(req);
            }
            LRESULT(0)
        }
        WM_IGUI_CLOSE_CHILD => {
            let req_ptr = lparam.0 as *mut CloseChildRequest;
            if !req_ptr.is_null() {
                let req = unsafe { &mut *req_ptr };
                if let Some(mdi_child) = registry::mdi_hwnd_of(req.child_id) {
                    if mdi.0 as isize != 0 {
                        child::close_via_mdi(mdi, mdi_child);
                        req.ok = true;
                    }
                }
            }
            LRESULT(0)
        }
        WM_IGUI_SET_TITLE => {
            let req_ptr = lparam.0 as *mut SetTitleRequest;
            if !req_ptr.is_null() {
                let req = unsafe { &*req_ptr };
                if let Some(mdi_child) = registry::mdi_hwnd_of(req.child_id) {
                    child::set_title(mdi_child, &req.title);
                }
            }
            LRESULT(0)
        }
        WM_SIZE => {
            // MDI client sizes itself via DefFrameProcW.
            channels::push(IGuiEvent::Resize {
                child_id: FRAME_CHILD_ID,
                width: (lparam.0 & 0xFFFF) as i64,
                height: ((lparam.0 >> 16) & 0xFFFF) as i64,
            });
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            push_key(true, wparam, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_KEYUP | WM_SYSKEYUP => {
            push_key(false, wparam, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_CHAR => {
            channels::push(IGuiEvent::Char {
                child_id: FRAME_CHILD_ID,
                codepoint: wparam.0 as i64,
                mods: current_modifiers(),
                time_ms: msg_time(),
            });
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_MOUSEMOVE => {
            push_mouse(FRAME_CHILD_ID, mouse_op::MOVE, 0, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_LBUTTONDOWN => {
            push_mouse(FRAME_CHILD_ID, mouse_op::LEFT_DOWN, 1, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_LBUTTONUP => {
            push_mouse(FRAME_CHILD_ID, mouse_op::LEFT_UP, 1, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_RBUTTONDOWN => {
            push_mouse(FRAME_CHILD_ID, mouse_op::RIGHT_DOWN, 2, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_RBUTTONUP => {
            push_mouse(FRAME_CHILD_ID, mouse_op::RIGHT_UP, 2, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_MBUTTONDOWN => {
            push_mouse(FRAME_CHILD_ID, mouse_op::MIDDLE_DOWN, 3, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_MBUTTONUP => {
            push_mouse(FRAME_CHILD_ID, mouse_op::MIDDLE_UP, 3, lparam);
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_MOUSEWHEEL => {
            let raw = ((wparam.0 >> 16) & 0xFFFF) as i16;
            let delta = raw as i64;
            let lines = if WHEEL_DELTA != 0 {
                delta / (WHEEL_DELTA as i64)
            } else {
                0
            };
            channels::push(IGuiEvent::Mouse {
                child_id: FRAME_CHILD_ID,
                x: (lparam.0 & 0xFFFF) as i16 as i64,
                y: ((lparam.0 >> 16) & 0xFFFF) as i16 as i64,
                op: mouse_op::WHEEL,
                button: 0,
                mods: current_modifiers(),
                wheel_delta: delta,
                wheel_lines: lines,
                time_ms: msg_time(),
            });
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_SETFOCUS => {
            channels::push(IGuiEvent::Focus {
                child_id: FRAME_CHILD_ID,
                gained: true,
            });
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_KILLFOCUS => {
            channels::push(IGuiEvent::Focus {
                child_id: FRAME_CHILD_ID,
                gained: false,
            });
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_SYSCOLORCHANGE | WM_THEMECHANGED => {
            super::system_colors::refresh_and_notify();
            unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) }
        }
        WM_CLOSE => {
            channels::push(IGuiEvent::FrameClose);
            // Close every registered MDI child, then destroy the frame.
            if mdi.0 as isize != 0 {
                for (_id, child_hwnd) in registry::snapshot() {
                    child::close_via_mdi(mdi, child_hwnd);
                }
            }
            let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefFrameProcW(hwnd, Some(mdi), msg, wparam, lparam) },
    }
}

fn handle_open_child(req: &OpenChildRequest) -> Option<i64> {
    let mdi = mdi_client_hwnd()?;
    let child_id = registry::allocate_child_id();
    let bootstrap = Box::into_raw(Box::new(MdiBootstrap { child_id }));
    let h_module = unsafe { GetModuleHandleW(None) }.ok()?;
    let h_owner = windows::Win32::Foundation::HANDLE(h_module.0);

    let mdi_create = MDICREATESTRUCTW {
        szClass: MDI_CHILD_CLASS,
        szTitle: PCWSTR::from_raw(req.title.as_ptr()),
        hOwner: h_owner,
        x: CW_USEDEFAULT,
        y: CW_USEDEFAULT,
        cx: CW_USEDEFAULT,
        cy: CW_USEDEFAULT,
        style: WS_VISIBLE | WS_OVERLAPPEDWINDOW,
        lParam: LPARAM(bootstrap as isize),
    };
    let result = unsafe {
        SendMessageW(
            mdi,
            WM_MDICREATE,
            Some(WPARAM(0)),
            Some(LPARAM(&mdi_create as *const _ as isize)),
        )
    };
    let new_hwnd = HWND(result.0 as *mut _);
    if new_hwnd.0.is_null() {
        // WM_MDICREATE failed; reclaim the bootstrap to avoid leaking.
        let _ = unsafe { Box::from_raw(bootstrap) };
        return None;
    }
    Some(child_id)
}

fn msg_time() -> i64 {
    unsafe { GetMessageTime() as i64 }
}

fn current_modifiers() -> i64 {
    let mut m = 0i64;
    unsafe {
        if (GetKeyState(VK_SHIFT.0 as i32) as i16) < 0 {
            m |= modifier::SHIFT;
        }
        if (GetKeyState(VK_CONTROL.0 as i32) as i16) < 0 {
            m |= modifier::CONTROL;
        }
        if (GetKeyState(VK_MENU.0 as i32) as i16) < 0 {
            m |= modifier::ALT;
        }
        if (GetKeyState(VK_LWIN.0 as i32) as i16) < 0
            || (GetKeyState(VK_RWIN.0 as i32) as i16) < 0
        {
            m |= modifier::WIN;
        }
        if (GetKeyState(VK_CAPITAL.0 as i32) & 1) != 0 {
            m |= modifier::CAPS;
        }
    }
    m
}

fn push_key(down: bool, wparam: WPARAM, lparam: LPARAM) {
    let scancode = ((lparam.0 >> 16) & 0xFF) as i64;
    let repeat = (lparam.0 & 0xFFFF) as i64;
    channels::push(IGuiEvent::Key {
        child_id: FRAME_CHILD_ID,
        vkey: wparam.0 as i64,
        scancode,
        mods: current_modifiers(),
        repeat,
        down,
        time_ms: msg_time(),
    });
}

fn push_mouse(child_id: i64, op: i64, button: i64, lparam: LPARAM) {
    let x = (lparam.0 & 0xFFFF) as i16 as i64;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i64;
    channels::push(IGuiEvent::Mouse {
        child_id,
        x,
        y,
        op,
        button,
        mods: current_modifiers(),
        wheel_delta: 0,
        wheel_lines: 0,
        time_ms: msg_time(),
    });
}

// ─── Cross-thread request structures ─────────────────────────────────

pub(crate) struct OpenChildRequest {
    pub title: Vec<u16>,
    pub out: Option<i64>,
}

pub(crate) struct CloseChildRequest {
    pub child_id: i64,
    pub ok: bool,
}

pub(crate) struct SetTitleRequest {
    pub child_id: i64,
    pub title: Vec<u16>,
}

/// Called from the language thread. Marshals to the GUI thread via
/// SendMessageW; blocks until the child has been created.
pub fn open_child(title: &str) -> Option<i64> {
    let frame_raw = *FRAME_HWND.get()?;
    let frame = HWND(frame_raw as *mut _);
    let mut title_w: Vec<u16> = title.encode_utf16().collect();
    title_w.push(0);
    let mut req = OpenChildRequest {
        title: title_w,
        out: None,
    };
    unsafe {
        SendMessageW(
            frame,
            WM_IGUI_OPEN_CHILD,
            Some(WPARAM(0)),
            Some(LPARAM(&mut req as *mut _ as isize)),
        )
    };
    req.out
}

pub fn close_child(child_id: i64) -> bool {
    let Some(frame_raw) = FRAME_HWND.get() else {
        return false;
    };
    let frame = HWND(*frame_raw as *mut _);
    let mut req = CloseChildRequest {
        child_id,
        ok: false,
    };
    unsafe {
        SendMessageW(
            frame,
            WM_IGUI_CLOSE_CHILD,
            Some(WPARAM(0)),
            Some(LPARAM(&mut req as *mut _ as isize)),
        )
    };
    req.ok
}

pub fn set_child_title(child_id: i64, title: &str) {
    let Some(frame_raw) = FRAME_HWND.get() else {
        return;
    };
    let frame = HWND(*frame_raw as *mut _);
    let mut title_w: Vec<u16> = title.encode_utf16().collect();
    title_w.push(0);
    let req = SetTitleRequest {
        child_id,
        title: title_w,
    };
    unsafe {
        SendMessageW(
            frame,
            WM_IGUI_SET_TITLE,
            Some(WPARAM(0)),
            Some(LPARAM(&req as *const _ as isize)),
        )
    };
}

