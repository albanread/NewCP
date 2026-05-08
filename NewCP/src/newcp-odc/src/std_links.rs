//! Decode and encode `StdLinks.Link` and `StdLinks.Target`.
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

/// Three-valued enum for which side of a paired range a piece marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// Stored payload string. Either a narrow byte sequence (XString) or a
/// wide u16 sequence (String). Each variant preserves the raw on-wire
/// content so the encoder reproduces it without re-transcoding.
#[derive(Debug, Clone)]
pub enum LinkString {
    Narrow(Vec<u8>),
    Wide(Vec<u16>),
}

impl LinkString {
    pub fn to_string(&self) -> String {
        match self {
            LinkString::Narrow(b) => match std::str::from_utf8(b) {
                Ok(s) => s.to_string(),
                Err(_) => b.iter().map(|&c| c as char).collect(),
            },
            LinkString::Wide(w) => string_as_utf16(w),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Link {
    pub store_version: u8,
    pub view_version: u8,
    pub version: u8,
    pub side: Side,
    /// Raw `cmdLen` Int read from the wire — preserved verbatim so the
    /// writer reproduces it (legacy reader uses it as buffer hint, not
    /// always equal to actual string length).
    pub cmd_len_raw: i32,
    /// Command payload for left-side links.
    pub cmd: Option<LinkString>,
    /// Closing behaviour. `None` for version 0 and right-side links.
    pub close: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub store_version: u8,
    pub view_version: u8,
    pub version: u8,
    pub side: Side,
    pub ident_len_raw: i32,
    pub ident: Option<LinkString>,
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
    let store_version = cur.read_byte()?;
    let view_version = cur.read_byte()?;
    let version = cur.read_version()?;

    let side_bool = cur.read_bool()?;
    let cmd_len_raw = cur.read_int()?;

    let cmd = if side_bool {
        if cmd_len_raw < 0 {
            return Err(OdcError::Inconsistent("link cmd length negative"));
        }
        if version <= 1 {
            Some(LinkString::Narrow(cur.read_sstring()?))
        } else {
            Some(LinkString::Wide(cur.read_string()?))
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
        store_version,
        view_version,
        version,
        side: if side_bool { Side::Left } else { Side::Right },
        cmd_len_raw,
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
    let store_version = cur.read_byte()?;
    let view_version = cur.read_byte()?;
    let version = cur.read_version()?;

    let side_bool = cur.read_bool()?;
    let ident_len_raw = cur.read_int()?;

    let ident = if side_bool {
        if version == 0 {
            Some(LinkString::Narrow(cur.read_sstring()?))
        } else {
            Some(LinkString::Wide(cur.read_string()?))
        }
    } else {
        None
    };

    Ok(Target {
        store_version,
        view_version,
        version,
        side: if side_bool { Side::Left } else { Side::Right },
        ident_len_raw,
        ident,
    })
}

pub fn encode_link(out: &mut Vec<u8>, l: &Link) {
    out.push(l.store_version);
    out.push(l.view_version);
    out.push(l.version);
    out.push(if l.side == Side::Left { 1 } else { 0 });
    out.extend_from_slice(&l.cmd_len_raw.to_le_bytes());
    if let Some(s) = &l.cmd {
        write_link_string(out, s);
    }
    if let Some(c) = l.close {
        out.extend_from_slice(&c.to_le_bytes());
    }
}

pub fn encode_target(out: &mut Vec<u8>, t: &Target) {
    out.push(t.store_version);
    out.push(t.view_version);
    out.push(t.version);
    out.push(if t.side == Side::Left { 1 } else { 0 });
    out.extend_from_slice(&t.ident_len_raw.to_le_bytes());
    if let Some(s) = &t.ident {
        write_link_string(out, s);
    }
}

fn write_link_string(out: &mut Vec<u8>, s: &LinkString) {
    match s {
        LinkString::Narrow(b) => {
            out.extend_from_slice(b);
            out.push(0);
        }
        LinkString::Wide(w) => {
            for cp in w {
                out.extend_from_slice(&cp.to_le_bytes());
            }
            out.extend_from_slice(&[0u8, 0u8]);
        }
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
