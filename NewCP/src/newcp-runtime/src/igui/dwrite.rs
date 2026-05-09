//! DirectWrite factory and stub font manager.
//!
//! Phase 1: just create the shared factory and a system font collection
//! handle so we can prove DWrite initializes cleanly. Format / layout
//! caches arrive in Phase 4.

#![cfg(windows)]

use windows::core::Interface;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFactory2, IDWriteFontCollection,
    DWRITE_FACTORY_TYPE_SHARED,
};

use super::IGuiError;

#[allow(dead_code)] // factory + system collection are held for Phase 4
pub struct DWriteContext {
    pub factory: IDWriteFactory2,
    pub system_fonts: IDWriteFontCollection,
}

impl DWriteContext {
    pub fn new() -> Result<Self, IGuiError> {
        let raw: IDWriteFactory =
            unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }.map_err(|e| {
                IGuiError::DWrite(format!("DWriteCreateFactory(SHARED) failed: {e}"))
            })?;

        let factory: IDWriteFactory2 = raw
            .cast()
            .map_err(|e| IGuiError::DWrite(format!("cast → IDWriteFactory2 failed: {e}")))?;

        // Eagerly grab the system font collection so any failure surfaces
        // at startup rather than mid-render.
        let mut system_fonts: Option<IDWriteFontCollection> = None;
        unsafe { factory.GetSystemFontCollection(&mut system_fonts, false) }
            .map_err(|e| IGuiError::DWrite(format!("GetSystemFontCollection failed: {e}")))?;
        let system_fonts = system_fonts.ok_or_else(|| {
            IGuiError::DWrite(
                "GetSystemFontCollection returned success but no collection".into(),
            )
        })?;

        Ok(Self {
            factory,
            system_fonts,
        })
    }
}
