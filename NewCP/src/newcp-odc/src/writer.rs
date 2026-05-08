//! Stores envelope writer — reconstructs `.odc` bytes from a parsed
//! [`Document`].
//!
//! The writer mirrors `Stores.Reader` step-for-step: it walks the store
//! tree depth-first, the same order in which the reader populated the
//! type dictionary, so every encoding decision (new-ext / new-base /
//! old-type) lines up exactly with what the reader saw. Body bytes that
//! aren't covered by a child store are copied verbatim from the original
//! file; child stores are spliced in via recursion.
//!
//! When the input is an unmodified `Document` produced by [`read_bytes`],
//! the output is byte-identical to the source. That property is what
//! [`check_roundtrip`] verifies — a hash match proves the reader didn't
//! lose information about the envelope, the path encoding, or the four
//! header fields.
//!
//! Subsequent encoder work (Stage B) will replace verbatim body copies
//! with structured re-encodes for the view kinds we already decode, but
//! the envelope writer here is the same machinery.

use crate::body_encoders::try_encode_leaf_body;
use crate::envelope::{Document, StoreKind, StoreNode, WirePath, WireTerminator};
use crate::error::{OdcError, Result};
use crate::std_folds::{decode_fold, encode_fold_prefix, matches_fold};
use crate::text_model::{decode_std_model, matches_std_model, Piece};
use crate::text_rulers::{
    decode_std_ruler_body, decode_std_style_body, matches_std_ruler, matches_std_style,
};
use crate::text_views::{decode_std_view, matches_std_view};

/// Tags written for path components — same values as the reader expects.
const NEW_BASE: u8 = 0xF0;
const NEW_EXT: u8 = 0xF1;
const OLD_TYPE: u8 = 0xF2;

/// Kind bytes — see `Stores.WriteStore` in the legacy source.
const KIND_NIL: u8 = 0x80;
const KIND_LINK: u8 = 0x81;
const KIND_STORE: u8 = 0x82;
const KIND_ELEM: u8 = 0x83;
const KIND_NEWLINK: u8 = 0x84;

// `Stores.ElemDesc` is a marker the reader strips from `type_path`. We
// don't synthesise it any more — the captured `wire_path` records
// whether the original wire emitted it and at which position.

/// Writer state — a fresh dictionary that grows in the same order the
/// reader's did.
struct WriterEnvelope {
    /// Each entry is the type name as written; index = type id. The
    /// reader's `tDict` records names plus parent-pointer base ids, but
    /// the writer only needs the name-to-id direction for emitting
    /// `oldType` references.
    dict: Vec<String>,
}

impl WriterEnvelope {
    fn new() -> Self {
        Self { dict: Vec::new() }
    }

    fn lookup(&self, name: &str) -> Option<usize> {
        self.dict.iter().position(|n| n == name)
    }

    fn add(&mut self, name: String) -> usize {
        let id = self.dict.len();
        self.dict.push(name);
        id
    }
}

/// Serialize a parsed document back to bytes.
pub fn write_document(doc: &Document) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(doc.bytes.len());
    out.extend_from_slice(b"CDOo");
    out.extend_from_slice(&[0u8; 4]); // four zero bytes after magic
    let mut env = WriterEnvelope::new();
    write_one(&mut out, &doc.root, &doc.bytes, &mut env)?;
    Ok(out)
}

/// Convenience: read a `.odc`, serialize it back, and report whether the
/// bytes match. Returns `(matched, original_len, output_len)`.
pub fn check_roundtrip(bytes: Vec<u8>) -> Result<(bool, usize, usize)> {
    let original_len = bytes.len();
    let doc = crate::envelope::read_bytes(bytes)?;
    let out = write_document(&doc)?;
    let matched = out == doc.bytes;
    Ok((matched, original_len, out.len()))
}

fn write_one(
    out: &mut Vec<u8>,
    node: &StoreNode,
    src: &[u8],
    env: &mut WriterEnvelope,
) -> Result<()> {
    match node.kind {
        StoreKind::Nil => {
            out.push(KIND_NIL);
            write_i32(out, node.comment);
            write_i32(out, node.raw_next);
        }
        StoreKind::Link => {
            out.push(KIND_LINK);
            write_i32(out, node.id);
            write_i32(out, node.comment);
            write_i32(out, node.raw_next);
        }
        StoreKind::NewLink => {
            out.push(KIND_NEWLINK);
            write_i32(out, node.id);
            write_i32(out, node.comment);
            write_i32(out, node.raw_next);
        }
        StoreKind::Store | StoreKind::Elem => {
            let kind_byte = if node.kind == StoreKind::Store { KIND_STORE } else { KIND_ELEM };
            out.push(kind_byte);
            write_path(out, node, env)?;
            write_i32(out, node.comment);
            write_i32(out, node.raw_next);
            write_i32(out, node.raw_down);
            write_i32(out, node.body_len as i32);
            write_body(out, node, src, env)?;
        }
    }
    Ok(())
}

/// Emit the type-path bytes for a store. Mirrors `Stores.ReadPath`'s
/// algorithm in reverse: walk the chain most-derived first, emit
/// `newExt` for each type until we either hit one already in the dict
/// (emit `oldType` and stop) or reach the end of the chain (emit
/// `newBase` for the final entry).
///
/// For elem stores we synthetically prepend `Stores.ElemDesc` because
/// the reader filters that name out of `type_path` but the writer must
/// still emit it on the wire.
fn write_path(out: &mut Vec<u8>, node: &StoreNode, env: &mut WriterEnvelope) -> Result<()> {
    let wire = node
        .wire_path
        .as_ref()
        .ok_or(OdcError::Inconsistent("full store missing wire_path"))?;

    match wire {
        WirePath::Reference(id) => {
            out.push(OLD_TYPE);
            write_i32(out, *id);
        }
        WirePath::Extension { extensions, terminator } => {
            // Each newExt name was added to the dict in encounter order
            // by the reader. Mirror that in the writer's dict so future
            // oldType references resolve to the same ids.
            for name in extensions {
                out.push(NEW_EXT);
                write_utf8_sstring(out, name);
                env.add(name.clone());
            }
            match terminator {
                WireTerminator::NewBase(name) => {
                    out.push(NEW_BASE);
                    write_utf8_sstring(out, name);
                    env.add(name.clone());
                }
                WireTerminator::OldType(id) => {
                    out.push(OLD_TYPE);
                    write_i32(out, *id);
                }
            }
        }
    }
    Ok(())
}

/// Emit a full store's body. Walks the original bytes between
/// `body_pos` and `body_end`, splicing in a recursive reconstruction at
/// each child's `header_pos`. Children are visited in the same order
/// the reader produced (parent's `children` vector).
fn write_body(
    out: &mut Vec<u8>,
    node: &StoreNode,
    src: &[u8],
    env: &mut WriterEnvelope,
) -> Result<()> {
    // Stage B: when this store has no inline child stores, try a
    // structured leaf encoder. Falls through to verbatim copy if none
    // registered for this type.
    if node.children.is_empty() {
        if let Some(bytes) = try_encode_leaf_body(node, src) {
            if bytes.len() != node.body_len as usize {
                return Err(OdcError::Inconsistent(
                    "structured leaf encoder produced wrong length",
                ));
            }
            out.extend_from_slice(&bytes);
            return Ok(());
        }
    }

    // Try a parent encoder — these emit primitive bytes from decoded
    // fields, interleaved with recursive `write_one` calls for inline
    // children.
    if try_encode_parent_body(out, node, src, env)? {
        return Ok(());
    }

    // Verbatim fallback. Use this store's captured body_data — bytes
    // between/around children come from there, children get written
    // recursively. Position arithmetic is parent-relative: each child's
    // start within body_data is `child.header_pos - parent.body_pos`,
    // its end is the same minus body_pos applied to body_pos+body_len.
    let parent_body_start = node.body_pos;
    let parent_body_data = if !node.body_data.is_empty() {
        node.body_data.as_slice()
    } else if (node.body_pos + node.body_len) as usize <= src.len() {
        &src[node.body_pos as usize..(node.body_pos + node.body_len) as usize]
    } else {
        return Err(OdcError::Inconsistent("body extends past source bytes"));
    };

    let mut sorted: Vec<&StoreNode> = node.children.iter().collect();
    sorted.sort_by_key(|c| c.header_pos);

    // `cur` is the offset within `parent_body_data` (= bytes since
    // body started). Each child's start within parent body =
    // `child.header_pos - parent_body_start`.
    let body_data_len = parent_body_data.len();
    let mut cur: usize = 0;
    for child in sorted {
        let cstart_abs = child.header_pos;
        let cend_abs = child.body_pos + child.body_len;
        if cstart_abs < parent_body_start || cend_abs > parent_body_start + body_data_len as u64 {
            return Err(OdcError::Inconsistent("child range outside parent body"));
        }
        let cstart = (cstart_abs - parent_body_start) as usize;
        let cend = (cend_abs - parent_body_start) as usize;
        if cstart < cur {
            return Err(OdcError::Inconsistent("child positions out of order"));
        }
        if cstart > cur {
            out.extend_from_slice(&parent_body_data[cur..cstart]);
        }
        write_one(out, child, src, env)?;
        cur = cend;
    }
    if cur < body_data_len {
        out.extend_from_slice(&parent_body_data[cur..body_data_len]);
    }
    Ok(())
}

/// Emit the `ano` byte for a piece referencing attribute pool slot
/// `attr_idx`, plus — when this piece introduces a new pool entry —
/// the inline `TextModels.Attributes` child store. Wire order is `ano
/// byte` then `inline store`, matching `Stores.ReadStore`.
fn write_ano(
    attr_idx: usize,
    emitted_attrs: &mut usize,
    node: &StoreNode,
    src: &[u8],
    out: &mut Vec<u8>,
    env: &mut WriterEnvelope,
) -> Result<()> {
    if attr_idx == *emitted_attrs {
        out.push(*emitted_attrs as u8);
        // Find the inline TextModels.Attributes child this slot refers
        // to (the N-th attributes child, document order).
        let mut seen = 0usize;
        for child in &node.children {
            if child.type_name() == "TextModels.AttributesDesc" {
                if seen == *emitted_attrs {
                    write_one(out, child, src, env)?;
                    *emitted_attrs += 1;
                    return Ok(());
                }
                seen += 1;
            }
        }
        Err(OdcError::Inconsistent("attribute pool index out of range"))
    } else {
        out.push(attr_idx as u8);
        Ok(())
    }
}

/// Try to encode a parent store's body from its decoded form. Returns
/// `Ok(true)` when a structured encoder fired and emitted the complete
/// body; `Ok(false)` to fall through to the verbatim-with-spliced-
/// children path.
///
/// Parent encoders interleave primitive bytes (version markers, fields)
/// with recursive `write_one` calls so inline child stores emit their
/// own envelopes. The dictionary advances naturally as those recursions
/// run, which keeps subsequent stores' path encodings aligned with the
/// reader.
fn try_encode_parent_body(
    out: &mut Vec<u8>,
    node: &StoreNode,
    src: &[u8],
    env: &mut WriterEnvelope,
) -> Result<bool> {
    if matches_std_model(node) {
        if let Ok(body) = decode_std_model(src, node) {
            // 6 super-version bytes
            out.extend_from_slice(&body.super_versions);
            // Run-list-length placeholder; backpatch when known.
            let len_pos = out.len();
            out.extend_from_slice(&0i32.to_le_bytes());
            let runlist_start = out.len();

            // Walk pieces; each one contributes ano + maybe-attribute-store
            // + len + maybe-(w,h+view-store).
            let mut emitted_attrs: usize = 0;
            for piece in &body.pieces {
                match piece {
                    Piece::Text { attr_idx, wide, raw, .. } => {
                        write_ano(*attr_idx, &mut emitted_attrs, node, src, out, env)?;
                        let len: i32 = if *wide {
                            -(raw.len() as i32)
                        } else {
                            raw.len() as i32
                        };
                        out.extend_from_slice(&len.to_le_bytes());
                    }
                    Piece::View { attr_idx, w, h, child_idx, .. } => {
                        write_ano(*attr_idx, &mut emitted_attrs, node, src, out, env)?;
                        out.extend_from_slice(&0i32.to_le_bytes()); // len = 0
                        out.extend_from_slice(&w.to_le_bytes());
                        out.extend_from_slice(&h.to_le_bytes());
                        write_one(out, &node.children[*child_idx], src, env)?;
                    }
                }
            }
            // Run-list terminator + any captured padding bytes.
            out.push(0xFF);
            out.extend_from_slice(&body.runlist_padding);

            let runlist_len = (out.len() - runlist_start) as i32;
            out[len_pos..len_pos + 4].copy_from_slice(&runlist_len.to_le_bytes());

            // Char buffer: per-piece bytes + view placeholders + trailing.
            for piece in &body.pieces {
                match piece {
                    Piece::Text { raw, .. } => out.extend_from_slice(raw),
                    Piece::View { placeholder, .. } => out.push(*placeholder),
                }
            }
            out.extend_from_slice(&body.char_trailing);
            return Ok(true);
        }
    }
    if matches_fold(node) {
        if let Ok(fold) = decode_fold(src, node) {
            encode_fold_prefix(out, &fold);
            // Emit the inline hidden store (could be a nil or a
            // TextModels.StdModel). The envelope walker placed it as
            // the fold's first child in either case.
            let child = node
                .children
                .first()
                .ok_or(OdcError::Inconsistent("Fold missing inline hidden store"))?;
            write_one(out, child, src, env)?;
            return Ok(true);
        }
    }
    if matches_std_ruler(node) {
        if let Ok(body) = decode_std_ruler_body(src, node) {
            out.push(body.store_version);
            out.push(body.view_version);
            out.push(body.ruler_version);
            write_one(out, &node.children[body.child_idx], src, env)?;
            out.push(body.stdruler_version);
            return Ok(true);
        }
    }
    if matches_std_style(node) {
        if let Ok(body) = decode_std_style_body(src, node) {
            out.push(body.store_version);
            out.push(body.elem_version);
            out.push(body.model_version);
            out.push(body.style_version);
            write_one(out, &node.children[body.child_idx], src, env)?;
            out.push(body.stdstyle_version);
            return Ok(true);
        }
    }
    if matches_std_view(node) {
        if let Ok(body) = decode_std_view(src, node) {
            // All four child slots must be populated for the structured
            // encoder to fire — TextViews.StdView always has model,
            // controller (or nil), default ruler, default attrs.
            let (Some(m), Some(c), Some(r), Some(a)) = (
                body.model_child,
                body.controller_child,
                body.default_ruler_child,
                body.default_attr_child,
            ) else {
                return Ok(false);
            };
            out.push(body.store_version);
            out.push(body.view_version);
            out.push(body.container_version);
            write_one(out, &node.children[m], src, env)?;
            write_one(out, &node.children[c], src, env)?;
            out.push(body.textview_version);
            out.push(body.stdview_version);
            out.push(if body.hide_marks { 1 } else { 0 });
            write_one(out, &node.children[r], src, env)?;
            write_one(out, &node.children[a], src, env)?;
            out.extend_from_slice(&body.origin.to_le_bytes());
            out.extend_from_slice(&body.dy.to_le_bytes());
            return Ok(true);
        }
    }
    Ok(false)
}

#[inline]
fn write_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// Emit a NUL-terminated UTF-8 string. The reader uses
/// `Kernel.Utf8ToString` to decode these; we encode the original bytes
/// back as the string the reader produced. Latin-1 sequences that
/// happened to round-trip through UTF-8 land here unchanged because we
/// re-encode the original `String` as UTF-8.
fn write_utf8_sstring(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(s.as_bytes());
    out.push(0);
}
