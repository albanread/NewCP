//! Decode and encode `TextModels.Attributes` — a character-attribute
//! pool entry.
//!
//! Body layout (after the store envelope's `len` field):
//!
//! ```text
//!     1  byte   Stores.Store version       (super, always 0 in 1.7)
//!     1  byte   Attributes version
//!     4  bytes  color           (Ports.Color = i32)
//!     4  bytes  fprint          (font fingerprint, opaque to readers)
//!     var       face XString    (1-byte chars, NUL-terminated; "*" means default)
//!     4  bytes  size            (font size, in BlackBox font units)
//!     4  bytes  style           (Set, 32 bit-flags)
//!     2  bytes  weight XInt     (Fonts.normal = 400, Fonts.bold = 700, ...)
//!     4  bytes  offset          (baseline offset, INTEGER)
//! ```

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::{sstring_as_latin1, Cursor};

#[derive(Debug, Clone)]
pub struct TextAttributes {
    /// Stores.Store super-class version byte (almost always 0).
    pub store_version: u8,
    /// Attributes own version byte.
    pub attr_version: u8,
    pub color: i32,
    /// Font fingerprint — preserved verbatim. The legacy reader ignores
    /// it but the writer must reproduce it for byte-identical output.
    pub fprint: i32,
    /// Face name as raw bytes including no terminator. Reader transcodes
    /// to `font_face` lazily.
    pub face_bytes: Vec<u8>,
    pub font_face: String,
    pub font_size: i32,
    pub font_style: u32,
    pub font_weight: i16,
    pub baseline_offset: i32,
}

impl TextAttributes {
    pub fn default_marker() -> &'static str {
        "*"
    }
}

pub fn matches_attributes(node: &StoreNode) -> bool {
    matches!(node.type_name(), "TextModels.AttributesDesc")
}

pub fn decode_attributes(file: &[u8], node: &StoreNode) -> Result<TextAttributes> {
    if !matches_attributes(node) {
        return Err(OdcError::Inconsistent("not a TextModels.Attributes store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("attributes body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;
    let store_version = cur.read_version()?;
    let attr_version = cur.read_version()?;

    let color = cur.read_int()?;
    let fprint = cur.read_int()?;
    let face_bytes = cur.read_sstring()?;
    let font_face = sstring_as_latin1(&face_bytes);
    let font_size = cur.read_int()?;
    let font_style = cur.read_set()?;
    let font_weight = cur.read_xint()?;
    let baseline_offset = cur.read_int()?;

    Ok(TextAttributes {
        store_version,
        attr_version,
        color,
        fprint,
        face_bytes,
        font_face,
        font_size,
        font_style,
        font_weight,
        baseline_offset,
    })
}

/// Encode a `TextAttributes` to its on-wire byte representation.
/// Matches `TextModels.Attributes.Externalize` precisely so the output
/// is byte-identical to the original body the reader consumed.
pub fn encode_attributes(out: &mut Vec<u8>, a: &TextAttributes) {
    out.push(a.store_version);
    out.push(a.attr_version);
    out.extend_from_slice(&a.color.to_le_bytes());
    out.extend_from_slice(&a.fprint.to_le_bytes());
    out.extend_from_slice(&a.face_bytes);
    out.push(0); // SString NUL terminator
    out.extend_from_slice(&a.font_size.to_le_bytes());
    out.extend_from_slice(&a.font_style.to_le_bytes());
    out.extend_from_slice(&a.font_weight.to_le_bytes());
    out.extend_from_slice(&a.baseline_offset.to_le_bytes());
}
