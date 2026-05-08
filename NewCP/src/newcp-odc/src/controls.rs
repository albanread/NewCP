//! Decode and encode the `Controls.Control` base layout — the
//! universal fields shared by every form / dialog widget (PushButton,
//! Caption, Field, CheckBox, OptionBox, ComboBox, ListBox, TabFrame,
//! …). Each concrete subclass adds a small `Internalize2` payload with
//! kind-specific defaults; we capture those trailing bytes verbatim so
//! they round-trip even when we don't yet decode the subclass fields.
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
//!     1 byte   opt[0]    Bool
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

/// Wide or narrow on-wire string. Stored as raw codepoints / bytes so
/// the encoder can reproduce the wire form exactly.
#[derive(Debug, Clone)]
pub enum CtrlString {
    Narrow(Vec<u8>),
    Wide(Vec<u16>),
}

impl CtrlString {
    pub fn to_string(&self) -> String {
        match self {
            CtrlString::Narrow(b) => match std::str::from_utf8(b) {
                Ok(s) => s.to_string(),
                Err(_) => b.iter().map(|&c| c as char).collect(),
            },
            CtrlString::Wide(w) => string_as_utf16(w),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Control {
    pub store_version: u8,
    pub view_version: u8,
    pub control_version: u8,
    pub link: CtrlString,
    pub label: CtrlString,
    pub guard: CtrlString,
    /// Notifier is empty for v0–v1, present from v2.
    pub notifier: Option<CtrlString>,
    pub level: i32,
    pub custom_font: bool,
    pub opts: [bool; 5],
    /// Bytes after the base fields belonging to the concrete subclass's
    /// own `Internalize2`. Preserved verbatim so unknown subclasses
    /// still round-trip cleanly. May also contain a font block (for v=4
    /// files with `customFont = true`); we don't separate that out yet.
    pub trailing: Vec<u8>,
    /// Whether this control was read along the legacy v0–v2 path. The
    /// encoder uses this to choose the right wire layout because v3+
    /// and v0–v2 are mutually exclusive shapes.
    legacy: bool,
    /// Legacy-only auxiliary fields captured during v0–v2 reads.
    legacy_v: Option<LegacyV>,
}

#[derive(Debug, Clone)]
struct LegacyV {
    /// Whether `notifier` is present in the legacy form (v1, v2 only).
    has_notifier: bool,
    /// `sort` Bool — only v1, v2.
    sort: bool,
    /// `multi_line` Bool — only v2.
    multi_line: bool,
    /// `x` Bool (free) — always.
    x: bool,
    /// `def` Bool — always.
    def: bool,
    /// `canc` Bool — always.
    canc: bool,
    /// XInt level — always.
    level_xint: i16,
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
    let store_version = cur.read_byte()?;
    let view_version = cur.read_byte()?;
    let control_version = cur.read_version()?;

    if control_version >= 3 {
        let link = CtrlString::Wide(cur.read_string()?);
        let label = CtrlString::Wide(cur.read_string()?);
        let guard = CtrlString::Wide(cur.read_string()?);
        let notifier = Some(CtrlString::Wide(cur.read_string()?));
        let level = cur.read_int()?;
        let custom_font = cur.read_bool()?;
        let opts = [
            cur.read_bool()?,
            cur.read_bool()?,
            cur.read_bool()?,
            cur.read_bool()?,
            cur.read_bool()?,
        ];
        let trailing = cur
            .read_bytes((body_end - cur.pos()) as usize)?
            .to_vec();
        Ok(Control {
            store_version,
            view_version,
            control_version,
            link,
            label,
            guard,
            notifier,
            level,
            custom_font,
            opts,
            trailing,
            legacy: false,
            legacy_v: None,
        })
    } else {
        // Legacy v0–v2 path. Field shape varies by version.
        let link = CtrlString::Narrow(cur.read_sstring()?);
        let label = CtrlString::Narrow(cur.read_sstring()?);
        let guard = CtrlString::Narrow(cur.read_sstring()?);
        let mut has_notifier = false;
        let mut notifier: Option<CtrlString> = None;
        let mut sort = false;
        let mut multi_line = false;
        if control_version == 2 {
            has_notifier = true;
            notifier = Some(CtrlString::Narrow(cur.read_sstring()?));
            sort = cur.read_bool()?;
            multi_line = cur.read_bool()?;
        } else if control_version == 1 {
            has_notifier = true;
            notifier = Some(CtrlString::Narrow(cur.read_sstring()?));
            sort = cur.read_bool()?;
        }
        let x = cur.read_bool()?;
        let def = cur.read_bool()?;
        let canc = cur.read_bool()?;
        let level_xint = cur.read_xint()?;
        let custom_font = cur.read_bool()?;
        let trailing = cur
            .read_bytes((body_end - cur.pos()) as usize)?
            .to_vec();
        Ok(Control {
            store_version,
            view_version,
            control_version,
            link,
            label,
            guard,
            notifier,
            level: level_xint as i32,
            custom_font,
            opts: [false; 5],
            trailing,
            legacy: true,
            legacy_v: Some(LegacyV {
                has_notifier,
                sort,
                multi_line,
                x,
                def,
                canc,
                level_xint,
            }),
        })
    }
}

pub fn encode_control(out: &mut Vec<u8>, c: &Control) {
    out.push(c.store_version);
    out.push(c.view_version);
    out.push(c.control_version);

    if !c.legacy {
        write_ctrl_string(out, &c.link);
        write_ctrl_string(out, &c.label);
        write_ctrl_string(out, &c.guard);
        if let Some(n) = &c.notifier {
            write_ctrl_string(out, n);
        } else {
            // Modern path always has notifier — emit empty string.
            write_ctrl_string(out, &CtrlString::Wide(Vec::new()));
        }
        out.extend_from_slice(&c.level.to_le_bytes());
        out.push(if c.custom_font { 1 } else { 0 });
        for b in &c.opts {
            out.push(if *b { 1 } else { 0 });
        }
        out.extend_from_slice(&c.trailing);
    } else {
        let lv = c
            .legacy_v
            .as_ref()
            .expect("legacy control without legacy_v");
        write_ctrl_string(out, &c.link);
        write_ctrl_string(out, &c.label);
        write_ctrl_string(out, &c.guard);
        if lv.has_notifier {
            if let Some(n) = &c.notifier {
                write_ctrl_string(out, n);
            } else {
                write_ctrl_string(out, &CtrlString::Narrow(Vec::new()));
            }
            out.push(if lv.sort { 1 } else { 0 });
            if c.control_version == 2 {
                out.push(if lv.multi_line { 1 } else { 0 });
            }
        }
        out.push(if lv.x { 1 } else { 0 });
        out.push(if lv.def { 1 } else { 0 });
        out.push(if lv.canc { 1 } else { 0 });
        out.extend_from_slice(&lv.level_xint.to_le_bytes());
        out.push(if c.custom_font { 1 } else { 0 });
        out.extend_from_slice(&c.trailing);
    }
}

fn write_ctrl_string(out: &mut Vec<u8>, s: &CtrlString) {
    match s {
        CtrlString::Narrow(b) => {
            out.extend_from_slice(b);
            out.push(0);
        }
        CtrlString::Wide(w) => {
            for cp in w {
                out.extend_from_slice(&cp.to_le_bytes());
            }
            out.extend_from_slice(&[0u8, 0u8]);
        }
    }
}
