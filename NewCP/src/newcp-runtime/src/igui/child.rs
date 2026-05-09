//! MDI child windows. Each child owns its own swap chain and Direct2D
//! bitmap target; rendering is per-child on `WM_PAINT`.
//!
//! Phase 3a paints a deterministic per-child background color so two
//! children in one frame are visibly distinct without any CP-side
//! batches yet. Phase 3b will replace the per-child color with a
//! drained `PaneBatch`.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::IDXGISwapChain1;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, HBRUSH, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    DefMDIChildProcW, GetClientRect, GetWindowLongPtrW, LoadCursorW, RegisterClassExW,
    SendMessageW, SetWindowLongPtrW, GWLP_USERDATA, IDC_ARROW, MDICREATESTRUCTW, WM_MDIDESTROY,
    WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WM_SETTEXT, WM_SIZE, WNDCLASSEXW, WNDCLASS_STYLES,
};

use super::channels::{self, IGuiEvent};
use super::d2d::SwapChainTarget;
use super::d3d::present;
use super::registry;
use super::renderer;
use super::IGuiError;

/// The Win32 class name iGui registers for MDI children.
pub(crate) const CHILD_CLASS: PCWSTR = w!("NewCP.iGui.Child");

/// Per-child renderer state, stored in `GWLP_USERDATA` of the child
/// HWND. Created when the child window arrives in `WM_NCCREATE` (via
/// the `MDICREATESTRUCT::lParam`) and dropped in `WM_NCDESTROY`.
pub(crate) struct ChildState {
    pub(crate) child_id: i64,
    pub(crate) swap_chain: IDXGISwapChain1,
    pub(crate) target: RefCell<Option<SwapChainTarget>>,
}

impl ChildState {
    fn render(&self) -> Result<(), IGuiError> {
        let r = renderer::ctx();
        let mut target_slot = self.target.borrow_mut();
        if target_slot.is_none() {
            *target_slot = Some(SwapChainTarget::new(&r.d2d, &self.swap_chain)?);
        }
        let target = target_slot.as_ref().unwrap();
        target.bind(&r.d2d);

        let color = phase3a_palette(self.child_id);
        unsafe {
            r.d2d.context.BeginDraw();
            r.d2d.context.Clear(Some(&D2D1_COLOR_F {
                r: color[0],
                g: color[1],
                b: color[2],
                a: 1.0,
            }));
            let mut tag1 = 0u64;
            let mut tag2 = 0u64;
            r.d2d
                .context
                .EndDraw(Some(&mut tag1), Some(&mut tag2))
                .map_err(|e| IGuiError::D2D(format!("child EndDraw failed: {e}")))?;
        }

        present(&self.swap_chain)?;
        Ok(())
    }

    fn handle_resize(&self, width: u32, height: u32) -> Result<(), IGuiError> {
        let r = renderer::ctx();
        SwapChainTarget::unbind(&r.d2d);
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
        .map_err(|e| IGuiError::D3D(format!("child ResizeBuffers failed: {e}")))?;
        Ok(())
    }
}

/// Deterministic per-child background. Picks a slate-with-tint based
/// on `child_id` so two simultaneously-open children are visually
/// distinct without any CP-side batches yet.
fn phase3a_palette(child_id: i64) -> [f32; 3] {
    let palette: [[f32; 3]; 6] = [
        [0.18, 0.20, 0.23],
        [0.22, 0.18, 0.20],
        [0.18, 0.23, 0.20],
        [0.20, 0.18, 0.23],
        [0.23, 0.22, 0.18],
        [0.18, 0.22, 0.23],
    ];
    palette[((child_id as usize).saturating_sub(2)) % palette.len()]
}

/// Register the child window class. Idempotent — Win32 returns 0 with
/// `ERROR_CLASS_ALREADY_EXISTS` on the second call which we treat as
/// success.
pub fn register_class() -> Result<(), IGuiError> {
    let h_instance = unsafe { GetModuleHandleW(None) }
        .map_err(|e| IGuiError::Win32(format!("GetModuleHandleW (child): {e}")))?
        .into();
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) }
        .map_err(|e| IGuiError::Win32(format!("LoadCursorW (child): {e}")))?;

    let class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(child_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: Default::default(),
        hCursor: cursor,
        hbrBackground: HBRUSH(std::ptr::null_mut()),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: CHILD_CLASS,
        hIconSm: Default::default(),
    };
    let _atom = unsafe { RegisterClassExW(&class) };
    // Non-zero atom = success; zero with ERROR_CLASS_ALREADY_EXISTS is also
    // fine because we may register the class on every iGui session and the
    // first session leaves the class around for later ones.
    Ok(())
}

unsafe extern "system" fn child_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCREATE {
        // The MDI client passes our `CREATESTRUCTW.lpCreateParams`,
        // which we set to a Box<ChildBootstrap>, through lparam.
        let create = lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
        let mdi_create =
            unsafe { (*create).lpCreateParams as *const MDICREATESTRUCTW };
        if !mdi_create.is_null() {
            let bootstrap = unsafe { (*mdi_create).lParam.0 as *mut ChildBootstrap };
            if !bootstrap.is_null() {
                let bootstrap = unsafe { Box::from_raw(bootstrap) };
                match build_child_state(hwnd, *bootstrap) {
                    Ok(state) => {
                        let raw = Box::into_raw(state);
                        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw as isize) };
                    }
                    Err(err) => {
                        eprintln!("[igui-child] init failed: {err}");
                        return LRESULT(0); // abort window creation
                    }
                }
            }
        }
        return unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) };
    }

    let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut ChildState;
    let state: Option<&ChildState> = if raw.is_null() {
        None
    } else {
        Some(unsafe { &*raw })
    };

    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _ = unsafe { BeginPaint(hwnd, &mut ps) };
            if let Some(state) = state {
                if let Err(err) = state.render() {
                    eprintln!("[igui-child] render error: {err}");
                }
            }
            let _ = unsafe { EndPaint(hwnd, &ps) };
            LRESULT(0)
        }
        WM_SIZE => {
            if let Some(state) = state {
                let w = (lparam.0 & 0xFFFF) as u32;
                let h = ((lparam.0 >> 16) & 0xFFFF) as u32;
                if let Err(err) = state.handle_resize(w, h) {
                    eprintln!("[igui-child] resize error: {err}");
                }
                channels::push(IGuiEvent::Resize {
                    child_id: state.child_id,
                    width: w as i64,
                    height: h as i64,
                });
            }
            unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) }
        }
        WM_NCDESTROY => {
            if let Some(state) = state {
                channels::push(IGuiEvent::Close {
                    child_id: state.child_id,
                });
                registry::unregister(state.child_id);
            }
            if !raw.is_null() {
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
                let _ = unsafe { Box::from_raw(raw) };
            }
            unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) }
        }
        _ => unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) },
    }
}

/// Bag of inputs needed to build `ChildState` once Win32 has given us
/// the HWND. Threaded through `MDICREATESTRUCTW::lParam`.
pub(crate) struct ChildBootstrap {
    pub(crate) child_id: i64,
}

fn build_child_state(hwnd: HWND, bootstrap: ChildBootstrap) -> Result<Box<ChildState>, IGuiError> {
    let r = renderer::ctx();
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) }
        .map_err(|e| IGuiError::Win32(format!("child GetClientRect failed: {e}")))?;
    let width = (rect.right - rect.left).max(1) as u32;
    let height = (rect.bottom - rect.top).max(1) as u32;
    let swap_chain = r.d3d.create_swap_chain_for_hwnd(hwnd, width, height)?;
    registry::register(bootstrap.child_id, hwnd);
    Ok(Box::new(ChildState {
        child_id: bootstrap.child_id,
        swap_chain,
        target: RefCell::new(None),
    }))
}

/// Decode UTF-16 (no NUL) → owned String. Used by debug logging only.
#[allow(dead_code)]
pub(crate) fn decode_utf16(buf: &[u16]) -> String {
    let trimmed: Vec<u16> = buf.iter().copied().take_while(|c| *c != 0).collect();
    OsString::from_wide(&trimmed)
        .into_string()
        .unwrap_or_default()
}

/// Send `WM_SETTEXT` to the child to update its title bar. Safe to
/// call from any thread; SendMessageW marshals to the GUI thread.
pub fn set_title(hwnd: HWND, title_w: &[u16]) {
    unsafe {
        SendMessageW(
            hwnd,
            WM_SETTEXT,
            Some(WPARAM(0)),
            Some(LPARAM(title_w.as_ptr() as isize)),
        )
    };
}

/// Ask the MDI client to destroy a child via `WM_MDIDESTROY`.
pub fn close_via_mdi(mdi_client: HWND, child: HWND) {
    unsafe {
        SendMessageW(
            mdi_client,
            WM_MDIDESTROY,
            Some(WPARAM(child.0 as usize)),
            Some(LPARAM(0)),
        )
    };
}
