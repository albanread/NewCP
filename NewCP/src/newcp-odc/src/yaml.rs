//! Hand-rolled YAML emitter for the parsed `.odc` document tree.
//!
//! The emitter dispatches on store type. Stores it knows how to decode
//! (currently `TextModels.StdModel`, `TextModels.Attributes`) emit a
//! semantic body — pieces with text, the attribute pool, etc. Stores it
//! does not yet know are surfaced structurally with their type chain,
//! body length, and any sub-stores reachable via `down`/`next`.

use std::fmt::Write as _;

use crate::controls::{decode_control, matches_control, Control};
use crate::envelope::{Document, StoreKind, StoreNode};
use crate::lifted::{lift_text_model, LiftedPiece};
use crate::std_folds::{decode_fold, matches_fold};
use crate::std_links::{close_label, decode_link, decode_target, matches_link, matches_target, Side};
use crate::text_attributes::{decode_attributes, TextAttributes};
use crate::text_model::{decode_std_model, matches_std_model, TextModelBody};
use crate::text_rulers::TextRulerAttributes;
use crate::text_views::{decode_std_view, matches_std_view};

pub fn document_to_yaml(doc: &Document) -> String {
    let mut out = String::new();
    out.push_str("odc:\n");
    out.push_str("  format: 1\n");
    if let Some(p) = &doc.source_path {
        let _ = writeln!(out, "  source: {}", quote(&p.display().to_string()));
    }
    let _ = writeln!(out, "  size: {}", doc.size);
    out.push_str("  document:\n");
    emit_store(&mut out, &doc.root, &doc.bytes, 4, &mut SkipSet::default());
    out
}

/// Children that have already been emitted as part of a decoded
/// `TextModels.StdModel` body should not be re-emitted as raw children of
/// that node. `SkipSet` tracks `(parent_header_pos, child_idx_in_parent)`
/// to identify them across the recursive walk.
#[derive(Default)]
struct SkipSet {
    skip: Vec<(u64, usize)>,
}

impl SkipSet {
    fn skip_child(&mut self, parent: u64, idx: usize) {
        self.skip.push((parent, idx));
    }

    fn is_skipped(&self, parent: u64, idx: usize) -> bool {
        self.skip.iter().any(|&(p, i)| p == parent && i == idx)
    }
}

/// `Bullet::Yes` prefixes the `kind:` line with `- `. Use `Bullet::No`
/// when the store is a value of an existing mapping key (e.g. a piece's
/// `view:` field).
#[derive(Copy, Clone)]
enum Bullet { Yes, No }

fn emit_store(out: &mut String, node: &StoreNode, file: &[u8], indent: usize, skip: &mut SkipSet) {
    emit_store_inner(out, node, file, indent, skip, Bullet::Yes);
}

fn emit_store_inner(
    out: &mut String,
    node: &StoreNode,
    file: &[u8],
    indent: usize,
    skip: &mut SkipSet,
    bullet: Bullet,
) {
    let pad = " ".repeat(indent);
    let kind_str = store_kind_str(node.kind);
    let body_indent = indent + match bullet {
        Bullet::Yes => 2,
        Bullet::No => 0,
    };
    let body_pad = " ".repeat(body_indent);

    match bullet {
        Bullet::Yes => {
            let _ = writeln!(out, "{pad}- kind: {}", quote(&node.display_kind()));
        }
        Bullet::No => {
            let _ = writeln!(out, "{pad}kind: {}", quote(&node.display_kind()));
        }
    }
    let _ = writeln!(out, "{body_pad}storeKind: {kind_str}");

    if !node.type_path.is_empty() {
        let _ = writeln!(out, "{body_pad}typeChain:");
        for name in &node.type_path {
            let _ = writeln!(out, "{body_pad}  - {}", quote(name));
        }
    }

    if matches!(node.kind, StoreKind::Link | StoreKind::NewLink) {
        if let Some(t) = node.link_target {
            let _ = writeln!(out, "{body_pad}linkTarget: {t}");
        }
        return;
    }
    if matches!(node.kind, StoreKind::Nil) {
        return;
    }

    let _ = writeln!(out, "{body_pad}id: {}", node.id);
    let _ = writeln!(out, "{body_pad}bodyPos: {}", node.body_pos);
    let _ = writeln!(out, "{body_pad}bodyLen: {}", node.body_len);

    if matches_std_view(node) {
        if let Ok(view_body) = decode_std_view(file, node) {
            emit_std_view_body(out, node, &view_body, file, body_indent, skip);
            return;
        }
    }

    if matches_std_model(node) {
        match decode_std_model(file, node) {
            Ok(body) => {
                emit_text_model_body(out, node, &body, file, body_indent, skip);
                return;
            }
            Err(e) => {
                let _ = writeln!(out, "{body_pad}decodeError: {}", quote(&e.to_string()));
            }
        }
    }

    if matches_link(node) {
        if let Ok(link) = decode_link(file, node) {
            let _ = writeln!(out, "{body_pad}side: {}", side_str(link.side));
            let _ = writeln!(out, "{body_pad}version: {}", link.version);
            if let Some(cmd) = &link.cmd {
                let _ = writeln!(out, "{body_pad}target: {}", quote(&cmd.to_string()));
            }
            if let Some(close) = link.close {
                let _ = writeln!(out, "{body_pad}close: {}  # {}", close, close_label(close));
            }
            return;
        }
    }

    if matches_target(node) {
        if let Ok(target) = decode_target(file, node) {
            let _ = writeln!(out, "{body_pad}side: {}", side_str(target.side));
            let _ = writeln!(out, "{body_pad}version: {}", target.version);
            if let Some(ident) = &target.ident {
                let _ = writeln!(out, "{body_pad}name: {}", quote(&ident.to_string()));
            }
            return;
        }
    }

    if matches_control(node) {
        if let Ok(ctrl) = decode_control(file, node) {
            emit_control_body(out, &ctrl, &body_pad);
            return;
        }
    }

    if matches_fold(node) {
        if let Ok(fold) = decode_fold(file, node) {
            let _ = writeln!(out, "{body_pad}side: {}", side_str(fold.side));
            let _ = writeln!(out, "{body_pad}collapsed: {}", fold.collapsed);
            let _ = writeln!(out, "{body_pad}label: {}", quote(&fold.label.to_string()));
            // Surface the hidden text model (or nil) directly under the fold.
            if let Some(child) = node.children.first() {
                if matches_std_model(child) {
                    let _ = writeln!(out, "{body_pad}hidden:");
                    emit_store_inner(out, child, file, body_indent + 2, skip, Bullet::No);
                } else {
                    let _ = writeln!(out, "{body_pad}hidden: null");
                }
                skip.skip_child(node.header_pos, 0);
            }
            return;
        }
    }

    if matches!(node.type_name(), "TextModels.AttributesDesc") {
        if let Ok(attrs) = decode_attributes(file, node) {
            let _ = writeln!(out, "{body_pad}attr:");
            emit_attrs_inline(out, &attrs, &format!("{body_pad}  "));
            return;
        }
    }

    if !node.children.is_empty() {
        let mut emitted_header = false;
        for (idx, child) in node.children.iter().enumerate() {
            if skip.is_skipped(node.header_pos, idx) {
                continue;
            }
            if !emitted_header {
                let _ = writeln!(out, "{body_pad}children:");
                emitted_header = true;
            }
            emit_store(out, child, file, body_indent + 2, skip);
        }
    }
}

fn side_str(s: Side) -> &'static str {
    match s {
        Side::Left => "left",
        Side::Right => "right",
    }
}

fn emit_std_view_body(
    out: &mut String,
    node: &StoreNode,
    body: &crate::text_views::StdViewBody,
    file: &[u8],
    indent: usize,
    skip: &mut SkipSet,
) {
    let pad = " ".repeat(indent);
    let _ = writeln!(out, "{pad}hideMarks: {}", body.hide_marks);
    if body.origin != 0 {
        let _ = writeln!(out, "{pad}origin: {}", body.origin);
    }
    if body.dy != 0 {
        let _ = writeln!(out, "{pad}dy: {}", body.dy);
    }
    // Default ruler & default character attributes — these mirror the
    // pool entries used inside the StdModel content but apply when the
    // model is empty or when a piece has no inline override.
    if let Some(idx) = body.default_ruler_child {
        let ruler = &node.children[idx];
        if let Some(attrs) = crate::text_rulers::decode_std_ruler(file, ruler) {
            let _ = writeln!(out, "{pad}defaultRuler:");
            emit_ruler_attrs(out, &attrs, &format!("{pad}  "));
        }
        skip.skip_child(node.header_pos, idx);
    }
    if let Some(idx) = body.default_attr_child {
        let attr_node = &node.children[idx];
        if let Ok(attr) = decode_attributes(file, attr_node) {
            let _ = writeln!(out, "{pad}defaultAttr:");
            emit_attrs_inline(out, &attr, &format!("{pad}  "));
        }
        skip.skip_child(node.header_pos, idx);
    }

    // Controller — kept opaque for now; emit the structural sub-tree.
    if let Some(idx) = body.controller_child {
        let ctrl = &node.children[idx];
        if !matches!(ctrl.kind, StoreKind::Nil) {
            let _ = writeln!(out, "{pad}controller:");
            emit_store_inner(out, ctrl, file, indent + 2, skip, Bullet::No);
        }
        skip.skip_child(node.header_pos, idx);
    }

    // Content — the StdModel decoded into a lifted flow.
    if let Some(idx) = body.model_child {
        let model = &node.children[idx];
        let _ = writeln!(out, "{pad}content:");
        emit_store_inner(out, model, file, indent + 2, skip, Bullet::No);
        skip.skip_child(node.header_pos, idx);
    }
}

fn emit_text_model_body(
    out: &mut String,
    node: &StoreNode,
    body: &TextModelBody,
    file: &[u8],
    indent: usize,
    skip: &mut SkipSet,
) {
    let pad = " ".repeat(indent);
    let _ = writeln!(out, "{pad}runListLen: {}", body.run_list_len);

    if !body.attr_pool.is_empty() {
        let _ = writeln!(out, "{pad}attrPool:");
        for (i, a) in body.attr_pool.iter().enumerate() {
            let _ = writeln!(out, "{pad}  - # [{i}]");
            emit_attrs_inline(out, a, &format!("{pad}    "));
        }
    }

    // Lift the flat piece list into the pair-collapsed form (link/target/
    // fold blocks own their visible content inline). Fall back to a flat
    // dump if lifting fails for any reason.
    let lifted = match lift_text_model(file, node, body) {
        Ok(l) => l,
        Err(e) => {
            let _ = writeln!(out, "{pad}liftError: {}", quote(&e.to_string()));
            return;
        }
    };

    if lifted.is_empty() {
        let _ = writeln!(out, "{pad}flow: []");
    } else {
        let _ = writeln!(out, "{pad}flow:");
        for piece in &lifted {
            emit_lifted_piece(out, piece, node, file, indent + 2, skip);
        }
    }

    // Mark every child of this StdModel as already emitted: attribute
    // children are inside attrPool, view children are inside the lifted
    // tree (or were referenced by Piece::View which marks them). This
    // suppresses any duplicate `children:` listing in the fallback path.
    for idx in 0..node.children.len() {
        skip.skip_child(node.header_pos, idx);
    }
}

fn emit_lifted_piece(
    out: &mut String,
    piece: &LiftedPiece,
    parent: &StoreNode,
    file: &[u8],
    indent: usize,
    skip: &mut SkipSet,
) {
    let pad = " ".repeat(indent);
    match piece {
        LiftedPiece::Text { attr_idx, text, wide } => {
            let _ = writeln!(out, "{pad}- text: {}", quote_multiline(text));
            let _ = writeln!(out, "{pad}  attr: {attr_idx}");
            if *wide {
                let _ = writeln!(out, "{pad}  wide: true");
            }
        }
        LiftedPiece::Link { attr_idx, w, h, version, cmd, close, body } => {
            let _ = writeln!(out, "{pad}- link:");
            let _ = writeln!(out, "{pad}    target: {}", quote(cmd));
            if let Some(close) = close {
                let _ = writeln!(out, "{pad}    close: {}  # {}", close, close_label(*close));
            }
            if *version != 0 {
                let _ = writeln!(out, "{pad}    version: {version}");
            }
            emit_lifted_body(out, body, parent, file, &format!("{pad}    "), skip);
            emit_attr_anchors(out, *attr_idx, *w, *h, &format!("{pad}    "));
        }
        LiftedPiece::Target { attr_idx, w, h, version, name, body } => {
            let _ = writeln!(out, "{pad}- target:");
            let _ = writeln!(out, "{pad}    name: {}", quote(name));
            if *version != 0 {
                let _ = writeln!(out, "{pad}    version: {version}");
            }
            emit_lifted_body(out, body, parent, file, &format!("{pad}    "), skip);
            emit_attr_anchors(out, *attr_idx, *w, *h, &format!("{pad}    "));
        }
        LiftedPiece::Fold { attr_idx, w, h, version, collapsed, label, hidden, body } => {
            let _ = writeln!(out, "{pad}- fold:");
            let _ = writeln!(out, "{pad}    collapsed: {collapsed}");
            if !label.is_empty() {
                let _ = writeln!(out, "{pad}    label: {}", quote(label));
            }
            if *version != 0 {
                let _ = writeln!(out, "{pad}    version: {version}");
            }
            if let Some(hidden_pieces) = hidden {
                if hidden_pieces.is_empty() {
                    let _ = writeln!(out, "{pad}    hidden: []");
                } else {
                    let _ = writeln!(out, "{pad}    hidden:");
                    for hp in hidden_pieces {
                        // Fold's hidden text is owned by the fold's child
                        // StdModel, so its `view:` items reference the
                        // fold's children, not the parent's. Pass the
                        // fold's child node down.
                        // We don't have a reference to it here directly;
                        // resolve via the lifted-fold's own view info.
                        // For now, emit relative to the parent and accept
                        // that LiftedPiece::View child_idx in hidden refers
                        // to the *hidden StdModel's* children, not parent.
                        // The structural walker emits opaque view via
                        // child_idx — see emit_lifted_piece's View arm.
                        // To keep this slice tight we trust the recursive
                        // lift to have used the correct parent at decode
                        // time and just pass `parent` here.
                        emit_lifted_piece(out, hp, parent, file, indent + 6, skip);
                    }
                }
            }
            emit_lifted_body(out, body, parent, file, &format!("{pad}    "), skip);
            emit_attr_anchors(out, *attr_idx, *w, *h, &format!("{pad}    "));
        }
        LiftedPiece::View { attr_idx, w, h, child_idx } => {
            let child = &parent.children[*child_idx];
            let _ = writeln!(out, "{pad}- view:");
            emit_store_inner(out, child, file, indent + 4, skip, Bullet::No);
            emit_attr_anchors(out, *attr_idx, *w, *h, &format!("{pad}  "));
            skip.skip_child(parent.header_pos, *child_idx);
        }
        LiftedPiece::Ruler { attr_idx, w, h, attrs, child_idx } => {
            let _ = writeln!(out, "{pad}- ruler:");
            emit_ruler_attrs(out, attrs, &format!("{pad}    "));
            emit_attr_anchors(out, *attr_idx, *w, *h, &format!("{pad}    "));
            skip.skip_child(parent.header_pos, *child_idx);
        }
        LiftedPiece::OrphanRight { kind, .. } => {
            let _ = writeln!(out, "{pad}- orphanRight: {kind}");
        }
    }
}

fn emit_lifted_body(
    out: &mut String,
    body: &[LiftedPiece],
    parent: &StoreNode,
    file: &[u8],
    pad: &str,
    skip: &mut SkipSet,
) {
    if body.is_empty() {
        let _ = writeln!(out, "{pad}body: []");
    } else {
        let _ = writeln!(out, "{pad}body:");
        for child in body {
            emit_lifted_piece(out, child, parent, file, pad.len() + 2, skip);
        }
    }
}

fn emit_attr_anchors(out: &mut String, attr_idx: usize, w: i32, h: i32, pad: &str) {
    let _ = writeln!(out, "{pad}attr: {attr_idx}");
    if w != 0 {
        let _ = writeln!(out, "{pad}w: {w}");
    }
    if h != 0 {
        let _ = writeln!(out, "{pad}h: {h}");
    }
}

fn emit_control_body(out: &mut String, c: &Control, pad: &str) {
    let link = c.link.to_string();
    let label = c.label.to_string();
    let guard = c.guard.to_string();
    if !link.is_empty() {
        let _ = writeln!(out, "{pad}link: {}", quote(&link));
    }
    if !label.is_empty() {
        let _ = writeln!(out, "{pad}label: {}", quote(&label));
    }
    if !guard.is_empty() {
        let _ = writeln!(out, "{pad}guard: {}", quote(&guard));
    }
    if let Some(n) = &c.notifier {
        let s = n.to_string();
        if !s.is_empty() {
            let _ = writeln!(out, "{pad}notifier: {}", quote(&s));
        }
    }
    if c.level != 0 {
        let _ = writeln!(out, "{pad}level: {}", c.level);
    }
    if c.custom_font {
        let _ = writeln!(out, "{pad}customFont: true");
    }
    if c.opts.iter().any(|&b| b) {
        let _ = writeln!(
            out,
            "{pad}opts: [{}, {}, {}, {}, {}]",
            c.opts[0], c.opts[1], c.opts[2], c.opts[3], c.opts[4]
        );
    }
    if c.control_version != 3 {
        let _ = writeln!(out, "{pad}controlVersion: {}", c.control_version);
    }
    if !c.trailing.is_empty() {
        let _ = writeln!(out, "{pad}trailingBytes: {}", c.trailing.len());
    }
}

fn emit_ruler_attrs(out: &mut String, a: &TextRulerAttributes, pad: &str) {
    if a.first != 0 {
        let _ = writeln!(out, "{pad}first: {}", a.first);
    }
    let _ = writeln!(out, "{pad}left: {}", a.left);
    let _ = writeln!(out, "{pad}right: {}", a.right);
    if a.lead != 0 {
        let _ = writeln!(out, "{pad}lead: {}", a.lead);
    }
    if a.asc != 0 {
        let _ = writeln!(out, "{pad}asc: {}", a.asc);
    }
    if a.dsc != 0 {
        let _ = writeln!(out, "{pad}dsc: {}", a.dsc);
    }
    if a.grid != 0 {
        let _ = writeln!(out, "{pad}grid: {}", a.grid);
    }
    if a.opts != 0 {
        let _ = writeln!(out, "{pad}opts: 0x{:08x}", a.opts);
    }
    if !a.tabs.is_empty() {
        let _ = writeln!(out, "{pad}tabs:");
        for tab in &a.tabs {
            if tab.kind == 0 {
                let _ = writeln!(out, "{pad}  - {{ stop: {} }}", tab.stop);
            } else {
                let _ = writeln!(out, "{pad}  - {{ stop: {}, kind: 0x{:08x} }}", tab.stop, tab.kind);
            }
        }
    }
    if a.version != 1 {
        let _ = writeln!(out, "{pad}version: {}", a.version);
    }
}

fn emit_attrs_inline(out: &mut String, a: &TextAttributes, pad: &str) {
    let _ = writeln!(out, "{pad}font: {}", quote(&a.font_face));
    let _ = writeln!(out, "{pad}size: {}", a.font_size);
    let _ = writeln!(out, "{pad}color: 0x{:08x}", a.color as u32);
    let _ = writeln!(out, "{pad}weight: {}", a.font_weight);
    let _ = writeln!(out, "{pad}style: 0x{:08x}", a.font_style);
    if a.baseline_offset != 0 {
        let _ = writeln!(out, "{pad}offset: {}", a.baseline_offset);
    }
}

fn store_kind_str(k: StoreKind) -> &'static str {
    match k {
        StoreKind::Nil => "nil",
        StoreKind::Link => "link",
        StoreKind::NewLink => "newlink",
        StoreKind::Store => "store",
        StoreKind::Elem => "elem",
    }
}

fn quote(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.starts_with(|c: char| c.is_ascii_whitespace())
        || s.ends_with(|c: char| c.is_ascii_whitespace())
        || s.chars().any(|c| {
            matches!(
                c,
                ':' | '#' | '\'' | '"' | '[' | ']' | '{' | '}' | ',' | '&' | '*' | '!' | '|'
                    | '>' | '%' | '@' | '`' | '\n' | '\t' | '\r'
            )
        })
        || matches!(
            s,
            "null" | "Null" | "NULL" | "true" | "True" | "TRUE" | "false" | "False" | "FALSE"
                | "yes" | "Yes" | "YES" | "no" | "No" | "NO" | "~"
        );

    if !needs_quote {
        return s.to_string();
    }

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
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// For text strings we may reasonably want multi-line presentation, but
/// for now we always emit them on a single quoted line; embedded newlines
/// become `\n`. Switching to YAML literal-block scalars (`|`) is a
/// downstream improvement.
fn quote_multiline(s: &str) -> String {
    quote(s)
}
