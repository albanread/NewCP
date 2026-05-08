//! Decode `StdLinks.Link` and `StdLinks.Target`.
//!
//! Both types are paired views: a left-side piece carries the payload and
//! a right-side piece marks where the linked range ends. The visible
//! clickable / anchored content sits in the parent text *between* the two
//! pieces. The pair-form encoding mirrors BlackBox's hand-authored
//! `<<cmd>>...<>` syntax.
//!
//! Body layout, after the 3-byte super-version prefix
//! (`Stores.Store` + `Views.View` + the type's own version byte):
//!
//! ```text
//! StdLinks.Link
//!     1 byte   sideBool          TRUE  ⇒ left side, has cmd
//!                                 FALSE ⇒ right side, no further fields
//!     4 bytes  cmdLen Int        0 if right side
//!     var      cmd               [X]String (version 0/1 narrow, version 2 wide)
//!     4 bytes  close Int         only if leftSide AND version ≥ 1
//!                                 0 = always, 1 = ifShiftDown, 2 = never
//!
//! StdLinks.Target
//!     1 byte   sideBool
//!     4 bytes  identLen Int
//!     var      ident             [X]String (version 0 narrow, version 1 wide)
//! ```

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::{string_as_utf16, Cursor};

const VIEW_PREFIX: usize = 3; // Store + View super versions, then own version byte
                              // We treat the own version as part of the prefix and read it explicitly below.

/// Three-valued enum for which side of a paired range a piece marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub version: u8,
    pub side: Side,
    /// Command string for left-side links. For right-side links this is `None`.
    pub cmd: Option<String>,
    /// Closing behaviour. `None` for version 0 and for right-side links.
    /// 0 = always close current dialog before invoking, 1 = if shift held, 2 = never.
    pub close: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub version: u8,
    pub side: Side,
    /// Anchor identifier for left-side targets.
    pub ident: Option<String>,
}

pub fn matches_link(node: &StoreNode) -> bool {
    matches!(node.type_name(), "StdLinks.LinkDesc")
}

pub fn matches_target(node: &StoreNode) -> bool {
    matches!(node.type_name(), "StdLinks.TargetDesc")
}

pub fn decode_link(file: &[u8], node: &StoreNode) -> Result<Link> {
    if !matches_link(node) {
        return Err(OdcError::Inconsistent("not a StdLinks.Link store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("link body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;

    // Skip Stores.Store + Views.View super versions.
    cur.read_byte()?;
    cur.read_byte()?;
    let version = cur.read_version()?;
    let _ = VIEW_PREFIX; // documentation marker

    let side_bool = cur.read_bool()?;
    let cmd_len = cur.read_int()?;

    let cmd = if side_bool {
        // The legacy reader allocates `cmd` of size `len`; the bytes consumed
        // by the string equal `len` regardless of which encoding is used
        // (XString = `len` bytes, String = `len` 16-bit codepoints — note this
        // means `len` for a wide string is in CHARS not bytes).
        if cmd_len < 0 {
            return Err(OdcError::Inconsistent("link cmd length negative"));
        }
        let n = cmd_len as usize;
        if version <= 1 {
            // XString: NUL-terminated 1-byte chars; reader scans until NUL.
            let raw = cur.read_sstring()?;
            // The legacy reader's `len` is the buffer size including NUL —
            // we don't validate it strictly, but expect raw.len() + 1 == n.
            let _ = n;
            Some(latin1_or_utf8(&raw))
        } else {
            // String: NUL-terminated 2-byte LE chars.
            let raw = cur.read_string()?;
            Some(string_as_utf16(&raw))
        }
    } else {
        None
    };

    let close = if side_bool && version >= 1 {
        Some(cur.read_int()?)
    } else {
        None
    };

    Ok(Link {
        version,
        side: if side_bool { Side::Left } else { Side::Right },
        cmd,
        close,
    })
}

pub fn decode_target(file: &[u8], node: &StoreNode) -> Result<Target> {
    if !matches_target(node) {
        return Err(OdcError::Inconsistent("not a StdLinks.Target store"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("target body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;
    cur.read_byte()?; // Store
    cur.read_byte()?; // View
    let version = cur.read_version()?;

    let side_bool = cur.read_bool()?;
    let _len = cur.read_int()?;

    let ident = if side_bool {
        if version == 0 {
            let raw = cur.read_sstring()?;
            Some(latin1_or_utf8(&raw))
        } else {
            let raw = cur.read_string()?;
            Some(string_as_utf16(&raw))
        }
    } else {
        None
    };

    Ok(Target {
        version,
        side: if side_bool { Side::Left } else { Side::Right },
        ident,
    })
}

/// Decode a 1-byte-per-char string. Most BlackBox cmd/ident strings are
/// pure ASCII, but some legacy files contain Latin-1 bytes (e.g. accented
/// characters in localised commands). Try UTF-8 first for forward
/// compatibility with hand-edited files, fall back to Latin-1 mapping.
fn latin1_or_utf8(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

/// Pretty-print a Link's `close` value in the BlackBox vocabulary.
pub fn close_label(close: i32) -> &'static str {
    match close {
        0 => "always",
        1 => "ifShiftDown",
        2 => "never",
        _ => "unknown",
    }
}
