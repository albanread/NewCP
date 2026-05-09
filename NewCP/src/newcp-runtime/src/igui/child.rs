//! MDI child windows + their render-host children.
//!
//! Architecture: each "document" is two HWNDs:
//!
//! 1. **MDI child** — the document window the user sees. Owns the
//!    title bar, MDI activate/close behavior, and lifetime. Created
//!    inside the MDI client via `WM_MDICREATE`. Style:
//!    `WS_OVERLAPPEDWINDOW | WS_VISIBLE`.
//!
//! 2. **Render host** — a borderless `WS_CHILD | WS_VISIBLE |
//!    WS_CLIPSIBLINGS` window inside the MDI child's client area.
//!    Owns the WM_PAINT loop and the active Phase 3b renderer.
//!
//! The current renderer prefers a per-window Direct2D HWND render target.
//! The older GDI path remains as a fallback because it was the first path
//! that produced visible pixels during bring-up and is still useful when
//! diagnosing target-creation or EndDraw failures.

#![cfg(windows)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::core::{w, Error, PCWSTR};
use windows_numerics::Vector2;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D_SIZE_U, D2D1_ALPHA_MODE_IGNORE, D2D1_COLOR_F, D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    ID2D1HwndRenderTarget, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_HWND_RENDER_TARGET_PROPERTIES,
    D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
    D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
};
use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreatePen, CreateSolidBrush, DeleteObject, EndPaint, FillRect, FrameRect,
    HBRUSH, LineTo, MoveToEx, PAINTSTRUCT, PS_SOLID, RoundRect, SelectObject,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefMDIChildProcW, DefWindowProcW, GetClientRect, GetParent,
    GetWindowLongPtrW, IsWindow, IsWindowVisible, LoadCursorW, RegisterClassExW, SendMessageW,
    SetWindowLongPtrW, SetWindowPos, CREATESTRUCTW, GWLP_USERDATA, IDC_ARROW,
    MDICREATESTRUCTW, SWP_NOACTIVATE, SWP_NOZORDER, WINDOW_EX_STYLE, WM_ERASEBKGND,
    WM_MDIDESTROY, WM_NCCREATE, WM_NCDESTROY, WM_PAINT, WM_SETTEXT, WM_SIZE, WNDCLASSEXW,
    WNDCLASS_STYLES, WS_CHILD, WS_CLIPSIBLINGS, WS_VISIBLE,
};

use super::batch as batch_mod;
use super::batch::SurfaceCmd;
use super::channels::{self, IGuiEvent};
use super::registry;
use super::renderer;
use super::window;
use super::IGuiError;

pub(crate) const MDI_CHILD_CLASS: PCWSTR = w!("NewCP.iGui.Child");
pub(crate) const RENDER_HOST_CLASS: PCWSTR = w!("NewCP.iGui.Render");

// ─── ChildState lives on the render host ─────────────────────────────

/// Per-child renderer state, stored in `GWLP_USERDATA` of the visible
/// MDI child HWND. Created in `WM_NCCREATE`, dropped in `WM_NCDESTROY`.
pub(crate) struct ChildState {
    pub(crate) child_id: i64,
    pub(crate) hwnd: HWND,
    pub(crate) target: Option<ID2D1HwndRenderTarget>,
    pub(crate) logged_hwnd_status: bool,
    pub(crate) last_logged_sequence: Option<u64>,
}

impl ChildState {
    fn render(&mut self, hdc: windows::Win32::Graphics::Gdi::HDC) -> Result<(), IGuiError> {
        if !self.logged_hwnd_status {
            if let Some(mdi_hwnd) = registry::mdi_hwnd_of(self.child_id) {
                log_hwnd_monitor("first-paint", self.child_id, mdi_hwnd, self.hwnd);
            } else {
                eprintln!(
                    "[igui-hwnd] first-paint child={} render={:?} missing mdi registry entry",
                    self.child_id, self.hwnd
                );
            }
            self.logged_hwnd_status = true;
        }

        let mut rect = RECT::default();
        unsafe { GetClientRect(self.hwnd, &mut rect) }
            .map_err(|e| IGuiError::Win32(format!("render-host GetClientRect failed: {e}")))?;
        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.ensure_render_target(width, height)?;

        let pending = batch_mod::snapshot(self.child_id);

        match pending.as_ref() {
            Some(batch) if self.last_logged_sequence != Some(batch.sequence) => {
                log_ui_batch(self.child_id, batch);
                self.last_logged_sequence = Some(batch.sequence);
            }
            None if self.last_logged_sequence.is_some() => {
                eprintln!(
                    "[igui-batch-ui] child={} no batch available at paint",
                    self.child_id
                );
                self.last_logged_sequence = None;
            }
            _ => {}
        }

        if let Some(target) = self.target.as_ref() {
            match render_d2d_frame(target, self.child_id, pending.as_deref()) {
                Ok(()) => return Ok(()),
                Err(err) => {
                    eprintln!(
                        "[igui-d2d] child={} render failed, falling back to GDI: {}",
                        self.child_id, err
                    );
                    self.target = None;
                }
            }
        }

        if let Some(batch) = pending.as_ref() {
            execute_gdi_batch(hdc, &rect, batch)?;
        } else {
            let color = phase3a_palette(self.child_id);
            fill_rect_color(hdc, &rect, rgba_to_colorref(color[0], color[1], color[2]))?;
        }
        Ok(())
    }

    fn ensure_render_target(&mut self, width: u32, height: u32) -> Result<(), IGuiError> {
        if let Some(target) = self.target.as_ref() {
            let current = unsafe { target.GetPixelSize() };
            if current.width == width && current.height == height {
                return Ok(());
            }
            unsafe { target.Resize(&D2D_SIZE_U { width, height }) }
                .map_err(|e| IGuiError::D2D(format!("ID2D1HwndRenderTarget::Resize failed: {e}")))?;
            return Ok(());
        }

        let factory = &renderer::ctx().d2d.factory;
        let target = unsafe {
            factory.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES {
                    r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_IGNORE,
                    },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    usage: D2D1_RENDER_TARGET_USAGE_NONE,
                    minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
                },
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd: self.hwnd,
                    pixelSize: D2D_SIZE_U { width, height },
                    presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                },
            )
        }
        .map_err(|e| IGuiError::D2D(format!("CreateHwndRenderTarget failed: {e}")))?;
        self.target = Some(target);
        Ok(())
    }

    fn handle_resize(&mut self, width: u32, height: u32) -> Result<(), IGuiError> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        self.ensure_render_target(width, height)
    }
}

fn render_d2d_frame(
    target: &ID2D1HwndRenderTarget,
    child_id: i64,
    batch: Option<&batch_mod::PaneBatch>,
) -> Result<(), IGuiError> {
    unsafe { target.BeginDraw() };

    match batch {
        Some(batch) => execute_d2d_batch(target, batch)?,
        None => {
            let color = phase3a_palette(child_id);
            unsafe {
                target.Clear(Some(&D2D1_COLOR_F {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: 1.0,
                }))
            };
        }
    }

    unsafe { target.EndDraw(None, None) }
        .map_err(|e| IGuiError::D2D(format!("ID2D1HwndRenderTarget::EndDraw failed: {e}")))?;
    Ok(())
}

fn execute_d2d_batch(
    target: &ID2D1HwndRenderTarget,
    batch: &batch_mod::PaneBatch,
) -> Result<(), IGuiError> {
    for cmd in &batch.cmds {
        match cmd {
            SurfaceCmd::Clear { color } => unsafe {
                target.Clear(Some(&D2D1_COLOR_F {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: color.a,
                }));
            },
            SurfaceCmd::PresentHint => {}
            SurfaceCmd::FillRect {
                rect,
                corner_radius,
                color,
            } => {
                let brush = unsafe {
                    target.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: color.a,
                        },
                        None,
                    )
                }
                .map_err(|e| IGuiError::D2D(format!("CreateSolidColorBrush failed: {e}")))?;
                let d2d_rect = D2D_RECT_F {
                    left: rect.x0,
                    top: rect.y0,
                    right: rect.x1,
                    bottom: rect.y1,
                };
                if *corner_radius <= 0.0 {
                    unsafe { target.FillRectangle(&d2d_rect, &brush) };
                } else {
                    unsafe {
                        target.FillRoundedRectangle(
                            &D2D1_ROUNDED_RECT {
                                rect: d2d_rect,
                                radiusX: *corner_radius,
                                radiusY: *corner_radius,
                            },
                            &brush,
                        )
                    };
                }
            }
            SurfaceCmd::StrokeRect {
                rect,
                corner_radius,
                half_thickness,
                color,
            } => {
                let brush = unsafe {
                    target.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: color.a,
                        },
                        None,
                    )
                }
                .map_err(|e| IGuiError::D2D(format!("CreateSolidColorBrush failed: {e}")))?;
                let d2d_rect = D2D_RECT_F {
                    left: rect.x0,
                    top: rect.y0,
                    right: rect.x1,
                    bottom: rect.y1,
                };
                let stroke_w = (2.0 * half_thickness).max(0.0);
                if *corner_radius <= 0.0 {
                    unsafe { target.DrawRectangle(&d2d_rect, &brush, stroke_w, None) };
                } else {
                    unsafe {
                        target.DrawRoundedRectangle(
                            &D2D1_ROUNDED_RECT {
                                rect: d2d_rect,
                                radiusX: *corner_radius,
                                radiusY: *corner_radius,
                            },
                            &brush,
                            stroke_w,
                            None,
                        )
                    };
                }
            }
            SurfaceCmd::DrawLine {
                p0,
                p1,
                half_thickness,
                color,
            } => {
                let brush = unsafe {
                    target.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: color.a,
                        },
                        None,
                    )
                }
                .map_err(|e| IGuiError::D2D(format!("CreateSolidColorBrush failed: {e}")))?;
                unsafe {
                    target.DrawLine(
                        Vector2 { X: p0.x, Y: p0.y },
                        Vector2 { X: p1.x, Y: p1.y },
                        &brush,
                        (2.0 * half_thickness).max(0.0),
                        None,
                    )
                };
            }
        }
    }
    Ok(())
}

fn log_ui_batch(child_id: i64, batch: &batch_mod::PaneBatch) {
    eprintln!(
        "[igui-batch-ui] child={} seq={} flags={} cmds={}",
        child_id,
        batch.sequence,
        batch.flags,
        batch.cmds.len(),
    );
    for (index, cmd) in batch.cmds.iter().enumerate() {
        match cmd {
            SurfaceCmd::Clear { color } => eprintln!(
                "[igui-batch-ui]   #{index} Clear rgba=({:.3}, {:.3}, {:.3}, {:.3})",
                color.r, color.g, color.b, color.a
            ),
            SurfaceCmd::PresentHint => {
                eprintln!("[igui-batch-ui]   #{index} PresentHint");
            }
            SurfaceCmd::FillRect {
                rect,
                corner_radius,
                color,
            } => eprintln!(
                "[igui-batch-ui]   #{index} FillRect rect=({:.1}, {:.1})-({:.1}, {:.1}) radius={:.1} rgba=({:.3}, {:.3}, {:.3}, {:.3})",
                rect.x0,
                rect.y0,
                rect.x1,
                rect.y1,
                corner_radius,
                color.r,
                color.g,
                color.b,
                color.a
            ),
            SurfaceCmd::StrokeRect {
                rect,
                corner_radius,
                half_thickness,
                color,
            } => eprintln!(
                "[igui-batch-ui]   #{index} StrokeRect rect=({:.1}, {:.1})-({:.1}, {:.1}) radius={:.1} half_thickness={:.1} rgba=({:.3}, {:.3}, {:.3}, {:.3})",
                rect.x0,
                rect.y0,
                rect.x1,
                rect.y1,
                corner_radius,
                half_thickness,
                color.r,
                color.g,
                color.b,
                color.a
            ),
            SurfaceCmd::DrawLine {
                p0,
                p1,
                half_thickness,
                color,
            } => eprintln!(
                "[igui-batch-ui]   #{index} DrawLine p0=({:.1}, {:.1}) p1=({:.1}, {:.1}) half_thickness={:.1} rgba=({:.3}, {:.3}, {:.3}, {:.3})",
                p0.x,
                p0.y,
                p1.x,
                p1.y,
                half_thickness,
                color.r,
                color.g,
                color.b,
                color.a
            ),
        }
    }
}

fn win32_failure(context: &str) -> IGuiError {
    IGuiError::Win32(format!("{context}: {}", Error::from_thread()))
}

fn log_cleanup_failure(context: &str) {
    eprintln!("[igui-win32] {context}: {}", Error::from_thread());
}

fn delete_gdi_object(obj: impl Into<windows::Win32::Graphics::Gdi::HGDIOBJ>, context: &str) {
    if !unsafe { DeleteObject(obj.into()) }.as_bool() {
        log_cleanup_failure(context);
    }
}

fn rgba_channel_to_u8(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn rgba_to_colorref(r: f32, g: f32, b: f32) -> COLORREF {
    let red = rgba_channel_to_u8(r) as u32;
    let green = rgba_channel_to_u8(g) as u32;
    let blue = rgba_channel_to_u8(b) as u32;
    COLORREF(red | (green << 8) | (blue << 16))
}

fn rect_to_win32(rect: &batch_mod::Rect) -> RECT {
    RECT {
        left: rect.x0.floor() as i32,
        top: rect.y0.floor() as i32,
        right: rect.x1.ceil() as i32,
        bottom: rect.y1.ceil() as i32,
    }
}

fn fill_rect_color(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    color: COLORREF,
) -> Result<(), IGuiError> {
    let brush = unsafe { CreateSolidBrush(color) };
    if brush.0.is_null() {
        return Err(win32_failure("CreateSolidBrush failed"));
    }
    if unsafe { FillRect(hdc, rect, brush) } == 0 {
        delete_gdi_object(brush, "DeleteObject after FillRect failure");
        return Err(win32_failure("FillRect failed"));
    }
    delete_gdi_object(brush, "DeleteObject after FillRect");
    Ok(())
}

fn stroke_rect_color(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    color: COLORREF,
    half_thickness: f32,
    corner_radius: f32,
) -> Result<(), IGuiError> {
    let brush = unsafe { CreateSolidBrush(color) };
    if brush.0.is_null() {
        return Err(win32_failure("CreateSolidBrush failed"));
    }
    let thickness = (2.0 * half_thickness).round().max(1.0) as i32;
    if corner_radius <= 0.0 {
        let mut current = *rect;
        for _ in 0..thickness {
            if unsafe { FrameRect(hdc, &current, brush) } == 0 {
                delete_gdi_object(brush, "DeleteObject after FrameRect failure");
                return Err(win32_failure("FrameRect failed"));
            }
            current.left += 1;
            current.top += 1;
            current.right -= 1;
            current.bottom -= 1;
            if current.right <= current.left || current.bottom <= current.top {
                break;
            }
        }
    } else {
        let pen = unsafe { CreatePen(PS_SOLID, thickness, color) };
        if pen.0.is_null() {
            delete_gdi_object(brush, "DeleteObject after CreatePen failure");
            return Err(win32_failure("CreatePen failed"));
        }
        let old_pen = unsafe { SelectObject(hdc, pen.into()) };
        if old_pen.0.is_null() {
            delete_gdi_object(pen, "DeleteObject after SelectObject(pen) failure");
            delete_gdi_object(brush, "DeleteObject after SelectObject(pen) failure");
            return Err(win32_failure("SelectObject(pen) failed"));
        }
        let old_brush = unsafe { SelectObject(hdc, brush.into()) };
        if old_brush.0.is_null() {
            let _ = unsafe { SelectObject(hdc, old_pen) };
            delete_gdi_object(pen, "DeleteObject after SelectObject(brush) failure");
            delete_gdi_object(brush, "DeleteObject after SelectObject(brush) failure");
            return Err(win32_failure("SelectObject(brush) failed"));
        }
        let radius = corner_radius.round().max(1.0) as i32;
        if !unsafe { RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, radius, radius) }
            .as_bool()
        {
            let _ = unsafe { SelectObject(hdc, old_pen) };
            let _ = unsafe { SelectObject(hdc, old_brush) };
            delete_gdi_object(pen, "DeleteObject after RoundRect failure");
            delete_gdi_object(brush, "DeleteObject after RoundRect failure");
            return Err(win32_failure("RoundRect failed"));
        }
        if unsafe { SelectObject(hdc, old_pen) }.0.is_null() {
            log_cleanup_failure("SelectObject restore pen failed");
        }
        if unsafe { SelectObject(hdc, old_brush) }.0.is_null() {
            log_cleanup_failure("SelectObject restore brush failed");
        }
        delete_gdi_object(pen, "DeleteObject after RoundRect");
        delete_gdi_object(brush, "DeleteObject after RoundRect");
        return Ok(());
    }
    delete_gdi_object(brush, "DeleteObject after FrameRect");
    Ok(())
}

fn draw_line_color(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    p0: &batch_mod::Point,
    p1: &batch_mod::Point,
    color: COLORREF,
    half_thickness: f32,
) -> Result<(), IGuiError> {
    let thickness = (2.0 * half_thickness).round().max(1.0) as i32;
    let pen = unsafe { CreatePen(PS_SOLID, thickness, color) };
    if pen.0.is_null() {
        return Err(win32_failure("CreatePen failed"));
    }
    let old_pen = unsafe { SelectObject(hdc, pen.into()) };
    if old_pen.0.is_null() {
        delete_gdi_object(pen, "DeleteObject after SelectObject(line pen) failure");
        return Err(win32_failure("SelectObject(line pen) failed"));
    }
    if !unsafe { MoveToEx(hdc, p0.x.round() as i32, p0.y.round() as i32, None) }.as_bool() {
        let _ = unsafe { SelectObject(hdc, old_pen) };
        delete_gdi_object(pen, "DeleteObject after MoveToEx failure");
        return Err(win32_failure("MoveToEx failed"));
    }
    if !unsafe { LineTo(hdc, p1.x.round() as i32, p1.y.round() as i32) }.as_bool() {
        let _ = unsafe { SelectObject(hdc, old_pen) };
        delete_gdi_object(pen, "DeleteObject after LineTo failure");
        return Err(win32_failure("LineTo failed"));
    }
    if unsafe { SelectObject(hdc, old_pen) }.0.is_null() {
        log_cleanup_failure("SelectObject restore line pen failed");
    }
    delete_gdi_object(pen, "DeleteObject after LineTo");
    Ok(())
}

fn hwnd_client_size(hwnd: HWND) -> Option<(i32, i32)> {
    if !unsafe { IsWindow(Some(hwnd)) }.as_bool() {
        return None;
    }
    let mut rect = RECT::default();
    if unsafe { GetClientRect(hwnd, &mut rect) }.is_err() {
        return None;
    }
    Some((rect.right - rect.left, rect.bottom - rect.top))
}

fn log_hwnd_monitor(phase: &str, child_id: i64, mdi_hwnd: HWND, render_hwnd: HWND) {
    let mdi_valid = unsafe { IsWindow(Some(mdi_hwnd)) }.as_bool();
    let render_valid = unsafe { IsWindow(Some(render_hwnd)) }.as_bool();
    let mdi_visible = mdi_valid && unsafe { IsWindowVisible(mdi_hwnd) }.as_bool();
    let render_visible = render_valid && unsafe { IsWindowVisible(render_hwnd) }.as_bool();
    let render_parent = if render_valid {
        unsafe { GetParent(render_hwnd) }.unwrap_or_default()
    } else {
        HWND::default()
    };
    let mdi_registry_match = registry::mdi_hwnd_of(child_id)
        .map(|h| h.0 == mdi_hwnd.0)
        .unwrap_or(false);
    let render_registry_match = registry::render_hwnd_of(child_id)
        .map(|h| h.0 == render_hwnd.0)
        .unwrap_or(false);
    let (mdi_w, mdi_h) = hwnd_client_size(mdi_hwnd).unwrap_or((-1, -1));
    let (render_w, render_h) = hwnd_client_size(render_hwnd).unwrap_or((-1, -1));

    eprintln!(
        "[igui-hwnd] {phase} child={child_id} mdi={:?} valid={} visible={} size={}x{} registry_match={} render={:?} valid={} visible={} size={}x{} registry_match={} parent={:?} parent_matches={}",
        mdi_hwnd,
        mdi_valid,
        mdi_visible,
        mdi_w,
        mdi_h,
        mdi_registry_match,
        render_hwnd,
        render_valid,
        render_visible,
        render_w,
        render_h,
        render_registry_match,
        render_parent,
        render_parent.0 == mdi_hwnd.0,
    );
}

fn execute_gdi_batch(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    client_rect: &RECT,
    batch: &batch_mod::PaneBatch,
) -> Result<(), IGuiError> {
    for cmd in &batch.cmds {
        match cmd {
            SurfaceCmd::Clear { color } => {
                fill_rect_color(
                    hdc,
                    client_rect,
                    rgba_to_colorref(color.r, color.g, color.b),
                )?;
            }
            SurfaceCmd::PresentHint => {}
            SurfaceCmd::FillRect {
                rect,
                corner_radius,
                color,
            } => {
                let rect = rect_to_win32(rect);
                if *corner_radius <= 0.0 {
                    fill_rect_color(hdc, &rect, rgba_to_colorref(color.r, color.g, color.b))?;
                } else {
                    let brush = unsafe { CreateSolidBrush(rgba_to_colorref(color.r, color.g, color.b)) };
                    if brush.0.is_null() {
                        return Err(IGuiError::Win32("CreateSolidBrush failed".into()));
                    }
                    let old_brush = unsafe { SelectObject(hdc, brush.into()) };
                    let radius = corner_radius.round().max(1.0) as i32;
                    let _ = unsafe { RoundRect(hdc, rect.left, rect.top, rect.right, rect.bottom, radius, radius) };
                    let _ = unsafe { SelectObject(hdc, old_brush) };
                    let _ = unsafe { DeleteObject(brush.into()) };
                }
            }
            SurfaceCmd::StrokeRect {
                rect,
                corner_radius,
                half_thickness,
                color,
            } => {
                let rect = rect_to_win32(rect);
                stroke_rect_color(
                    hdc,
                    &rect,
                    rgba_to_colorref(color.r, color.g, color.b),
                    *half_thickness,
                    *corner_radius,
                )?;
            }
            SurfaceCmd::DrawLine {
                p0,
                p1,
                half_thickness,
                color,
            } => {
                draw_line_color(
                    hdc,
                    p0,
                    p1,
                    rgba_to_colorref(color.r, color.g, color.b),
                    *half_thickness,
                )?;
            }
        }
    }
    Ok(())
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

// ─── Class registration ──────────────────────────────────────────────

pub fn register_classes() -> Result<(), IGuiError> {
    let h_instance = unsafe { GetModuleHandleW(None) }
        .map_err(|e| IGuiError::Win32(format!("GetModuleHandleW (child): {e}")))?
        .into();
    let cursor = unsafe { LoadCursorW(None, IDC_ARROW) }
        .map_err(|e| IGuiError::Win32(format!("LoadCursorW (child): {e}")))?;

    let mdi = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(mdi_child_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: Default::default(),
        hCursor: cursor,
        hbrBackground: HBRUSH(std::ptr::null_mut()),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: MDI_CHILD_CLASS,
        hIconSm: Default::default(),
    };
    let _ = unsafe { RegisterClassExW(&mdi) };

    let render = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(render_host_wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: Default::default(),
        hCursor: cursor,
        hbrBackground: HBRUSH(std::ptr::null_mut()),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: RENDER_HOST_CLASS,
        hIconSm: Default::default(),
    };
    let _ = unsafe { RegisterClassExW(&render) };

    Ok(())
}

// ─── MDI child WndProc ───────────────────────────────────────────────

/// `GWLP_USERDATA` on the MDI child stores its `child_id` as a raw
/// `isize`. The render-host HWND is looked up via the registry.
unsafe extern "system" fn mdi_child_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCREATE {
        // Recover the bootstrap from MDICREATESTRUCT.lParam.
        let create = lparam.0 as *const CREATESTRUCTW;
        let mdi_create =
            unsafe { (*create).lpCreateParams as *const MDICREATESTRUCTW };
        if mdi_create.is_null() {
            return unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) };
        }
        let bootstrap_ptr =
            unsafe { (*mdi_create).lParam.0 as *mut MdiBootstrap };
        if bootstrap_ptr.is_null() {
            return unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) };
        }
        let bootstrap = unsafe { Box::from_raw(bootstrap_ptr) };
        let child_id = bootstrap.child_id;

        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, child_id as isize) };

        let render_bootstrap = Box::into_raw(Box::new(RenderBootstrap { child_id }));
        let h_instance = unsafe { GetModuleHandleW(None) }
            .ok()
            .map(|h| windows::Win32::Foundation::HINSTANCE(h.0));
        let render_hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                RENDER_HOST_CLASS,
                PCWSTR::null(),
                WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
                0,
                0,
                0,
                0,
                Some(hwnd),
                None,
                h_instance,
                Some(render_bootstrap as *mut _),
            )
        };
        let render_hwnd = match render_hwnd {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[igui-mdi] render-host creation failed: {e}");
                let _ = unsafe { Box::from_raw(render_bootstrap) };
                return LRESULT(0);
            }
        };

        registry::register(child_id, hwnd, render_hwnd);
        log_hwnd_monitor("post-create", child_id, hwnd, render_hwnd);

        return unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) };
    }

    let child_id = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as i64;
    let render_hwnd = if child_id != 0 {
        registry::render_hwnd_of(child_id)
    } else {
        None
    };

    match msg {
        WM_SIZE => {
            if let Some(rh) = render_hwnd {
                let w = (lparam.0 & 0xFFFF) as i32;
                let h = ((lparam.0 >> 16) & 0xFFFF) as i32;
                if let Err(err) = unsafe {
                    SetWindowPos(rh, None, 0, 0, w, h, SWP_NOZORDER | SWP_NOACTIVATE)
                } {
                    eprintln!("[igui-win32] SetWindowPos failed for render host {:?}: {err}", rh);
                }
            }
            channels::push(IGuiEvent::Resize {
                child_id,
                width: (lparam.0 & 0xFFFF) as i64,
                height: ((lparam.0 >> 16) & 0xFFFF) as i64,
            });
            unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) }
        }
        WM_NCDESTROY => {
            if child_id != 0 {
                channels::push(IGuiEvent::Close { child_id });
                batch_mod::forget(child_id);
                registry::unregister(child_id);
            }
            unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) }
        }
        _ => unsafe { DefMDIChildProcW(hwnd, msg, wparam, lparam) },
    }
}

// ─── Render-host WndProc ────────────────────────────────────────────

unsafe extern "system" fn render_host_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = lparam.0 as *const CREATESTRUCTW;
        if create.is_null() {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
        let bootstrap_ptr =
            unsafe { (*create).lpCreateParams as *mut RenderBootstrap };
        if bootstrap_ptr.is_null() {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
        let bootstrap = unsafe { Box::from_raw(bootstrap_ptr) };

        let state = Box::new(ChildState {
            child_id: bootstrap.child_id,
            hwnd,
            target: None,
            logged_hwnd_status: false,
            last_logged_sequence: None,
        });
        let raw = Box::into_raw(state);
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw as isize) };

        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut ChildState;

    let ensure_ui_thread = |phase: &str| {
        let current = unsafe { GetCurrentThreadId() };
        if let Some(expected) = window::gui_thread_id() {
            if current != expected {
                eprintln!(
                    "[igui-render] {phase} on wrong thread: current={current} expected-ui={expected} hwnd={:?}",
                    hwnd
                );
            }
        } else {
            eprintln!(
                "[igui-render] {phase} without recorded UI thread id: current={current} hwnd={:?}",
                hwnd
            );
        }
    };

    match msg {
        // Suppress GDI background erase. Our render host paints
        // entirely through D2D + DXGI.
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            ensure_ui_thread("WM_PAINT");
            let mut ps = PAINTSTRUCT::default();
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
            if hdc.0.is_null() {
                eprintln!("[igui-win32] BeginPaint failed for {:?}: {}", hwnd, Error::from_thread());
                return LRESULT(0);
            }
            eprintln!(
                "[igui-paint] hwnd={:?} rcPaint=({}, {})-({}, {}) fErase={}",
                hwnd,
                ps.rcPaint.left,
                ps.rcPaint.top,
                ps.rcPaint.right,
                ps.rcPaint.bottom,
                ps.fErase.as_bool(),
            );
            if !raw.is_null() {
                let state = unsafe { &mut *raw };
                if let Err(err) = state.render(hdc) {
                    eprintln!("[igui-gdi] render error: {err}");
                }
            }
            if !unsafe { EndPaint(hwnd, &ps) }.as_bool() {
                eprintln!("[igui-win32] EndPaint failed for {:?}: {}", hwnd, Error::from_thread());
            }
            LRESULT(0)
        }
        WM_SIZE => {
            ensure_ui_thread("WM_SIZE");
            if !raw.is_null() {
                let w = (lparam.0 & 0xFFFF) as u32;
                let h = ((lparam.0 >> 16) & 0xFFFF) as u32;
                let state = unsafe { &mut *raw };
                if let Err(err) = state.handle_resize(w, h) {
                    eprintln!("[igui-render] resize error: {err}");
                }
            }
            LRESULT(0)
        }
        WM_NCDESTROY => {
            if !raw.is_null() {
                unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
                let _ = unsafe { Box::from_raw(raw) };
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

// ─── Bootstraps threaded through CreateWindow lpCreateParams ─────────

/// Threaded through `MDICREATESTRUCTW.lParam` for the MDI child window.
pub(crate) struct MdiBootstrap {
    pub(crate) child_id: i64,
}

/// Threaded through `CREATESTRUCTW.lpCreateParams` for the render host.
struct RenderBootstrap {
    child_id: i64,
}

// ─── Helpers used from window.rs ─────────────────────────────────────

/// Send `WM_SETTEXT` to the MDI child to update its title bar.
pub fn set_title(mdi_hwnd: HWND, title_w: &[u16]) {
    unsafe {
        SendMessageW(
            mdi_hwnd,
            WM_SETTEXT,
            Some(WPARAM(0)),
            Some(LPARAM(title_w.as_ptr() as isize)),
        )
    };
}

/// Ask the MDI client to destroy a child via `WM_MDIDESTROY`.
pub fn close_via_mdi(mdi_client: HWND, mdi_child: HWND) {
    unsafe {
        SendMessageW(
            mdi_client,
            WM_MDIDESTROY,
            Some(WPARAM(mdi_child.0 as usize)),
            Some(LPARAM(0)),
        )
    };
}

/// UTF-16 → owned String. Debug only.
#[allow(dead_code)]
pub(crate) fn decode_utf16(buf: &[u16]) -> String {
    let trimmed: Vec<u16> = buf.iter().copied().take_while(|c| *c != 0).collect();
    OsString::from_wide(&trimmed)
        .into_string()
        .unwrap_or_default()
}
