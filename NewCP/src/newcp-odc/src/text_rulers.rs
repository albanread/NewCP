//! Decode `TextRulers.Attributes` (paragraph rulers) and helpers for
//! reaching them from `TextRulers.StdRuler` / `TextRulers.StdStyle` via
//! the existing store-tree.
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
//! `TextRulers.Attributes` byte layout (after super = 1 byte Stores.Store
//! version):
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
//!     4n bytes    tab.type    Set × n   (only if version ≥ 2; older
//!                                        files leave the type bits 0)
//! ```
//!
//! Older files (version 0) used a "rightFixed default" interpretation;
//! `Internalize` adjusts by setting `rightFixed` in `opts`. We replicate
//! that so callers see consistent semantics across versions.

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::Cursor;

/// One tab stop within a paragraph ruler.
#[derive(Debug, Clone)]
pub struct Tab {
    /// Stop position (in BlackBox internal length units — millipoints × scale).
    pub stop: i32,
    /// Tab kind flags (alignment, fill, etc.). Always 0 in version-1 files.
    pub kind: u32,
}

/// `TextRulers.Attributes` — the actual paragraph metrics carried by a
/// ruler. The legacy struct uses Component Pascal `INTEGER`s for lengths;
/// units are BlackBox's internal 1/36000-of-an-inch (Ports.point) scale.
#[derive(Debug, Clone)]
pub struct TextRulerAttributes {
    pub version: u8,
    pub first: i32,
    pub left: i32,
    pub right: i32,
    pub lead: i32,
    pub asc: i32,
    pub dsc: i32,
    pub grid: i32,
    pub opts: u32,
    pub tabs: Vec<Tab>,
}

/// Bit set by version-0 files when their `rightFixed` flag was implicit.
/// The legacy `Internalize` sets it on the way in for compatibility.
const OPT_RIGHT_FIXED: u32 = 1 << 0;

/// Capacity matches the legacy `maxTabs` constant. Tabs declared past this
/// limit are read but discarded, matching legacy behaviour exactly.
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
    cur.read_byte()?; // Stores.Store version
    let version = cur.read_version()?;

    let first = cur.read_int()?;
    let left = cur.read_int()?;
    let right = cur.read_int()?;
    let lead = cur.read_int()?;
    let asc = cur.read_int()?;
    let dsc = cur.read_int()?;
    let grid = cur.read_int()?;
    let mut opts = cur.read_set()?;

    let declared = cur.read_xint()? as i32;
    if declared < 0 {
        return Err(OdcError::Inconsistent("negative tab count"));
    }
    let kept = (declared as usize).min(MAX_TABS);
    let trash = declared as usize - kept;

    let mut tabs: Vec<Tab> = Vec::with_capacity(kept);
    for _ in 0..kept {
        tabs.push(Tab { stop: cur.read_int()?, kind: 0 });
    }
    for _ in 0..trash {
        cur.read_int()?; // discard
    }

    if version == 0 {
        opts |= OPT_RIGHT_FIXED;
    }

    if version >= 2 {
        for tab in tabs.iter_mut() {
            tab.kind = cur.read_set()?;
        }
        for _ in 0..trash {
            cur.read_set()?;
        }
    }

    Ok(TextRulerAttributes {
        version,
        first,
        left,
        right,
        lead,
        asc,
        dsc,
        grid,
        opts,
        tabs,
    })
}

/// Walk `StdRuler → StdStyle → Attributes` and return the decoded
/// attributes. Returns `None` if the chain is malformed (e.g. an alien
/// store anywhere along the path, or the legacy "no style" case).
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
