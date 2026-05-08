//! Stage B: dispatch from a [`StoreNode`] to a structured body encoder
//! when one is available. The writer calls [`try_encode_leaf_body`] on
//! leaf stores (those whose envelope walker captured no children) and
//! [`try_encode_parent_body`] for stores whose body interleaves
//! primitive data with inline child stores.
//!
//! Each encoder is the inverse of the decoder for its type and is
//! verified by the round-trip sweep — if a structured encoder produces
//! different bytes than the original body, the `--check` mode catches
//! it.

use crate::controls::{decode_control, encode_control, matches_control};
use crate::envelope::StoreNode;
use crate::std_links::{
    decode_link, decode_target, encode_link, encode_target, matches_link, matches_target,
};
use crate::text_attributes::{decode_attributes, encode_attributes, matches_attributes};
use crate::text_rulers::{
    decode_ruler_attributes, encode_ruler_attributes, matches_ruler_attributes,
};

/// Encode a leaf store's body from its decoded form. Returns `None` if
/// no structured encoder is registered for this type, in which case the
/// writer falls back to copying source bytes verbatim. Returns
/// `Some(bytes)` when a structured encoder fired and produced the
/// complete body.
pub fn try_encode_leaf_body(node: &StoreNode, src: &[u8]) -> Option<Vec<u8>> {
    if matches_attributes(node) {
        if let Ok(a) = decode_attributes(src, node) {
            let mut out = Vec::with_capacity(node.body_len as usize);
            encode_attributes(&mut out, &a);
            return Some(out);
        }
    }
    if matches_link(node) {
        if let Ok(l) = decode_link(src, node) {
            let mut out = Vec::with_capacity(node.body_len as usize);
            encode_link(&mut out, &l);
            return Some(out);
        }
    }
    if matches_target(node) {
        if let Ok(t) = decode_target(src, node) {
            let mut out = Vec::with_capacity(node.body_len as usize);
            encode_target(&mut out, &t);
            return Some(out);
        }
    }
    if matches_ruler_attributes(node) {
        if let Ok(r) = decode_ruler_attributes(src, node) {
            let mut out = Vec::with_capacity(node.body_len as usize);
            encode_ruler_attributes(&mut out, &r);
            return Some(out);
        }
    }
    if matches_control(node) {
        if let Ok(c) = decode_control(src, node) {
            let mut out = Vec::with_capacity(node.body_len as usize);
            encode_control(&mut out, &c);
            return Some(out);
        }
    }
    None
}
