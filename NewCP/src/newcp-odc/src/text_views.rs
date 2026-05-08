//! Decode `TextViews.StdView` — the visible-text view that wraps a
//! `TextModels.StdModel` plus a controller, default ruler, and default
//! character attributes.
//!
//! The legacy reader uses Component Pascal's two-pass init pattern: the
//! base `Stores.Store` / `Views.View` / `Containers.View` chain runs as
//! the normal `Internalize` protocol, then a second `Internalize2` pass
//! kicks in for the text-view-specific fields.
//!
//! Body layout, in the order bytes appear in the file:
//!
//! ```text
//!     1 byte   Stores.Store version
//!     1 byte   Views.View version
//!     1 byte   Containers.View version
//!     store    inline Model  (the TextModels.StdModel child)
//!     store    inline Controller  (e.g. TextControllers.StdCtrl, may be NIL)
//!     1 byte   TextViews.View Internalize2 version
//!     1 byte   TextViews.StdView Internalize2 version
//!     1 byte   hideMarks Bool
//!     store    inline default Ruler  (the TextRulers.StdRuler child)
//!     store    inline default Attributes  (the TextModels.Attributes child)
//!     4 bytes  org  Int  (top-of-view character offset)
//!     4 bytes  dy   Int  (sub-line scroll offset, in pixels)
//! ```
//!
//! Each `inline X store` is captured by the envelope walker as a child of
//! the StdView. We don't re-decode them here; we return their indices in
//! `StoreNode.children` so callers can dispatch.

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::Cursor;

#[derive(Debug, Clone)]
pub struct StdViewBody {
    pub store_version: u8,
    pub view_version: u8,
    pub container_version: u8,
    pub textview_version: u8,
    pub stdview_version: u8,
    pub hide_marks: bool,
    pub origin: i32,
    pub dy: i32,
    /// Indices into `node.children` for each of the four inline child
    /// stores, in the order the writer recorded them. `None` if the
    /// envelope didn't surface a child where one was expected (the file
    /// is malformed or the store was nil).
    pub model_child: Option<usize>,
    pub controller_child: Option<usize>,
    pub default_ruler_child: Option<usize>,
    pub default_attr_child: Option<usize>,
}

pub fn matches_std_view(node: &StoreNode) -> bool {
    matches!(node.type_name(), "TextViews.StdViewDesc")
}

pub fn decode_std_view(file: &[u8], node: &StoreNode) -> Result<StdViewBody> {
    if !matches_std_view(node) {
        return Err(OdcError::Inconsistent("not a TextViews.StdView store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("std-view body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;

    // The first three version bytes from Stores.Store, Views.View,
    // Containers.View. Stores.Store version is always 0 for non-elem.
    let store_version = cur.read_byte()?;
    let view_version = cur.read_byte()?;
    let container_version = cur.read_byte()?;

    // Inline Model store at the cursor — match against children[0].
    let model_child = consume_inline_child(&mut cur, node, 0)?;

    // Inline Controller store. Legacy code allows this to be NIL (the
    // writer in that case emits a `nil` store kind 0x80). We just consume
    // whatever the envelope captured at the cursor — could be nil or a
    // real controller; our envelope walker still places it in children.
    let controller_child = consume_inline_child(&mut cur, node, 1)?;

    // Internalize2 chain: TextViews.View version, then StdView version.
    let textview_version = cur.read_byte()?;
    let stdview_version = cur.read_byte()?;

    let hide_marks = cur.read_bool()?;

    // Inline default Ruler store, then inline default Attributes store.
    let default_ruler_child = consume_inline_child(&mut cur, node, 2)?;
    let default_attr_child = consume_inline_child(&mut cur, node, 3)?;

    let origin = cur.read_int()?;
    let dy = cur.read_int()?;

    Ok(StdViewBody {
        store_version,
        view_version,
        container_version,
        textview_version,
        stdview_version,
        hide_marks,
        origin,
        dy,
        model_child,
        controller_child,
        default_ruler_child,
        default_attr_child,
    })
}

fn consume_inline_child(cur: &mut Cursor<'_>, node: &StoreNode, expected: usize) -> Result<Option<usize>> {
    let child = node.children.get(expected);
    let Some(child) = child else {
        return Err(OdcError::Inconsistent(
            "TextViews.StdView body expected an inline child the envelope didn't capture",
        ));
    };
    if child.header_pos != cur.pos() {
        return Err(OdcError::Inconsistent(
            "TextViews.StdView inline child not at cursor position",
        ));
    }
    cur.set_pos(child.body_pos + child.body_len)?;
    Ok(Some(expected))
}
