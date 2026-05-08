//! Decode and encode `StdFolds.Fold`.
//!
//! A fold is a paired view, like a `StdLinks.Link`: a left-side fold
//! owns the label and the hidden text body, a right-side fold marks
//! where the foldable region ends. Body layout (after the 3-byte
//! super-version prefix `Stores.Store` + `Views.View` + own version):
//!
//! ```text
//!     2 bytes  sideMarker XInt       0 ⇒ left side (has hidden text)
//!                                    1 ⇒ right side
//!     2 bytes  collapsedMarker XInt  0 ⇒ collapsed
//!                                    1 ⇒ expanded
//!     var      label [X]String       version 0 narrow, version 1 wide
//!     store    hidden                a TextModels.Model on the left side,
//!                                    a NIL store on the right side.
//! ```
//!
//! The hidden store is captured by our envelope reader as the fold
//! node's single child. The decoded `Fold` records the primitive
//! fields plus the raw label bytes / codepoints so the encoder can
//! reproduce the wire form exactly. The encoder emits those primitives
//! and then leaves the inline hidden store to the parent's recursive
//! `write_one` call.

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::{string_as_utf16, Cursor};
use crate::std_links::Side;

/// Stored label form. Mirrors `LinkString` from `std_links` but kept
/// independent so the modules don't take a dependency on each other for
/// data types.
#[derive(Debug, Clone)]
pub enum FoldLabel {
    Narrow(Vec<u8>),
    Wide(Vec<u16>),
}

impl FoldLabel {
    pub fn to_string(&self) -> String {
        match self {
            FoldLabel::Narrow(b) => match std::str::from_utf8(b) {
                Ok(s) => s.to_string(),
                Err(_) => b.iter().map(|&c| c as char).collect(),
            },
            FoldLabel::Wide(w) => string_as_utf16(w),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fold {
    pub store_version: u8,
    pub view_version: u8,
    pub version: u8,
    pub side: Side,
    pub side_marker: i16,
    pub collapsed: bool,
    pub collapsed_marker: i16,
    pub label: FoldLabel,
    /// Index of the hidden child store within `node.children`. Always 0
    /// for well-formed folds.
    pub child_idx: usize,
}

pub fn matches_fold(node: &StoreNode) -> bool {
    matches!(node.type_name(), "StdFolds.FoldDesc")
}

pub fn decode_fold(file: &[u8], node: &StoreNode) -> Result<Fold> {
    if !matches_fold(node) {
        return Err(OdcError::Inconsistent("not a StdFolds.Fold store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("fold body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;
    let store_version = cur.read_byte()?;
    let view_version = cur.read_byte()?;
    let version = cur.read_version()?;

    let side_marker = cur.read_xint()?;
    let collapsed_marker = cur.read_xint()?;

    let label = if version == 0 {
        FoldLabel::Narrow(cur.read_sstring()?)
    } else {
        FoldLabel::Wide(cur.read_string()?)
    };

    Ok(Fold {
        store_version,
        view_version,
        version,
        side: if side_marker == 0 { Side::Left } else { Side::Right },
        side_marker,
        collapsed: collapsed_marker == 0,
        collapsed_marker,
        label,
        child_idx: 0,
    })
}

pub fn encode_fold_prefix(out: &mut Vec<u8>, f: &Fold) {
    out.push(f.store_version);
    out.push(f.view_version);
    out.push(f.version);
    out.extend_from_slice(&f.side_marker.to_le_bytes());
    out.extend_from_slice(&f.collapsed_marker.to_le_bytes());
    match &f.label {
        FoldLabel::Narrow(b) => {
            out.extend_from_slice(b);
            out.push(0);
        }
        FoldLabel::Wide(w) => {
            for cp in w {
                out.extend_from_slice(&cp.to_le_bytes());
            }
            out.extend_from_slice(&[0u8, 0u8]);
        }
    }
}
