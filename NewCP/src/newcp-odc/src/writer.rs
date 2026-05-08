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

use crate::envelope::{Document, StoreKind, StoreNode, WirePath, WireTerminator};
use crate::error::{OdcError, Result};

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
    let body_pos = node.body_pos as usize;
    let body_end = (node.body_pos + node.body_len) as usize;
    if body_end > src.len() {
        return Err(OdcError::Inconsistent("body extends past source bytes"));
    }

    // Collect children sorted by header_pos. The order in `children` is
    // already document order (down/next walk), but sort defensively in
    // case a future caller ever rearranges them.
    let mut sorted: Vec<&StoreNode> = node.children.iter().collect();
    sorted.sort_by_key(|c| c.header_pos);

    let mut cur = body_pos;
    for child in sorted {
        let cstart = child.header_pos as usize;
        let cend = (child.body_pos + child.body_len) as usize;
        if cstart < cur || cstart >= body_end {
            return Err(OdcError::Inconsistent("child header_pos outside parent body"));
        }
        if cend > body_end {
            return Err(OdcError::Inconsistent("child body_end past parent body_end"));
        }
        // Copy primitive bytes that sit between the previous cursor and
        // this child verbatim. They're things like version bytes the
        // parent's Internalize wrote before reading the inline store.
        if cstart > cur {
            out.extend_from_slice(&src[cur..cstart]);
        }
        // Recurse — the child writes its own envelope plus body, with
        // the dictionary advancing in the same order the reader used.
        write_one(out, child, src, env)?;
        cur = cend;
    }
    if cur < body_end {
        out.extend_from_slice(&src[cur..body_end]);
    }
    Ok(())
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
