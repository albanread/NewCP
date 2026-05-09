//! iGui — integrated GUI for NewCP.
//!
//! Direct-rendered MDI frame using Direct2D and DirectWrite, implemented
//! entirely inside `newcp-runtime` (no external multiwingui DLL).
//!
//! Phase 1 scope: open an MDI frame + MDI client, initialize D3D11 /
//! Direct2D / DirectWrite, paint a solid color into the MDI client area
//! during `WM_PAINT`, and exit cleanly on `WM_CLOSE` / `WM_DESTROY`.
//! No language thread, no children, no batches, no event mailbox yet.

#![cfg(windows)]

mod d2d;
mod d3d;
mod dwrite;
mod window;

pub use window::run;

/// Errors surfaced from iGui startup. Phase 1 keeps this lossy on purpose;
/// every variant carries enough text to diagnose without a debugger.
#[derive(Debug)]
pub enum IGuiError {
    Win32(String),
    D3D(String),
    D2D(String),
    DWrite(String),
}

impl std::fmt::Display for IGuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IGuiError::Win32(msg) => write!(f, "iGui: Win32: {msg}"),
            IGuiError::D3D(msg) => write!(f, "iGui: D3D: {msg}"),
            IGuiError::D2D(msg) => write!(f, "iGui: D2D: {msg}"),
            IGuiError::DWrite(msg) => write!(f, "iGui: DirectWrite: {msg}"),
        }
    }
}

impl std::error::Error for IGuiError {}

/// Phase 1 paints this slate gray into the MDI client area so we can see
/// the renderer is actually running. Will be replaced once the surface
/// executor lands and children own their own colors.
pub(crate) const PHASE1_BACKGROUND: [f32; 4] = [0.18, 0.20, 0.23, 1.0];
