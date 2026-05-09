//! Frame window class, message pump, and Phase 1 paint loop.
//!
//! Phase 1 deliberately ships a single top-level frame window (no MDI
//! client window yet) so the bring-up has fewer moving parts. The MDI
//! client child window arrives in Phase 3 alongside child documents and
//! the surface command queue, since that is when it has anything to
//! coordinate.

#![cfg(windows)]

use std::cell::RefCell;
use std::ptr;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::IDXGISwapChain1;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, UpdateWindow, HBRUSH, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT, VK_CAPITAL};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageTime,
    GetMessageW, GetWindowLongPtrW, LoadCursorW, PostQuitMessage, RegisterClassExW,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, CREATESTRUCTW, CW_USEDEFAULT,
    GWLP_USERDATA, IDC_ARROW, MSG, SW_SHOW, WHEEL_DELTA, WM_CHAR, WM_CLOSE, WM_DESTROY,
    WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
    WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE, WM_PAINT, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_SETFOCUS, WM_SIZE, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW,
    WNDCLASS_STYLES, WS_EX_APPWINDOW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use super::channels::{self, modifier, mouse_op, IGuiEvent};
use super::cp_exports::FRAME_HWND;
use super::d2d::{D2dContext, SwapChainTarget};
use super::d3d::{present, D3dContext};
use super::dwrite::DWriteContext;
use super::{IGuiError, PHASE1_BACKGROUND};

/// Frame window's child id is fixed at 1; child documents (Phase 3+)
/// use higher ids.
const FRAME_CHILD_ID: i64 = 1;

/// State stored as a Box behind the frame HWND's `GWLP_USERDATA`. The
/// WndProc reads this pointer on every message; all renderer access
/// goes through the borrow.
#[allow(dead_code)] // d3d held for lifetime; queried in later phases
struct FrameState {
    d3d: D3dContext,
    d2d: D2dContext,
    dwrite: DWriteContext,
    swap_chain: IDXGISwapChain1,
    target: RefCell<Option<SwapChainTarget>>,
}

impl FrameState {
    fn render(&self, hwnd: HWND) -> Result<(), IGuiError> {
        // Lazily (re)create the bitmap target after a resize.
        let mut target_slot = self.target.borrow_mut();
        if target_slot.is_none() {
            *target_slot = Some(SwapChainTarget::new(&self.d2d, &self.swap_chain)?);
        }
        let target = target_slot.as_ref().unwrap();
        target.bind(&self.d2d);

        unsafe {
            self.d2d.context.BeginDraw();
            self.d2d.context.Clear(Some(&D2D1_COLOR_F {
                r: PHASE1_BACKGROUND[0],
                g: PHASE1_BACKGROUND[1],
                b: PHASE1_BACKGROUND[2],
                a: PHASE1_BACKGROUND[3],
            }));
            // EndDraw returns a tag pair we ignore in Phase 1.
            let mut tag1 = 0u64;
            let mut tag2 = 0u64;
            self.d2d
                .context
                .EndDraw(Some(&mut tag1), Some(&mut tag2))
                .map_err(|e| IGuiError::D2D(format!("EndDraw failed: {e}")))?;
        }

        present(&self.swap_chain)?;
        let _ = hwnd;
        Ok(())
    }

    fn handle_resize(&self, width: u32, height: u32) -> Result<(), IGuiError> {
        // Drop the bitmap target before resizing the swap chain — DXGI
        // requires no outstanding references to its back buffer.
        SwapChainTarget::unbind(&self.d2d);
        self.target.borrow_mut().take();

        unsafe {
            self.swap_chain.ResizeBuffers(
                0,
                width.max(1),
                height.max(1),
                DXGI_FORMAT_B8G8R8A8_UNORM,
                windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG(0),
            )
        }
        .map_err(|e| IGuiError::D3D(format!("ResizeBuffers failed: {e}")))?;
        // Target bitmap is recreated on the next render call.
        Ok(())
    }
}

const FRAME_CLASS: PCWSTR = w!("NewCP.iGui.Frame");

/// Public entry point. Opens the iGui frame, runs the Win32 message
/// pump until `WM_QUIT`, and returns the quit code. If `worker` is
/// provided, it is spawned on a background thread once the frame is
/// up so the language thread can call `iGui.NextEvent` against the
/// mailbox while the message pump runs here.
pub fn run<F>(worker: Option<F>) -> Result<i32, IGuiError>
where
    F: FnOnce() + Send + 'static,
{
    // Per-monitor DPI awareness — required for crisp rendering across
    // multi-monitor setups. Failure is non-fatal (older Windows might
    // not support V2); we fall through and accept the default.
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let h_instance = unsafe { GetModuleHandleW(None) }
        .map_err(|e| IGuiError::Win32(format!("GetModuleHandleW failed: {e}")))?
        .into();

    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) }
        .map_err(|e| IGuiError::Win32(format!("LoadCursorW failed: {e}")))?;

    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(frame_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: Default::default(),
        hCursor: cursor,
        hbrBackground: HBRUSH(ptr::null_mut()),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: FRAME_CLASS,
        hIconSm: Default::default(),
    };
    let atom = unsafe { RegisterClassExW(&class) };
    if atom == 0 {
        return Err(IGuiError::Win32("RegisterClassExW returned 0".into()));
    }

    // Renderer plumbing comes up before the window so we can stash the
    // state pointer in WM_NCCREATE.
    let d3d = D3dContext::new()?;
    let d2d = D2dContext::new(&d3d)?;
    let dwrite = DWriteContext::new()?;

    // Create the frame.
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_APPWINDOW,
            FRAME_CLASS,
            w!("NewCP — iGui"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
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
    .map_err(|e| IGuiError::Win32(format!("CreateWindowExW failed: {e}")))?;

    // Build the swap chain bound to the frame's client area.
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) }
        .map_err(|e| IGuiError::Win32(format!("GetClientRect failed: {e}")))?;
    let width = (rect.right - rect.left).max(1) as u32;
    let height = (rect.bottom - rect.top).max(1) as u32;
    let swap_chain = d3d.create_swap_chain_for_hwnd(hwnd, width, height)?;

    let state = Box::new(FrameState {
        d3d,
        d2d,
        dwrite,
        swap_chain,
        target: RefCell::new(None),
    });
    let state_ptr = Box::into_raw(state);
    unsafe {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
    }

    // Install the event mailbox before the worker thread is started so
    // any early `iGui.NextEvent` calls find the channel ready.
    channels::install();
    let _ = FRAME_HWND.set(hwnd.0 as isize);

    let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };
    let _ = unsafe { UpdateWindow(hwnd) };

    // Spawn the language-thread worker if the caller provided one.
    if let Some(worker) = worker {
        std::thread::Builder::new()
            .name("igui-language".into())
            .spawn(worker)
            .map_err(|e| IGuiError::Win32(format!("spawn language thread: {e}")))?;
    }

    // Pump.
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
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    };

    // Tear down state. The HWND is already gone by the time we get
    // WM_QUIT, so the userdata pointer might be stale; only consume it
    // if we still own it.
    if !state_ptr.is_null() {
        let _ = unsafe { Box::from_raw(state_ptr) };
    }

    Ok(exit_code)
}

unsafe extern "system" fn frame_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // WM_NCCREATE arrives before our SetWindowLongPtrW above, so the
    // userdata is null then; default-handle and bail.
    if msg == WM_NCCREATE {
        let _ = lparam.0 as *const CREATESTRUCTW;
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut FrameState;
    let state: Option<&FrameState> = if raw.is_null() {
        None
    } else {
        Some(unsafe { &*raw })
    };

    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _hdc = unsafe { BeginPaint(hwnd, &mut ps) };
            if let Some(state) = state {
                if let Err(err) = state.render(hwnd) {
                    eprintln!("[igui] render error: {err}");
                }
            }
            let _ = unsafe { EndPaint(hwnd, &ps) };
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (lparam.0 & 0xFFFF) as i64;
            let height = ((lparam.0 >> 16) & 0xFFFF) as i64;
            if let Some(state) = state {
                if let Err(err) = state.handle_resize(width as u32, height as u32) {
                    eprintln!("[igui] resize error: {err}");
                }
            }
            channels::push(IGuiEvent::Resize {
                child_id: FRAME_CHILD_ID,
                width,
                height,
            });
            LRESULT(0)
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            push_key(true, wparam, lparam);
            LRESULT(0)
        }
        WM_KEYUP | WM_SYSKEYUP => {
            push_key(false, wparam, lparam);
            LRESULT(0)
        }
        WM_CHAR => {
            channels::push(IGuiEvent::Char {
                child_id: FRAME_CHILD_ID,
                codepoint: wparam.0 as i64,
                mods: current_modifiers(),
                time_ms: msg_time(),
            });
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            push_mouse(mouse_op::MOVE, 0, wparam, lparam);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            push_mouse(mouse_op::LEFT_DOWN, 1, wparam, lparam);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            push_mouse(mouse_op::LEFT_UP, 1, wparam, lparam);
            LRESULT(0)
        }
        WM_RBUTTONDOWN => {
            push_mouse(mouse_op::RIGHT_DOWN, 2, wparam, lparam);
            LRESULT(0)
        }
        WM_RBUTTONUP => {
            push_mouse(mouse_op::RIGHT_UP, 2, wparam, lparam);
            LRESULT(0)
        }
        WM_MBUTTONDOWN => {
            push_mouse(mouse_op::MIDDLE_DOWN, 3, wparam, lparam);
            LRESULT(0)
        }
        WM_MBUTTONUP => {
            push_mouse(mouse_op::MIDDLE_UP, 3, wparam, lparam);
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            // GET_WHEEL_DELTA_WPARAM: high-order signed short of wparam.
            let raw = ((wparam.0 >> 16) & 0xFFFF) as i16;
            let delta = raw as i64;
            let lines = if WHEEL_DELTA != 0 {
                delta / (WHEEL_DELTA as i64)
            } else {
                0
            };
            // Mouse wheel uses screen coordinates in lparam; translate
            // is omitted in Phase 2 — the language thread can ignore
            // x/y for wheel.
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
            LRESULT(0)
        }
        WM_SETFOCUS => {
            channels::push(IGuiEvent::Focus {
                child_id: FRAME_CHILD_ID,
                gained: true,
            });
            LRESULT(0)
        }
        WM_KILLFOCUS => {
            channels::push(IGuiEvent::Focus {
                child_id: FRAME_CHILD_ID,
                gained: false,
            });
            LRESULT(0)
        }
        WM_CLOSE => {
            channels::push(IGuiEvent::FrameClose);
            // Drop the userdata box before the HWND is destroyed.
            if !raw.is_null() {
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
                let _ = unsafe { Box::from_raw(raw) };
            }
            let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
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

fn push_mouse(op: i64, button: i64, wparam: WPARAM, lparam: LPARAM) {
    let x = (lparam.0 & 0xFFFF) as i16 as i64;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i64;
    // wparam carries pressed buttons + mods (MK_*). For Phase 2 we
    // ignore the MK_ bits and re-read modifiers from GetKeyState so
    // every event has a consistent shape.
    let _ = wparam;
    channels::push(IGuiEvent::Mouse {
        child_id: FRAME_CHILD_ID,
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
