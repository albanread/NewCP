//! Decode and encode `TextRulers.Attributes` (paragraph rulers) plus
//! helpers for reaching them from `TextRulers.StdRuler` /
//! `TextRulers.StdStyle` via the existing store-tree.
//!
//! Class hierarchy in the legacy source:
//!
//! ```text
//!   Stores.Store        Models.Model         Views.View
//!     │                    │                    │
//!     ▼                    ▼                    ▼
//!     Attributes           Style                Ruler
//!                            │                    │
//!                            ▼                    ▼
//!                            StdStyle             StdRuler
//!                            (owns Attributes)    (owns Style)
//! ```
//!
//! `TextRulers.Attributes` byte layout (after super = 1 byte
//! `Stores.Store` version):
//!
//! ```text
//!     1 byte      Attributes own version
//!     4 bytes     first       Int  (first-line indent)
//!     4 bytes     left        Int
//!     4 bytes     right       Int
//!     4 bytes     lead        Int  (leading)
//!     4 bytes     asc         Int  (ascent override)
//!     4 bytes     dsc         Int  (descent override)
//!     4 bytes     grid        Int  (grid spacing)
//!     4 bytes     opts        Set
//!     2 bytes     n           XInt (declared tab count)
//!     4n bytes    tab.stop    Int × n
//!     4n bytes    tab.type    Set × n   (only if version ≥ 2)
//! ```

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::Cursor;

#[derive(Debug, Clone)]
pub struct Tab {
    pub stop: i32,
    pub kind: u32,
}

#[derive(Debug, Clone)]
pub struct TextRulerAttributes {
    /// Stores.Store super-class version byte.
    pub store_version: u8,
    pub version: u8,
    pub first: i32,
    pub left: i32,
    pub right: i32,
    pub lead: i32,
    pub asc: i32,
    pub dsc: i32,
    pub grid: i32,
    /// `opts` as-read from the wire — not synthetically patched for v0.
    pub opts: u32,
    /// Declared `n` (tab count) from the XInt — may exceed `tabs.len()`
    /// in old files, in which case the trailing `(declared - tabs.len())`
    /// stops/types are read into `trash_*` and reproduced verbatim.
    pub declared_tab_count: i16,
    pub tabs: Vec<Tab>,
    pub trash_stops: Vec<i32>,
    pub trash_types: Vec<u32>,
}

const MAX_TABS: usize = 32;

pub fn matches_ruler_attributes(node: &StoreNode) -> bool {
    matches!(node.type_name(), "TextRulers.AttributesDesc")
}

pub fn matches_std_style(node: &StoreNode) -> bool {
    matches!(node.type_name(), "TextRulers.StdStyleDesc")
}

pub fn matches_std_ruler(node: &StoreNode) -> bool {
    matches!(node.type_name(), "TextRulers.StdRulerDesc")
}

pub fn decode_ruler_attributes(file: &[u8], node: &StoreNode) -> Result<TextRulerAttributes> {
    if !matches_ruler_attributes(node) {
        return Err(OdcError::Inconsistent("not a TextRulers.Attributes store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("ruler-attrs body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;
    let store_version = cur.read_byte()?;
    let version = cur.read_version()?;

    let first = cur.read_int()?;
    let left = cur.read_int()?;
    let right = cur.read_int()?;
    let lead = cur.read_int()?;
    let asc = cur.read_int()?;
    let dsc = cur.read_int()?;
    let grid = cur.read_int()?;
    let opts = cur.read_set()?;

    let declared = cur.read_xint()?;
    if declared < 0 {
        return Err(OdcError::Inconsistent("negative tab count"));
    }
    let kept = (declared as usize).min(MAX_TABS);
    let trash_n = declared as usize - kept;

    let mut tabs: Vec<Tab> = Vec::with_capacity(kept);
    for _ in 0..kept {
        tabs.push(Tab { stop: cur.read_int()?, kind: 0 });
    }
    let mut trash_stops = Vec::with_capacity(trash_n);
    for _ in 0..trash_n {
        trash_stops.push(cur.read_int()?);
    }

    let mut trash_types = Vec::new();
    if version >= 2 {
        for tab in tabs.iter_mut() {
            tab.kind = cur.read_set()?;
        }
        for _ in 0..trash_n {
            trash_types.push(cur.read_set()?);
        }
    }

    Ok(TextRulerAttributes {
        store_version,
        version,
        first,
        left,
        right,
        lead,
        asc,
        dsc,
        grid,
        opts,
        declared_tab_count: declared,
        tabs,
        trash_stops,
        trash_types,
    })
}

pub fn encode_ruler_attributes(out: &mut Vec<u8>, a: &TextRulerAttributes) {
    out.push(a.store_version);
    out.push(a.version);
    out.extend_from_slice(&a.first.to_le_bytes());
    out.extend_from_slice(&a.left.to_le_bytes());
    out.extend_from_slice(&a.right.to_le_bytes());
    out.extend_from_slice(&a.lead.to_le_bytes());
    out.extend_from_slice(&a.asc.to_le_bytes());
    out.extend_from_slice(&a.dsc.to_le_bytes());
    out.extend_from_slice(&a.grid.to_le_bytes());
    out.extend_from_slice(&a.opts.to_le_bytes());
    out.extend_from_slice(&a.declared_tab_count.to_le_bytes());
    for tab in &a.tabs {
        out.extend_from_slice(&tab.stop.to_le_bytes());
    }
    for stop in &a.trash_stops {
        out.extend_from_slice(&stop.to_le_bytes());
    }
    if a.version >= 2 {
        for tab in &a.tabs {
            out.extend_from_slice(&tab.kind.to_le_bytes());
        }
        for kind in &a.trash_types {
            out.extend_from_slice(&kind.to_le_bytes());
        }
    }
}

/// Walk `StdRuler → StdStyle → Attributes` and return the decoded
/// attributes. Returns `None` if the chain is malformed.
pub fn decode_std_ruler(file: &[u8], ruler: &StoreNode) -> Option<TextRulerAttributes> {
    if !matches_std_ruler(ruler) {
        return None;
    }
    let style = ruler.children.first()?;
    if !matches_std_style(style) {
        return None;
    }
    let attrs_child = style.children.first()?;
    decode_ruler_attributes(file, attrs_child).ok()
}

/// Wrapper-version bytes captured from a `TextRulers.StdRuler` body.
/// The body is `Store(1) + View(1) + Ruler(1) + inline Style + StdRuler(1)`.
/// `child_idx` indexes the inline Style child within `node.children`.
#[derive(Debug, Clone)]
pub struct StdRulerBody {
    pub store_version: u8,
    pub view_version: u8,
    pub ruler_version: u8,
    pub stdruler_version: u8,
    pub child_idx: usize,
}

pub fn decode_std_ruler_body(file: &[u8], node: &StoreNode) -> Result<StdRulerBody> {
    if !matches_std_ruler(node) {
        return Err(OdcError::Inconsistent("not a TextRulers.StdRuler store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("std-ruler body past end of file"));
    }
    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;
    let store_version = cur.read_byte()?;
    let view_version = cur.read_byte()?;
    let ruler_version = cur.read_byte()?;
    // The inline Style is at the cursor.
    let style = node
        .children
        .first()
        .ok_or(OdcError::Inconsistent("StdRuler missing inline Style"))?;
    if style.header_pos != cur.pos() {
        return Err(OdcError::Inconsistent("StdRuler inline Style not at cursor"));
    }
    cur.set_pos(style.body_pos + style.body_len)?;
    let stdruler_version = cur.read_byte()?;
    let _ = cur;
    Ok(StdRulerBody {
        store_version,
        view_version,
        ruler_version,
        stdruler_version,
        child_idx: 0,
    })
}

/// Wrapper-version bytes captured from a `TextRulers.StdStyle` body.
/// The body is `Store(1) + Elem(1) + Models.Model(1) + Style(1) +
/// inline Attributes + StdStyle(1)`.
#[derive(Debug, Clone)]
pub struct StdStyleBody {
    pub store_version: u8,
    pub elem_version: u8,
    pub model_version: u8,
    pub style_version: u8,
    pub stdstyle_version: u8,
    pub child_idx: usize,
}

pub fn decode_std_style_body(file: &[u8], node: &StoreNode) -> Result<StdStyleBody> {
    if !matches_std_style(node) {
        return Err(OdcError::Inconsistent("not a TextRulers.StdStyle store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("std-style body past end of file"));
    }
    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;
    let store_version = cur.read_byte()?;
    let elem_version = cur.read_byte()?;
    let model_version = cur.read_byte()?;
    let style_version = cur.read_byte()?;
    let attr = node
        .children
        .first()
        .ok_or(OdcError::Inconsistent("StdStyle missing inline Attributes"))?;
    if attr.header_pos != cur.pos() {
        return Err(OdcError::Inconsistent("StdStyle inline Attributes not at cursor"));
    }
    cur.set_pos(attr.body_pos + attr.body_len)?;
    let stdstyle_version = cur.read_byte()?;
    Ok(StdStyleBody {
        store_version,
        elem_version,
        model_version,
        style_version,
        stdstyle_version,
        child_idx: 0,
    })
}
