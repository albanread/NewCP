//! Decode and encode `TextModels.StdModel` — the text + piece-list
//! payload.
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
//!                     <0 ⇒ 2-byte-char text run of `-len` bytes
//!                     =0 ⇒ embedded view: w: Int, h: Int, then inline view store
//!     chars     contiguous text bytes for all text runs in run-list order;
//!               each embedded view occupies one placeholder byte at its slot.
//! ```
//!
//! For round-trip the decoder records the on-wire bytes for every piece
//! (including each text run's raw bytes/words and each view's
//! placeholder byte) so the encoder can reproduce the file exactly.

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::Cursor;
use crate::text_attributes::{decode_attributes, matches_attributes, TextAttributes};

const STDMODEL_VERSION_PREFIX: usize = 6;
const ANO_TERMINATOR: u8 = 0xFF;

#[derive(Debug)]
pub struct TextModelBody {
    /// Six-byte super-class version chain at the start of the body.
    pub super_versions: [u8; STDMODEL_VERSION_PREFIX],
    pub run_list_len: u32,
    pub pieces: Vec<Piece>,
    /// Character-attribute pool, in dictionary order.
    pub attr_pool: Vec<TextAttributes>,
    pub view_children: Vec<usize>,
    /// Bytes between the run-list `0xFF` terminator and the start of
    /// the character buffer. Zero in modern files; preserved for
    /// completeness.
    pub runlist_padding: Vec<u8>,
    /// Trailing bytes after the last piece's content in the character
    /// buffer (before `body_end`). Some files reserve a few bytes for
    /// soft-EOL or padding.
    pub char_trailing: Vec<u8>,
}

#[derive(Debug)]
pub enum Piece {
    Text {
        attr_idx: usize,
        text: String,
        wide: bool,
        /// Raw on-wire bytes for this run. `wide=false` ⇒ Latin-1 / ASCII
        /// 1-byte chars (length == text length). `wide=true` ⇒ UTF-16 LE
        /// codepoints, length == 2 × text codepoints.
        raw: Vec<u8>,
    },
    View {
        attr_idx: usize,
        w: i32,
        h: i32,
        /// Index into the parent's `node.children` array.
        child_idx: usize,
        /// One-byte placeholder this view occupies in the char buffer.
        /// Conventionally `0x02` (STX) but captured verbatim for safety.
        placeholder: u8,
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

    let mut super_versions = [0u8; STDMODEL_VERSION_PREFIX];
    for slot in super_versions.iter_mut() {
        *slot = cur.read_byte()?;
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

    let mut attr_pool: Vec<TextAttributes> = Vec::new();
    let mut pieces: Vec<Piece> = Vec::new();
    let mut text_specs: Vec<(usize, u64, u32, bool)> = Vec::new();
    let mut view_pieces: Vec<(usize, u64)> = Vec::new(); // (piece_idx, placeholder_org)
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
                raw: Vec::new(),
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
                raw: Vec::new(),
            });
            text_specs.push((piece_idx, org, bytes, true));
            org += bytes as u64;
        } else {
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
            let piece_idx = pieces.len();
            pieces.push(Piece::View {
                attr_idx,
                w,
                h,
                child_idx: next_child,
                placeholder: 0,
            });
            view_pieces.push((piece_idx, org));
            view_children.push(next_child);
            next_child += 1;
            org += 1;
        }
    }

    // Capture any bytes between the terminator and chars_pos (rare, but
    // keep verbatim for safety).
    let runlist_padding_len = (chars_pos - cur.pos()) as usize;
    let runlist_padding = if runlist_padding_len > 0 {
        cur.read_bytes(runlist_padding_len)?.to_vec()
    } else {
        Vec::new()
    };

    cur.set_pos(chars_pos)?;

    // Read each text piece's raw bytes and decoded text from its
    // char-buffer org. Per-piece seek so view placeholders never leak.
    for (piece_idx, piece_org, byte_len, wide) in text_specs {
        cur.set_pos(piece_org)?;
        let bytes = cur.read_bytes(byte_len as usize)?.to_vec();
        let text = if wide {
            let words: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&words)
        } else {
            bytes.iter().map(|&b| b as char).collect()
        };
        if let Piece::Text { text: dst, raw, .. } = &mut pieces[piece_idx] {
            *dst = text;
            *raw = bytes;
        }
    }

    // Read each view piece's placeholder byte from its slot in the char
    // buffer.
    for (piece_idx, view_org) in view_pieces {
        cur.set_pos(view_org)?;
        let placeholder = cur.read_byte()?;
        if let Piece::View { placeholder: slot, .. } = &mut pieces[piece_idx] {
            *slot = placeholder;
        }
    }

    // Trailing chars that weren't covered by any piece (rare).
    let used_end = org;
    let char_trailing = if used_end < body_end {
        cur.set_pos(used_end)?;
        cur.read_bytes((body_end - used_end) as usize)?.to_vec()
    } else {
        Vec::new()
    };

    Ok(TextModelBody {
        super_versions,
        run_list_len: run_list_len as u32,
        pieces,
        attr_pool,
        view_children,
        runlist_padding,
        char_trailing,
    })
}
