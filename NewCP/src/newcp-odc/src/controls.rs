//! Decode the `Controls.Control` base layout — the universal fields
//! shared by every form / dialog widget (PushButton, Caption, Field,
//! CheckBox, OptionBox, ComboBox, ListBox, TabFrame, …). Each concrete
//! subclass adds a small `Internalize2` payload with kind-specific
//! defaults; we capture the trailing bytes verbatim so they round-trip
//! even when we don't yet decode the subclass fields.
//!
//! Body layout, after the 3-byte `Stores.Store` + `Views.View` super
//! prefix and the 1-byte Control version:
//!
//! Versions 3+ (modern, found in 1.7 files):
//!
//! ```text
//!     var      link      String   (UTF-16, NUL-terminated)
//!     var      label     String
//!     var      guard     String
//!     var      notifier  String
//!     4 bytes  level     Int
//!     1 byte   customFont Bool
//!     1 byte   opt[0]    Bool   (designed for default/cancel/etc.)
//!     1 byte   opt[1]    Bool
//!     1 byte   opt[2]    Bool
//!     1 byte   opt[3]    Bool
//!     1 byte   opt[4]    Bool
//!     ?        font      ReadFont — only when customFont AND version == 4
//!     ...      Internalize2 payload (subclass-specific, captured as raw)
//! ```
//!
//! Versions 0–2 are legacy ASCII. We surface them but don't try to
//! reproduce the precise legacy quirks — they're rare in 1.7 trees.

use crate::envelope::StoreNode;
use crate::error::{OdcError, Result};
use crate::primitives::{string_as_utf16, Cursor};

#[derive(Debug, Clone)]
pub struct Control {
    pub control_version: u8,
    pub link: String,
    pub label: String,
    pub guard: String,
    pub notifier: String,
    pub level: i32,
    pub custom_font: bool,
    pub opts: [bool; 5],
    /// Bytes after the base fields belonging to the concrete subclass's
    /// own `Internalize2`. Preserved verbatim so unknown subclasses still
    /// round-trip cleanly. May also contain a font block (for v=4 files
    /// with `customFont = true`); we don't separate that out yet.
    pub trailing: Vec<u8>,
}

pub fn matches_control(node: &StoreNode) -> bool {
    let name = node.type_name();
    name.starts_with("Controls.")
        && matches!(
            name,
            "Controls.PushButtonDesc"
                | "Controls.CaptionDesc"
                | "Controls.FieldDesc"
                | "Controls.CheckBoxDesc"
                | "Controls.OptionBoxDesc"
                | "Controls.ComboBoxDesc"
                | "Controls.ListBoxDesc"
                | "Controls.SelectionDesc"
                | "Controls.GroupDesc"
                | "Controls.ControlDesc"
        )
}

pub fn decode_control(file: &[u8], node: &StoreNode) -> Result<Control> {
    if !matches_control(node) {
        return Err(OdcError::Inconsistent("not a Controls.Control"));
    }
    let body_end = node.body_pos + node.body_len;
    if body_end > file.len() as u64 {
        return Err(OdcError::Inconsistent("control body past end of file"));
    }

    let mut cur = Cursor::new(&file[..body_end as usize]);
    cur.set_pos(node.body_pos)?;
    cur.read_byte()?; // Stores.Store version
    cur.read_byte()?; // Views.View version
    let control_version = cur.read_version()?;

    if control_version >= 3 {
        let link = string_as_utf16(&cur.read_string()?);
        let label = string_as_utf16(&cur.read_string()?);
        let guard = string_as_utf16(&cur.read_string()?);
        let notifier = string_as_utf16(&cur.read_string()?);
        let level = cur.read_int()?;
        let custom_font = cur.read_bool()?;
        let opts = [
            cur.read_bool()?,
            cur.read_bool()?,
            cur.read_bool()?,
            cur.read_bool()?,
            cur.read_bool()?,
        ];
        // Skip the optional font block — version 4 with customFont reads
        // a font here. We don't decode it yet; capture as part of trailing.
        let trailing = cur
            .read_bytes((body_end - cur.pos()) as usize)?
            .to_vec();
        Ok(Control {
            control_version,
            link,
            label,
            guard,
            notifier,
            level,
            custom_font,
            opts,
            trailing,
        })
    } else {
        // Legacy v0–v2: surface basic strings and stash everything else.
        let link = sstring_to_string(cur.read_sstring()?);
        let label = sstring_to_string(cur.read_sstring()?);
        let guard = sstring_to_string(cur.read_sstring()?);
        let trailing = cur
            .read_bytes((body_end - cur.pos()) as usize)?
            .to_vec();
        Ok(Control {
            control_version,
            link,
            label,
            guard,
            notifier: String::new(),
            level: 0,
            custom_font: false,
            opts: [false; 5],
            trailing,
        })
    }
}

fn sstring_to_string(bytes: Vec<u8>) -> String {
    match std::str::from_utf8(&bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}
