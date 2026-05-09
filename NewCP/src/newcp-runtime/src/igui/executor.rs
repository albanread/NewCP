//! Surface executor: drains a `PaneBatch` and translates each
//! `SurfaceCmd` into Direct2D draw calls. Phase 3b implements the
//! lifecycle and basic geometry primitives; the rest land in 3c / 5.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::HashMap;

use windows::core::Interface;
use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{
    ID2D1Brush, ID2D1DeviceContext, ID2D1SolidColorBrush, ID2D1StrokeStyle, D2D1_ROUNDED_RECT,
};
use windows_numerics::Vector2;

use super::batch::{PaneBatch, Rgba, SurfaceCmd};
use super::renderer;
use super::IGuiError;

/// Process-wide solid-color brush cache. Brushes are bound to the D2D
/// device context, which is itself process-wide (one per `iGui::run`),
/// so brushes outlive any individual swap chain.
struct BrushCache {
    map: HashMap<u128, ID2D1SolidColorBrush>,
}

impl BrushCache {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn get(&mut self, ctx: &ID2D1DeviceContext, color: Rgba) -> Result<ID2D1Brush, IGuiError> {
        let key = pack_color(color);
        if !self.map.contains_key(&key) {
            let d2d_color = D2D1_COLOR_F {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            };
            let brush = unsafe { ctx.CreateSolidColorBrush(&d2d_color, None) }
                .map_err(|e| IGuiError::D2D(format!("CreateSolidColorBrush: {e}")))?;
            self.map.insert(key, brush);
        }
        Ok(self.map.get(&key).unwrap().cast::<ID2D1Brush>().unwrap())
    }
}

fn pack_color(c: Rgba) -> u128 {
    let r = c.r.to_bits() as u128;
    let g = c.g.to_bits() as u128;
    let b = c.b.to_bits() as u128;
    let a = c.a.to_bits() as u128;
    r | (g << 32) | (b << 64) | (a << 96)
}

thread_local! {
    static BRUSHES: RefCell<BrushCache> = RefCell::new(BrushCache::new());
}

/// Execute every command in `batch` against the currently bound D2D
/// render target. Caller is responsible for `BeginDraw` / `EndDraw` /
/// `Present`. Returns `Ok(present_hint)` — true if the batch wants an
/// explicit Present beyond the default.
pub fn execute(batch: &PaneBatch) -> Result<bool, IGuiError> {
    let r = renderer::ctx();
    let ctx = &r.d2d.context;
    let mut want_present = false;

    let no_stroke: Option<&ID2D1StrokeStyle> = None;

    for cmd in &batch.cmds {
        match cmd {
            SurfaceCmd::Clear { color } => unsafe {
                ctx.Clear(Some(&D2D1_COLOR_F {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: color.a,
                }));
            },
            SurfaceCmd::PresentHint => {
                want_present = true;
            }
            SurfaceCmd::FillRect {
                rect,
                corner_radius,
                color,
            } => {
                let brush = BRUSHES.with(|c| c.borrow_mut().get(ctx, *color))?;
                let r2d = D2D_RECT_F {
                    left: rect.x0,
                    top: rect.y0,
                    right: rect.x1,
                    bottom: rect.y1,
                };
                if *corner_radius <= 0.0 {
                    unsafe { ctx.FillRectangle(&r2d, &brush) };
                } else {
                    let rr = D2D1_ROUNDED_RECT {
                        rect: r2d,
                        radiusX: *corner_radius,
                        radiusY: *corner_radius,
                    };
                    unsafe { ctx.FillRoundedRectangle(&rr, &brush) };
                }
            }
            SurfaceCmd::StrokeRect {
                rect,
                corner_radius,
                half_thickness,
                color,
            } => {
                let brush = BRUSHES.with(|c| c.borrow_mut().get(ctx, *color))?;
                let r2d = D2D_RECT_F {
                    left: rect.x0,
                    top: rect.y0,
                    right: rect.x1,
                    bottom: rect.y1,
                };
                let stroke_w = (2.0 * half_thickness).max(0.0);
                if *corner_radius <= 0.0 {
                    unsafe { ctx.DrawRectangle(&r2d, &brush, stroke_w, no_stroke) };
                } else {
                    let rr = D2D1_ROUNDED_RECT {
                        rect: r2d,
                        radiusX: *corner_radius,
                        radiusY: *corner_radius,
                    };
                    unsafe { ctx.DrawRoundedRectangle(&rr, &brush, stroke_w, no_stroke) };
                }
            }
            SurfaceCmd::DrawLine {
                p0,
                p1,
                half_thickness,
                color,
            } => {
                let brush = BRUSHES.with(|c| c.borrow_mut().get(ctx, *color))?;
                let stroke_w = (2.0 * half_thickness).max(0.0);
                unsafe {
                    ctx.DrawLine(
                        Vector2 { X: p0.x, Y: p0.y },
                        Vector2 { X: p1.x, Y: p1.y },
                        &brush,
                        stroke_w,
                        no_stroke,
                    )
                };
            }
        }
    }

    Ok(want_present)
}
