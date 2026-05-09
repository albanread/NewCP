//! DirectWrite factory and font manager.
//!
//! Owns the process-wide `IDWriteFactory2` and the system font
//! collection. Phase 4 adds a format cache keyed on
//! `(family, size, weight, style, stretch, alignment, locale)` so
//! repeated `DrawTextRun` calls with the same descriptor reuse one
//! `IDWriteTextFormat`. Layouts are not cached yet — they're rebuilt
//! each call. Optimization for a follow-up phase if profiling
//! warrants it.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::HashMap;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteFactory2, IDWriteFontCollection,
    IDWriteTextFormat, IDWriteTextLayout,
    DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH, DWRITE_FONT_STRETCH_CONDENSED, DWRITE_FONT_STRETCH_EXPANDED,
    DWRITE_FONT_STRETCH_EXTRA_CONDENSED, DWRITE_FONT_STRETCH_EXTRA_EXPANDED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STRETCH_SEMI_CONDENSED,
    DWRITE_FONT_STRETCH_SEMI_EXPANDED, DWRITE_FONT_STRETCH_ULTRA_CONDENSED,
    DWRITE_FONT_STRETCH_ULTRA_EXPANDED, DWRITE_FONT_STYLE,
    DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STYLE_OBLIQUE,
    DWRITE_FONT_WEIGHT, DWRITE_TEXT_ALIGNMENT,
    DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
    DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TRIMMING,
    DWRITE_TRIMMING_GRANULARITY_CHARACTER, DWRITE_TRIMMING_GRANULARITY_NONE,
    DWRITE_TRIMMING_GRANULARITY_WORD,
};

use super::batch::{FontStretch, FontStyle, TextAlign, TextRun, TextTrimming};
use super::IGuiError;

#[allow(dead_code)] // system_fonts held for future font enumeration
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

// ─── Format cache ────────────────────────────────────────────────────

#[derive(Clone, Eq, PartialEq, Hash)]
struct FormatKey {
    family: String,
    size_q: u32, // quantized to micro-DIPs to make f32 hashable
    weight: u16,
    style: FontStyle,
    stretch: FontStretch,
    locale: String,
    alignment: TextAlign,
}

fn quantize(size: f32) -> u32 {
    // Quantize to 1/256 DIP to avoid hashing floats while keeping
    // sub-pixel formats distinct.
    (size * 256.0).round() as u32
}

thread_local! {
    static FORMATS: RefCell<HashMap<FormatKey, IDWriteTextFormat>> =
        RefCell::new(HashMap::new());
}

/// Get or build a `IDWriteTextFormat` for the given run. Cached on
/// the GUI thread (where rendering happens) keyed by the run's
/// formatting fields (text payload not included).
pub fn format_for(run: &TextRun) -> Result<IDWriteTextFormat, IGuiError> {
    let key = FormatKey {
        family: run.family.clone(),
        size_q: quantize(run.size),
        weight: run.weight,
        style: run.style,
        stretch: run.stretch,
        locale: run.locale.clone(),
        alignment: run.alignment,
    };
    FORMATS.with(|cell| {
        let mut map = cell.borrow_mut();
        if let Some(f) = map.get(&key) {
            return Ok(f.clone());
        }
        let factory = &super::renderer::ctx().dwrite.factory;
        let family_w = utf16(&run.family);
        let locale_w = utf16(&run.locale);
        let format = unsafe {
            factory.CreateTextFormat(
                PCWSTR(family_w.as_ptr()),
                None,
                DWRITE_FONT_WEIGHT(run.weight as i32),
                map_style(run.style),
                map_stretch(run.stretch),
                run.size,
                PCWSTR(locale_w.as_ptr()),
            )
        }
        .map_err(|e| IGuiError::DWrite(format!("CreateTextFormat: {e}")))?;
        unsafe {
            format
                .SetTextAlignment(map_alignment(run.alignment))
                .map_err(|e| IGuiError::DWrite(format!("SetTextAlignment: {e}")))?;
        }
        map.insert(key, format.clone());
        Ok(format)
    })
}

/// Build an `IDWriteTextLayout` for the run. Layouts aren't cached
/// yet — every text command rebuilds. The format _is_ cached so the
/// expensive part (font face resolution) only happens once per
/// formatting tuple.
pub fn layout_for(run: &TextRun) -> Result<IDWriteTextLayout, IGuiError> {
    let format = format_for(run)?;
    let factory = &super::renderer::ctx().dwrite.factory;
    let text_w = utf16(&run.text);
    let max_width = run.max_width.unwrap_or(f32::MAX);
    let max_height = f32::MAX;
    let layout = unsafe {
        factory.CreateTextLayout(&text_w, &format, max_width, max_height)
    }
    .map_err(|e| IGuiError::DWrite(format!("CreateTextLayout: {e}")))?;

    // Apply trimming if requested.
    apply_trimming(&layout, run.trimming)?;
    Ok(layout)
}

fn apply_trimming(layout: &IDWriteTextLayout, t: TextTrimming) -> Result<(), IGuiError> {
    if matches!(t, TextTrimming::None) {
        // Default DirectWrite layout has no trimming; skip SetTrimming
        // entirely. Calling it with a NULL inline-object sign returns
        // E_POINTER on some Windows versions.
        return Ok(());
    }
    let granularity = match t {
        TextTrimming::EllipsisChar => DWRITE_TRIMMING_GRANULARITY_CHARACTER,
        TextTrimming::EllipsisWord => DWRITE_TRIMMING_GRANULARITY_WORD,
        TextTrimming::None => DWRITE_TRIMMING_GRANULARITY_NONE, // unreachable
    };
    let trimming = DWRITE_TRIMMING {
        granularity,
        delimiter: 0,
        delimiterCount: 0,
    };
    let factory = &super::renderer::ctx().dwrite.factory;
    let format = format_for_basic("Segoe UI", 12.0)?;
    let sign = unsafe { factory.CreateEllipsisTrimmingSign(&format) }
        .map_err(|e| IGuiError::DWrite(format!("CreateEllipsisTrimmingSign: {e}")))?;
    unsafe { layout.SetTrimming(&trimming, &sign) }
        .map_err(|e| IGuiError::DWrite(format!("SetTrimming(ellipsis): {e}")))?;
    Ok(())
}

fn format_for_basic(family: &str, size: f32) -> Result<IDWriteTextFormat, IGuiError> {
    let factory = &super::renderer::ctx().dwrite.factory;
    let family_w = utf16(family);
    let locale_w = utf16("en-us");
    unsafe {
        factory.CreateTextFormat(
            PCWSTR(family_w.as_ptr()),
            None,
            DWRITE_FONT_WEIGHT(400),
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            size,
            PCWSTR(locale_w.as_ptr()),
        )
    }
    .map_err(|e| IGuiError::DWrite(format!("CreateTextFormat(basic): {e}")))
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn map_style(s: FontStyle) -> DWRITE_FONT_STYLE {
    match s {
        FontStyle::Normal => DWRITE_FONT_STYLE_NORMAL,
        FontStyle::Italic => DWRITE_FONT_STYLE_ITALIC,
        FontStyle::Oblique => DWRITE_FONT_STYLE_OBLIQUE,
    }
}

fn map_stretch(s: FontStretch) -> DWRITE_FONT_STRETCH {
    match s {
        FontStretch::UltraCondensed => DWRITE_FONT_STRETCH_ULTRA_CONDENSED,
        FontStretch::ExtraCondensed => DWRITE_FONT_STRETCH_EXTRA_CONDENSED,
        FontStretch::Condensed => DWRITE_FONT_STRETCH_CONDENSED,
        FontStretch::SemiCondensed => DWRITE_FONT_STRETCH_SEMI_CONDENSED,
        FontStretch::Normal => DWRITE_FONT_STRETCH_NORMAL,
        FontStretch::SemiExpanded => DWRITE_FONT_STRETCH_SEMI_EXPANDED,
        FontStretch::Expanded => DWRITE_FONT_STRETCH_EXPANDED,
        FontStretch::ExtraExpanded => DWRITE_FONT_STRETCH_EXTRA_EXPANDED,
        FontStretch::UltraExpanded => DWRITE_FONT_STRETCH_ULTRA_EXPANDED,
    }
}

fn map_alignment(a: TextAlign) -> DWRITE_TEXT_ALIGNMENT {
    match a {
        TextAlign::Leading => DWRITE_TEXT_ALIGNMENT_LEADING,
        TextAlign::Trailing => DWRITE_TEXT_ALIGNMENT_TRAILING,
        TextAlign::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
        TextAlign::Justified => DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
    }
}

/// Map `i32` from CP enum constants into our typed `FontStyle`.
pub fn cp_style(v: i32) -> FontStyle {
    match v {
        1 => FontStyle::Italic,
        2 => FontStyle::Oblique,
        _ => FontStyle::Normal,
    }
}

/// Map `i32` from CP enum constants into our typed `FontStretch`.
pub fn cp_stretch(v: i32) -> FontStretch {
    match v {
        1 => FontStretch::UltraCondensed,
        2 => FontStretch::ExtraCondensed,
        3 => FontStretch::Condensed,
        4 => FontStretch::SemiCondensed,
        5 => FontStretch::Normal,
        6 => FontStretch::SemiExpanded,
        7 => FontStretch::Expanded,
        8 => FontStretch::ExtraExpanded,
        9 => FontStretch::UltraExpanded,
        _ => FontStretch::Normal,
    }
}

pub fn cp_align(v: i32) -> TextAlign {
    match v {
        1 => TextAlign::Trailing,
        2 => TextAlign::Center,
        3 => TextAlign::Justified,
        _ => TextAlign::Leading,
    }
}

pub fn cp_trimming(v: i32) -> TextTrimming {
    match v {
        1 => TextTrimming::EllipsisChar,
        2 => TextTrimming::EllipsisWord,
        _ => TextTrimming::None,
    }
}
