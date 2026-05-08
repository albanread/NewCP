//! Lossless YAML serialisation of a parsed [`Document`] — emit and
//! parse paths both go through the canonical schema below.
//!
//! Schema (all keys lowercase camelCase, mapping order matters in
//! emitted output but is not significant for parsing):
//!
//! ```yaml
//! odc:
//!   schema: lossless
//!   format: 1
//!   source: "..."             # informational
//!   size: 5079                # informational
//!   root: <store>
//! ```
//!
//! Each store is one of:
//!
//! ```yaml
//! # store / elem
//! kind: store              # or: elem
//! typeChain:               # informational, lossy view of wirePath
//!   - "..."
//! wirePath:
//!   reference: 4           # OR
//!   extensions: ["...", "..."]
//!   terminator:
//!     newBase: "..."       # OR
//!     oldType: 4
//! id: 0
//! comment: 0
//! rawNext: 0
//! rawDown: 0
//! headerPos: 8             # absolute offset where this store's kind byte lives
//! bodyPos: 131             # absolute offset where its body bytes begin
//! bodyLen: 4948
//! bodyData: !!binary |
//!   <base64>
//! children:
//!   - <store>
//!
//! # nil
//! kind: nil
//! comment: 0
//! rawNext: 0
//!
//! # link / newlink
//! kind: link               # or: newlink
//! linkTarget: 4
//! comment: 0
//! rawNext: 0
//! ```
//!
//! The schema is verbose on purpose — `bodyData` carries every byte
//! verbatim, and the envelope fields together let the writer rebuild
//! the binary from the AST without consulting the original file.
//!
//! For human-readable / hand-editable output, see [`document_to_yaml`]
//! which emits the structural lifted form. The two formats are
//! independent.

use crate::envelope::{Document, StoreKind, StoreNode, WirePath, WireTerminator};
use crate::error::{OdcError, Result};
use crate::yaml_parse::{encode_base64, parse, YamlValue};

const BASE64_LINE_WIDTH: usize = 76;

pub fn document_to_lossless_yaml(doc: &Document) -> String {
    let mut out = String::new();
    out.push_str("odc:\n");
    out.push_str("  schema: lossless\n");
    out.push_str("  format: 1\n");
    if let Some(p) = &doc.source_path {
        out.push_str("  source: ");
        out.push_str(&quote(&p.display().to_string()));
        out.push('\n');
    }
    out.push_str(&format!("  size: {}\n", doc.size));
    out.push_str("  root:\n");
    emit_store(&mut out, &doc.root, 4, /* in_seq */ false);
    out
}

fn emit_store(out: &mut String, node: &StoreNode, indent: usize, in_seq: bool) {
    let pad = " ".repeat(indent);
    let first_pfx = if in_seq {
        format!("{}- ", " ".repeat(indent.saturating_sub(2)))
    } else {
        pad.clone()
    };
    let cont_pfx = pad.clone();

    let kind_str = kind_str(node.kind);
    out.push_str(&first_pfx);
    out.push_str("kind: ");
    out.push_str(kind_str);
    out.push('\n');

    match node.kind {
        StoreKind::Nil => {
            out.push_str(&cont_pfx);
            out.push_str(&format!("comment: {}\n", node.comment));
            out.push_str(&cont_pfx);
            out.push_str(&format!("rawNext: {}\n", node.raw_next));
            out.push_str(&cont_pfx);
            out.push_str(&format!("headerPos: {}\n", node.header_pos));
            out.push_str(&cont_pfx);
            out.push_str(&format!("bodyPos: {}\n", node.body_pos));
        }
        StoreKind::Link | StoreKind::NewLink => {
            out.push_str(&cont_pfx);
            out.push_str(&format!("linkTarget: {}\n", node.link_target.unwrap_or(node.id)));
            out.push_str(&cont_pfx);
            out.push_str(&format!("comment: {}\n", node.comment));
            out.push_str(&cont_pfx);
            out.push_str(&format!("rawNext: {}\n", node.raw_next));
            out.push_str(&cont_pfx);
            out.push_str(&format!("headerPos: {}\n", node.header_pos));
            out.push_str(&cont_pfx);
            out.push_str(&format!("bodyPos: {}\n", node.body_pos));
        }
        StoreKind::Store | StoreKind::Elem => {
            // Type chain — informational, lets a human read the YAML.
            if !node.type_path.is_empty() {
                out.push_str(&cont_pfx);
                out.push_str("typeChain:\n");
                for name in &node.type_path {
                    out.push_str(&cont_pfx);
                    out.push_str("  - ");
                    out.push_str(&quote(name));
                    out.push('\n');
                }
            }
            // Wire path — load-bearing for byte-identical round-trip.
            if let Some(wp) = &node.wire_path {
                out.push_str(&cont_pfx);
                out.push_str("wirePath:\n");
                emit_wire_path(out, wp, indent + 2);
            }
            out.push_str(&cont_pfx);
            out.push_str(&format!("id: {}\n", node.id));
            out.push_str(&cont_pfx);
            out.push_str(&format!("comment: {}\n", node.comment));
            out.push_str(&cont_pfx);
            out.push_str(&format!("rawNext: {}\n", node.raw_next));
            out.push_str(&cont_pfx);
            out.push_str(&format!("rawDown: {}\n", node.raw_down));
            out.push_str(&cont_pfx);
            out.push_str(&format!("headerPos: {}\n", node.header_pos));
            out.push_str(&cont_pfx);
            out.push_str(&format!("bodyPos: {}\n", node.body_pos));
            out.push_str(&cont_pfx);
            out.push_str(&format!("bodyLen: {}\n", node.body_len));
            // Body data — base64, wrapped to BASE64_LINE_WIDTH.
            out.push_str(&cont_pfx);
            out.push_str("bodyData: !!binary |\n");
            let b64 = encode_base64(&node.body_data);
            let inner_pad = " ".repeat(indent + 2);
            let mut i = 0;
            while i < b64.len() {
                let end = (i + BASE64_LINE_WIDTH).min(b64.len());
                out.push_str(&inner_pad);
                out.push_str(&b64[i..end]);
                out.push('\n');
                i = end;
            }
            // Children
            if node.children.is_empty() {
                out.push_str(&cont_pfx);
                out.push_str("children: []\n");
            } else {
                out.push_str(&cont_pfx);
                out.push_str("children:\n");
                let child_indent = indent + 2;
                for child in &node.children {
                    emit_store(out, child, child_indent + 2, true);
                }
            }
        }
    }
}

fn emit_wire_path(out: &mut String, wp: &WirePath, indent: usize) {
    let pad = " ".repeat(indent);
    match wp {
        WirePath::Reference(id) => {
            out.push_str(&pad);
            out.push_str(&format!("reference: {}\n", id));
        }
        WirePath::Extension { extensions, terminator } => {
            out.push_str(&pad);
            out.push_str("extensions:\n");
            for name in extensions {
                out.push_str(&pad);
                out.push_str("  - ");
                out.push_str(&quote(name));
                out.push('\n');
            }
            out.push_str(&pad);
            out.push_str("terminator:\n");
            match terminator {
                WireTerminator::NewBase(name) => {
                    out.push_str(&pad);
                    out.push_str("  newBase: ");
                    out.push_str(&quote(name));
                    out.push('\n');
                }
                WireTerminator::OldType(id) => {
                    out.push_str(&pad);
                    out.push_str(&format!("  oldType: {}\n", id));
                }
            }
        }
    }
}

fn kind_str(k: StoreKind) -> &'static str {
    match k {
        StoreKind::Nil => "nil",
        StoreKind::Link => "link",
        StoreKind::NewLink => "newlink",
        StoreKind::Store => "store",
        StoreKind::Elem => "elem",
    }
}

fn parse_kind(s: &str) -> Result<StoreKind> {
    match s {
        "nil" => Ok(StoreKind::Nil),
        "link" => Ok(StoreKind::Link),
        "newlink" => Ok(StoreKind::NewLink),
        "store" => Ok(StoreKind::Store),
        "elem" => Ok(StoreKind::Elem),
        _ => Err(OdcError::Inconsistent("unknown store kind in YAML")),
    }
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ----------------------------------------------------------------------
// Parsing path: YAML → Document
// ----------------------------------------------------------------------

pub fn document_from_lossless_yaml(input: &str) -> Result<Document> {
    let root = parse(input)?;
    let odc = root.require("odc")?;
    let schema = odc.require("schema")?.as_str()?;
    if schema != "lossless" {
        return Err(OdcError::Inconsistent("YAML is not lossless schema"));
    }
    let size = odc.require("size")?.as_int()? as u64;
    let source_path = odc
        .get("source")
        .and_then(|v| v.as_str().ok())
        .map(std::path::PathBuf::from);
    let root_node = parse_store(odc.require("root")?)?;

    // Synthesise Document.bytes such that each store's body_data sits
    // at its recorded body_pos. The bytes between body ranges are
    // envelope (kind+path+headers) and are never read by decoders —
    // the writer regenerates them from the captured fields. We allocate
    // a vector of size = max(body_pos + body_len, 8) (header floor)
    // and fill body_data slots.
    let mut max_end: u64 = size;
    walk_max_end(&root_node, &mut max_end);
    let mut bytes = vec![0u8; max_end as usize];
    place_body_data(&root_node, &mut bytes);

    Ok(Document {
        source_path,
        size,
        root: root_node,
        bytes,
    })
}

fn walk_max_end(node: &StoreNode, max: &mut u64) {
    let end = node.body_pos + node.body_len;
    if end > *max {
        *max = end;
    }
    for child in &node.children {
        walk_max_end(child, max);
    }
}

fn place_body_data(node: &StoreNode, bytes: &mut [u8]) {
    if node.body_data.len() as u64 == node.body_len && node.body_data.len() > 0 {
        let start = node.body_pos as usize;
        let end = start + node.body_data.len();
        if end <= bytes.len() {
            bytes[start..end].copy_from_slice(&node.body_data);
        }
    }
    for child in &node.children {
        place_body_data(child, bytes);
    }
}

fn parse_store(value: &YamlValue) -> Result<StoreNode> {
    let kind = parse_kind(value.require("kind")?.as_str()?)?;
    match kind {
        StoreKind::Nil => Ok(StoreNode {
            kind,
            type_path: Vec::new(),
            wire_path: None,
            header_pos: value.require("headerPos")?.as_int()? as u64,
            body_pos: value.require("bodyPos")?.as_int()? as u64,
            body_len: 0,
            id: -1,
            link_target: None,
            comment: value.require("comment")?.as_int()? as i32,
            raw_next: value.require("rawNext")?.as_int()? as i32,
            raw_down: 0,
            body_data: Vec::new(),
            children: Vec::new(),
        }),
        StoreKind::Link | StoreKind::NewLink => {
            let target = value.require("linkTarget")?.as_int()? as i32;
            Ok(StoreNode {
                kind,
                type_path: Vec::new(),
                wire_path: None,
                header_pos: value.require("headerPos")?.as_int()? as u64,
                body_pos: value.require("bodyPos")?.as_int()? as u64,
                body_len: 0,
                id: target,
                link_target: Some(target),
                comment: value.require("comment")?.as_int()? as i32,
                raw_next: value.require("rawNext")?.as_int()? as i32,
                raw_down: 0,
                body_data: Vec::new(),
                children: Vec::new(),
            })
        }
        StoreKind::Store | StoreKind::Elem => {
            let type_path = parse_type_chain(value.get("typeChain"))?;
            let wire_path = parse_wire_path(value.get("wirePath"))?;
            let id = value.require("id")?.as_int()? as i32;
            let comment = value.require("comment")?.as_int()? as i32;
            let raw_next = value.require("rawNext")?.as_int()? as i32;
            let raw_down = value.require("rawDown")?.as_int()? as i32;
            let header_pos = value.require("headerPos")?.as_int()? as u64;
            let body_pos = value.require("bodyPos")?.as_int()? as u64;
            let body_len = value.require("bodyLen")?.as_int()? as u64;
            let body_data = value.require("bodyData")?.as_bytes()?.to_vec();
            if body_data.len() as u64 != body_len {
                return Err(OdcError::Inconsistent(
                    "bodyData length doesn't match bodyLen",
                ));
            }
            let children = match value.get("children") {
                Some(YamlValue::Sequence(items)) => {
                    let mut cs = Vec::with_capacity(items.len());
                    for item in items {
                        cs.push(parse_store(item)?);
                    }
                    cs
                }
                Some(YamlValue::Null) | None => Vec::new(),
                _ => return Err(OdcError::Inconsistent("children: expected sequence")),
            };
            Ok(StoreNode {
                kind,
                type_path,
                wire_path,
                header_pos,
                body_pos,
                body_len,
                id,
                link_target: None,
                comment,
                raw_next,
                raw_down,
                body_data,
                children,
            })
        }
    }
}

fn parse_type_chain(value: Option<&YamlValue>) -> Result<Vec<String>> {
    let Some(v) = value else { return Ok(Vec::new()) };
    let seq = v.as_sequence()?;
    let mut out = Vec::with_capacity(seq.len());
    for item in seq {
        out.push(item.as_str()?.to_string());
    }
    Ok(out)
}

fn parse_wire_path(value: Option<&YamlValue>) -> Result<Option<WirePath>> {
    let Some(v) = value else { return Ok(None) };
    if let Some(id) = v.get("reference") {
        return Ok(Some(WirePath::Reference(id.as_int()? as i32)));
    }
    let extensions = v
        .get("extensions")
        .map(|e| -> Result<Vec<String>> {
            let seq = e.as_sequence()?;
            let mut xs = Vec::with_capacity(seq.len());
            for item in seq {
                xs.push(item.as_str()?.to_string());
            }
            Ok(xs)
        })
        .transpose()?
        .unwrap_or_default();
    let terminator = v.require("terminator")?;
    let term = if let Some(name) = terminator.get("newBase") {
        WireTerminator::NewBase(name.as_str()?.to_string())
    } else if let Some(id) = terminator.get("oldType") {
        WireTerminator::OldType(id.as_int()? as i32)
    } else {
        return Err(OdcError::Inconsistent("wirePath.terminator missing newBase / oldType"));
    };
    Ok(Some(WirePath::Extension { extensions, terminator: term }))
}

/// Convenience: full bin → YAML → bin round-trip with hash check.
pub fn check_yaml_roundtrip(bytes: Vec<u8>) -> Result<(bool, usize, usize)> {
    let original_len = bytes.len();
    let doc = crate::envelope::read_bytes(bytes)?;
    let yaml = document_to_lossless_yaml(&doc);
    let doc2 = document_from_lossless_yaml(&yaml)?;
    let out = crate::writer::write_document(&doc2)?;
    let matched = out == doc.bytes;
    Ok((matched, original_len, out.len()))
}
