//! Stores envelope: magic + recursive `ReadStore` / `ReadPath`.
//!
//! Mirrors the `Stores.Reader.ReadStore` and `ReadPath` procedures in the
//! BlackBox source so any `.odc` file can be parsed into a tree of
//! [`StoreNode`]s. Body bytes of each store are kept opaque for now;
//! children are recovered by following the explicit `down` (first child)
//! and `next` (next sibling) offsets the writer recorded, which lets us
//! walk the structure without understanding any individual view's
//! Internalize.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{OdcError, Result};
use crate::primitives::{sstring_as_utf8, Cursor};

const MAGIC: &[u8; 4] = b"CDOo";

const NEW_BASE: u8 = 0xF0;
const NEW_EXT: u8 = 0xF1;
const OLD_TYPE: u8 = 0xF2;

const KIND_NIL: u8 = 0x80;
const KIND_LINK: u8 = 0x81;
const KIND_STORE: u8 = 0x82;
const KIND_ELEM: u8 = 0x83;
const KIND_NEWLINK: u8 = 0x84;

const ELEM_T_NAME: &str = "Stores.ElemDesc";

/// A parsed `.odc` document. Owns the original bytes so view-body
/// decoders can revisit any region by `header_pos` / `body_pos`.
#[derive(Debug)]
pub struct Document {
    pub source_path: Option<PathBuf>,
    pub size: u64,
    pub root: StoreNode,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreKind {
    Nil,
    Link,
    NewLink,
    Store,
    Elem,
}

impl StoreKind {
    fn from_byte(b: u8, at: u64) -> Result<Self> {
        match b {
            KIND_NIL => Ok(StoreKind::Nil),
            KIND_LINK => Ok(StoreKind::Link),
            KIND_NEWLINK => Ok(StoreKind::NewLink),
            KIND_STORE => Ok(StoreKind::Store),
            KIND_ELEM => Ok(StoreKind::Elem),
            _ => Err(OdcError::BadStoreKind { at, kind: b }),
        }
    }

    pub fn is_full_store(self) -> bool {
        matches!(self, StoreKind::Store | StoreKind::Elem)
    }
}

/// Exact wire-format type-path encoding for a store, captured during
/// reading so the writer can reproduce identical bytes regardless of
/// which encoding the original writer used.
#[derive(Debug, Clone)]
pub enum WirePath {
    /// Whole chain referenced via `oldType <id>` only.
    Reference(i32),
    /// One or more `newExt` entries (in wire order) followed by the
    /// terminator. `newExt` names include `Stores.ElemDesc` verbatim
    /// when the original wire did.
    Extension {
        extensions: Vec<String>,
        terminator: WireTerminator,
    },
}

#[derive(Debug, Clone)]
pub enum WireTerminator {
    NewBase(String),
    OldType(i32),
}

#[derive(Debug)]
pub struct StoreNode {
    pub kind: StoreKind,
    /// Type chain, most-derived first (`Stores.ElemDesc` markers stripped).
    /// User-facing — for display, dispatch, and identity.
    pub type_path: Vec<String>,
    /// Original wire-format path encoding. Populated for full stores
    /// only (`Store` / `Elem`); empty enum value for nil/link/newlink.
    pub wire_path: Option<WirePath>,
    /// Absolute byte offset of the kind byte that introduced this store.
    pub header_pos: u64,
    /// Offset of the first body byte (just after `next/down/len`).
    pub body_pos: u64,
    /// `len` field — number of body bytes belonging to this store.
    pub body_len: u64,
    /// Identifier this store was assigned in the elem/store dictionary.
    pub id: i32,
    /// For Link / NewLink kinds: the referenced id.
    pub link_target: Option<i32>,
    /// Original `comment` field — preserved verbatim for round-tripping.
    pub comment: i32,
    /// Original `next` field as written, before any offset translation.
    /// Reader interprets this differently per-kind; we just record it raw.
    pub raw_next: i32,
    /// Original `down` field. 0 for kinds without one (`nil`, `link`, ...).
    pub raw_down: i32,
    /// Children walked via the `down` / `next` offset chain.
    pub children: Vec<StoreNode>,
}

impl StoreNode {
    pub fn type_name(&self) -> &str {
        if let Some(first) = self.type_path.first() {
            first
        } else {
            match self.kind {
                StoreKind::Nil => "<nil>",
                StoreKind::Link => "<link>",
                StoreKind::NewLink => "<newlink>",
                _ => "<unknown>",
            }
        }
    }

    /// Type name with the Component-Pascal `Desc` suffix stripped.
    pub fn display_kind(&self) -> String {
        let name = self.type_name();
        match name.strip_suffix("Desc") {
            Some(stripped) if !stripped.is_empty() => stripped.to_string(),
            _ => name.to_string(),
        }
    }
}

struct Envelope {
    type_dict: Vec<TypeEntry>,
    next_type_id: i32,
    next_elem_id: i32,
    next_store_id: i32,
}

#[derive(Debug, Clone)]
struct TypeEntry {
    name: String,
    /// Type id of this entry's base class within the same dictionary,
    /// or -1 if this is a root base.
    base_id: i32,
}

impl Envelope {
    fn new() -> Self {
        Self {
            type_dict: Vec::new(),
            next_type_id: 0,
            next_elem_id: 0,
            next_store_id: 0,
        }
    }
}

/// Parse a `.odc` file at `path`.
pub fn read_document(path: impl AsRef<Path>) -> Result<Document> {
    let path = path.as_ref();
    let bytes = fs::read(path)?;
    let mut doc = read_bytes(bytes)?;
    doc.source_path = Some(path.to_path_buf());
    Ok(doc)
}

/// Parse `.odc` bytes already in memory. Takes ownership so the resulting
/// [`Document`] can hand the bytes to body decoders.
pub fn read_bytes(bytes: Vec<u8>) -> Result<Document> {
    let size = bytes.len() as u64;
    if bytes.len() < 4 || &bytes[..4] != MAGIC {
        let mut found = [0u8; 4];
        for (i, b) in bytes.iter().take(4).enumerate() {
            found[i] = *b;
        }
        return Err(OdcError::BadMagic { path: None, found });
    }

    let root = {
        let mut cur = Cursor::new(&bytes);
        cur.skip(4)?; // 'CDOo'
        cur.skip(4)?; // four zero bytes seen in every document
        let mut env = Envelope::new();
        let (root, _next) = read_one(&mut cur, &mut env)?;
        root
    };

    Ok(Document { source_path: None, size, root, bytes })
}

/// Read one store at the current cursor position. Returns the parsed node
/// and the absolute file offset of its next sibling (or 0 if there is no
/// sibling — chain end). On return the cursor sits at `body_end` for full
/// stores, or just past the header for nil/link kinds.
fn read_one(cur: &mut Cursor<'_>, env: &mut Envelope) -> Result<(StoreNode, u64)> {
    let header_pos = cur.pos();
    let kind_byte = cur.read_byte()?;
    let kind = StoreKind::from_byte(kind_byte, header_pos)?;

    match kind {
        StoreKind::Nil => {
            let comment = cur.read_int()?;
            let next = cur.read_int()?;
            let pos_after_header = cur.pos();
            let next_abs = compute_next_after_header(pos_after_header, next, comment);
            Ok((
                StoreNode {
                    kind,
                    type_path: Vec::new(),
                    wire_path: None,
                    header_pos,
                    body_pos: cur.pos(),
                    body_len: 0,
                    id: -1,
                    link_target: None,
                    comment,
                    raw_next: next,
                    raw_down: 0,
                    children: Vec::new(),
                },
                next_abs,
            ))
        }
        StoreKind::Link | StoreKind::NewLink => {
            let target = cur.read_int()?;
            let comment = cur.read_int()?;
            let next = cur.read_int()?;
            let next_abs = compute_next_after_header(cur.pos(), next, comment);
            Ok((
                StoreNode {
                    kind,
                    type_path: Vec::new(),
                    wire_path: None,
                    header_pos,
                    body_pos: cur.pos(),
                    body_len: 0,
                    id: target,
                    link_target: Some(target),
                    comment,
                    raw_next: next,
                    raw_down: 0,
                    children: Vec::new(),
                },
                next_abs,
            ))
        }
        StoreKind::Store | StoreKind::Elem => {
            let id = match kind {
                StoreKind::Elem => {
                    let id = env.next_elem_id;
                    env.next_elem_id += 1;
                    id
                }
                StoreKind::Store => {
                    let id = env.next_store_id;
                    env.next_store_id += 1;
                    id
                }
                _ => unreachable!(),
            };

            let (type_path, wire_path) = read_path(cur, env)?;

            let comment = cur.read_int()?;
            let pos1 = cur.pos();
            let next = cur.read_int()?;
            let down = cur.read_int()?;
            let len = cur.read_int()?;
            if len < 0 {
                return Err(OdcError::Inconsistent("negative len"));
            }
            let body_pos = cur.pos();
            let body_end = body_pos
                .checked_add(len as u64)
                .ok_or(OdcError::Inconsistent("body length overflow"))?;

            // Stores.ReadStore: rd.st.next := pos1 + next + 4 if next > 0.
            let next_abs = if next > 0 {
                let off = pos1 as i64 + next as i64 + 4;
                if off > 0 { off as u64 } else { 0 }
            } else {
                0
            };

            let mut children = Vec::new();
            if down > 0 {
                let child_pos = pos1 as i64 + down as i64 + 8;
                if child_pos <= pos1 as i64 || child_pos >= body_end as i64 {
                    return Err(OdcError::Inconsistent("down points outside body"));
                }
                children = read_child_chain(cur, env, child_pos as u64, body_end)?;
            }

            // Always end with the cursor at body_end so the caller can use
            // sequential reads without further repositioning.
            if cur.pos() != body_end {
                cur.set_pos(body_end)?;
            }

            Ok((
                StoreNode {
                    kind,
                    type_path,
                    wire_path: Some(wire_path),
                    header_pos,
                    body_pos,
                    body_len: len as u64,
                    id,
                    link_target: None,
                    comment,
                    raw_next: next,
                    raw_down: down,
                    children,
                },
                next_abs,
            ))
        }
    }
}

/// Walk the sibling chain starting at `start`, bounded by `end`. Each
/// sibling's `next` field gives the absolute offset of the following one;
/// the chain ends when `next` is zero or refers outside `[start, end)`.
fn read_child_chain(
    cur: &mut Cursor<'_>,
    env: &mut Envelope,
    start: u64,
    end: u64,
) -> Result<Vec<StoreNode>> {
    let mut out = Vec::new();
    let saved = cur.pos();
    cur.set_pos(start)?;

    loop {
        if cur.pos() >= end {
            break;
        }
        let (node, next_abs) = read_one(cur, env)?;
        out.push(node);
        if next_abs == 0 || next_abs >= end || next_abs <= cur.pos().saturating_sub(1) && next_abs < start {
            break;
        }
        if next_abs >= end {
            break;
        }
        cur.set_pos(next_abs)?;
    }

    cur.set_pos(saved)?;
    Ok(out)
}

/// Translate a writer-encoded `next` field for the `nil`/`link`/`newlink`
/// kinds (where `pos_after_header` is the cursor position after reading
/// `next` itself). Mirrors:
///
/// ```text
/// IF (next > 0) OR ((next = 0) & ODD(comment)) THEN
///     rd.st.next := rd.st.end + next
/// ELSE rd.st.next := 0 END
/// ```
fn compute_next_after_header(pos_after_header: u64, next: i32, comment: i32) -> u64 {
    if next > 0 || (next == 0 && (comment & 1) != 0) {
        let off = pos_after_header as i64 + next as i64;
        if off > 0 { off as u64 } else { 0 }
    } else {
        0
    }
}

fn read_path(cur: &mut Cursor<'_>, env: &mut Envelope) -> Result<(Vec<String>, WirePath)> {
    let mut path: Vec<String> = Vec::new();
    // Wire-form capture, populated as we read.
    let mut wire_extensions: Vec<String> = Vec::new();
    let mut prev_idx: Option<usize> = None;

    let first_at = cur.pos();
    let mut kind = cur.read_byte()?;

    // If the very first byte is `oldType`, the entire chain is a
    // single reference — no `newExt` entries at all. Capture as such.
    if kind == OLD_TYPE && wire_extensions.is_empty() {
        let id_at = cur.pos();
        let id = cur.read_int()?;
        // Walk the chain to populate user-facing path.
        let mut walk = id;
        loop {
            if walk < 0 || (walk as usize) >= env.type_dict.len() {
                return Err(OdcError::UnknownTypeId { at: id_at, id: walk });
            }
            let entry = &env.type_dict[walk as usize];
            if entry.name != ELEM_T_NAME {
                path.push(entry.name.clone());
            }
            walk = entry.base_id;
            if walk == -1 {
                break;
            }
        }
        return Ok((path, WirePath::Reference(id)));
    }

    while kind == NEW_EXT {
        let at = cur.pos();
        let bytes = cur.read_sstring()?;
        let name = sstring_as_utf8(&bytes, at)?;

        let new_idx = env.type_dict.len();
        env.next_type_id += 1;
        env.type_dict.push(TypeEntry { name: name.clone(), base_id: -1 });
        if let Some(p) = prev_idx {
            env.type_dict[p].base_id = new_idx as i32;
        }
        prev_idx = Some(new_idx);

        wire_extensions.push(name.clone());
        if name != ELEM_T_NAME {
            path.push(name);
        }
        kind = cur.read_byte()?;
    }

    let terminator = if kind == NEW_BASE {
        let at = cur.pos();
        let bytes = cur.read_sstring()?;
        let name = sstring_as_utf8(&bytes, at)?;

        let new_idx = env.type_dict.len();
        env.next_type_id += 1;
        env.type_dict.push(TypeEntry { name: name.clone(), base_id: -1 });
        if let Some(p) = prev_idx {
            env.type_dict[p].base_id = new_idx as i32;
        }
        path.push(name.clone());
        WireTerminator::NewBase(name)
    } else if kind == OLD_TYPE {
        let id_at = cur.pos();
        let mut id = cur.read_int()?;
        let captured_id = id;
        if let Some(p) = prev_idx {
            env.type_dict[p].base_id = id;
        }
        loop {
            if id < 0 || (id as usize) >= env.type_dict.len() {
                return Err(OdcError::UnknownTypeId { at: id_at, id });
            }
            let entry = &env.type_dict[id as usize];
            if entry.name != ELEM_T_NAME {
                path.push(entry.name.clone());
            }
            id = entry.base_id;
            if id == -1 {
                break;
            }
        }
        WireTerminator::OldType(captured_id)
    } else {
        return Err(OdcError::BadPathKind { at: first_at, kind });
    };

    Ok((
        path,
        WirePath::Extension {
            extensions: wire_extensions,
            terminator,
        },
    ))
}
