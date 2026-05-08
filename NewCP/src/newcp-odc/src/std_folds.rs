//! Decode `StdFolds.Fold`.
//!
//! A fold is a paired view, like a `StdLinks.Link`: a left-side fold owns
//! the label and the hidden text body, a right-side fold marks where the
//! foldable region ends. Body layout (after the 3-byte super-version
//! prefix `Stores.Store` + `Views.View` + own version):
//!
//! ```text
//!     2 bytes  sideMarker XInt        0 ⇒ left side (has hidden text)
//!                                     1 ⇒ right side
//!     2 bytes  collapsedMarker XInt   0 ⇒ collapsed
//!                                     1 ⇒ expanded
//!     var      label [X]String        version 0 narrow, version 1 wide
//!     store    hidden                 a TextModels.Model on the left side,
//!                                     a NIL store on the right side. Always
//!                                     written but only meaningful for left.
//! ```
//!
//! The hidden store is captured by our envelope reader as the fold node's
//! single child. The decoder here surfaces the fold's primitive fields
//! (side / collapsed / label) and lets the caller resolve `node.children[0]`
//! for the hidden body.

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::{string_as_utf16, Cursor};
use crate::std_links::Side;

#[derive(Debug, Clone)]
pub struct Fold {
    pub version: u8,
    pub side: Side,
    pub collapsed: bool,
    pub label: String,
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
    cur.read_byte()?; // Store version
    cur.read_byte()?; // View version
    let version = cur.read_version()?;

    let side_marker = cur.read_xint()?;
    let collapsed_marker = cur.read_xint()?;

    let label = if version == 0 {
        let raw = cur.read_sstring()?;
        match std::str::from_utf8(&raw) {
            Ok(s) => s.to_string(),
            Err(_) => raw.iter().map(|&b| b as char).collect::<String>(),
        }
    } else {
        let raw = cur.read_string()?;
        string_as_utf16(&raw)
    };

    Ok(Fold {
        version,
        side: if side_marker == 0 { Side::Left } else { Side::Right },
        collapsed: collapsed_marker == 0,
        label,
    })
}
