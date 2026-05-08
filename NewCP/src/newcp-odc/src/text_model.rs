//! Decode `TextModels.StdModel` — the text + piece-list payload.
//!
//! Body layout (after the store envelope's `len` field):
//!
//! ```text
//!     6 bytes   super version chain:
//!                 Stores.Store         (1 byte) + isElem ⇒ +1 byte
//!                 Models.Model         (1 byte)
//!                 Containers.Model     (1 byte)
//!                 TextModels.Model     (1 byte)
//!                 TextModels.StdModel  (1 byte)
//!     4 bytes   run-list length        Int (bytes consumed by the run list)
//!     run list  sequence of:
//!                 ano: byte  (terminator = 0xFF)
//!                 IF ano = dict.len (a NEW attribute) THEN
//!                     inline TextModels.Attributes store
//!                 END
//!                 len: Int
//!                     >0 ⇒ 1-byte-char text run of `len` chars
//!                     <0 ⇒ 2-byte-char text run of `-len` bytes (-len/2 chars)
//!                     =0 ⇒ embedded view: w: Int, h: Int, then inline view store
//!     chars     contiguous text bytes for all text runs in run-list order
//! ```
//!
//! Each text piece's character buffer slice starts at `body_pos + 5 + run_list_len`
//! and grows by each run's byte length in the order the runs appear.
//!
//! The legacy reader inlines new attributes and embedded views as full
//! sub-stores (`Stores.ReadStore`) right inside the run list. Our envelope
//! reader has already captured those as children of the StdModel node, so
//! when decoding the run list we just walk `node.children` in order:
//! `TextModels.Attributes` children become attribute-pool entries, all
//! other children become embedded-view pieces. The run-list cursor jumps
//! past each inline store using the child's known `body_end` offset.

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::Cursor;
use crate::text_attributes::{decode_attributes, matches_attributes, TextAttributes};

const STDMODEL_VERSION_PREFIX: usize = 6;
const ANO_TERMINATOR: u8 = 0xFF;

#[derive(Debug)]
pub struct TextModelBody {
    pub run_list_len: u32,
    pub pieces: Vec<Piece>,
    /// Character-attribute pool, in dictionary order.
    pub attr_pool: Vec<TextAttributes>,
    /// Indices into `pool` reused across pieces. `None` if a piece has no
    /// attribute (only happens for malformed files).
    pub view_children: Vec<usize>,
}

#[derive(Debug)]
pub enum Piece {
    Text {
        attr_idx: usize,
        text: String,
        wide: bool,
    },
    View {
        attr_idx: usize,
        w: i32,
        h: i32,
        /// Index into the parent's `node.children` array.
        child_idx: usize,
    },
}

pub fn matches_std_model(node: &StoreNode) -> bool {
    matches!(node.type_name(), "TextModels.StdModelDesc")
}

pub fn decode_std_model(file: &[u8], node: &StoreNode) -> Result<TextModelBody> {
    if !matches_std_model(node) {
        return Err(OdcError::Inconsistent("not a TextModels.StdModel store"));
    }

    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("std-model body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;

    // Skip the super-version chain.
    for _ in 0..STDMODEL_VERSION_PREFIX {
        cur.read_byte()?;
    }

    let run_list_len = cur.read_int()?;
    if run_list_len < 0 {
        return Err(OdcError::Inconsistent("negative run-list length"));
    }
    let run_list_start = cur.pos();
    let chars_pos = run_list_start
        .checked_add(run_list_len as u64)
        .ok_or(OdcError::Inconsistent("run-list end overflow"))?;
    if chars_pos > body_end {
        return Err(OdcError::Inconsistent("run list runs past body end"));
    }

    // Walk the run list, building pieces and the attribute pool.
    let mut attr_pool: Vec<TextAttributes> = Vec::new();
    let mut pieces: Vec<Piece> = Vec::new();
    // Per-text-piece spec: (piece_index, char-buffer org, byte_len, wide).
    // Embedded views also consume one byte of the char buffer (the
    // placeholder character) but do not contribute a text-spec entry —
    // we just bump `org` past their slot.
    let mut text_specs: Vec<(usize, u64, u32, bool)> = Vec::new();
    let mut view_children: Vec<usize> = Vec::new();

    let mut next_child = 0usize;
    let mut org: u64 = chars_pos;

    loop {
        if cur.pos() >= chars_pos {
            return Err(OdcError::Inconsistent("run list missing terminator"));
        }
        let ano = cur.read_byte()?;
        if ano == ANO_TERMINATOR {
            break;
        }

        let attr_idx: usize = if (ano as usize) == attr_pool.len() {
            // New pool entry — the next child must be the inline
            // TextModels.Attributes store at the cursor position.
            let child = node
                .children
                .get(next_child)
                .ok_or(OdcError::Inconsistent("run list expects more attribute children"))?;
            if !matches_attributes(child) {
                return Err(OdcError::Inconsistent(
                    "run list wanted a TextModels.Attributes child but found a different kind",
                ));
            }
            if child.header_pos != cur.pos() {
                return Err(OdcError::Inconsistent(
                    "inline attribute store not at the cursor position",
                ));
            }
            let attrs = decode_attributes(file, child)?;
            // Skip over the inline store's bytes.
            cur.set_pos(child.body_pos + child.body_len)?;
            attr_pool.push(attrs);
            next_child += 1;
            attr_pool.len() - 1
        } else {
            ano as usize
        };

        let len = cur.read_int()?;
        if len > 0 {
            let piece_idx = pieces.len();
            pieces.push(Piece::Text {
                attr_idx,
                text: String::new(),
                wide: false,
            });
            text_specs.push((piece_idx, org, len as u32, false));
            org += len as u64;
        } else if len < 0 {
            let bytes = (-len) as u32;
            if bytes & 1 != 0 {
                return Err(OdcError::Inconsistent("longchar run with odd byte length"));
            }
            let piece_idx = pieces.len();
            pieces.push(Piece::Text {
                attr_idx,
                text: String::new(),
                wide: true,
            });
            text_specs.push((piece_idx, org, bytes, true));
            org += bytes as u64;
        } else {
            // len == 0 ⇒ embedded view. Reader code does INC(org) — each
            // view occupies one placeholder byte in the char buffer.
            let w = cur.read_int()?;
            let h = cur.read_int()?;
            let child = node
                .children
                .get(next_child)
                .ok_or(OdcError::Inconsistent("run list expects more view children"))?;
            if matches_attributes(child) {
                return Err(OdcError::Inconsistent(
                    "run list wanted an embedded view but the next child is a TextModels.Attributes",
                ));
            }
            if child.header_pos != cur.pos() {
                return Err(OdcError::Inconsistent(
                    "inline view store not at the cursor position",
                ));
            }
            cur.set_pos(child.body_pos + child.body_len)?;
            pieces.push(Piece::View {
                attr_idx,
                w,
                h,
                child_idx: next_child,
            });
            view_children.push(next_child);
            next_child += 1;
            org += 1;
        }
    }

    // Cursor should now be at exactly chars_pos (the writer rounds the run
    // list to use exactly run_list_len bytes).
    if cur.pos() > chars_pos {
        return Err(OdcError::Inconsistent("run list overran its declared length"));
    }
    cur.set_pos(chars_pos)?;

    // Read each text piece's bytes from its specific char-buffer org. We
    // seek per-piece because embedded view placeholders sit between text
    // runs and we don't want their bytes leaking into the next text run.
    for (piece_idx, piece_org, byte_len, wide) in text_specs {
        cur.set_pos(piece_org)?;
        let bytes = cur.read_bytes(byte_len as usize)?;
        let text = if wide {
            let words: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&words)
        } else {
            bytes.iter().map(|&b| b as char).collect()
        };
        if let Piece::Text { text: dst, .. } = &mut pieces[piece_idx] {
            *dst = text;
        }
    }

    Ok(TextModelBody {
        run_list_len: run_list_len as u32,
        pieces,
        attr_pool,
        view_children,
    })
}
