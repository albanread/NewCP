//! Pair-matching lift over a `TextModels.StdModel` piece list.
//!
//! The legacy storage represents `StdLinks.Link`, `StdLinks.Target`, and
//! `StdFolds.Fold` as paired views: a left-side view carries the payload,
//! a right-side view ends the range, and the visible content lives in the
//! parent text *between* the two pieces. The structural YAML output
//! exposes that pair-wise representation faithfully but is awkward to
//! read or edit — every cross-reference is split across three siblings.
//!
//! `lift_text_model` converts the flat piece list into a tree where each
//! pair plus its visible content collapses into a single block:
//!
//! ```yaml
//! - link:
//!     target: "StdLinks.ShowTarget('Essentials')"
//!     close: ifShiftDown
//!     body:
//!       - text: "1 Essentials"
//! ```
//!
//! Because BlackBox enforces well-formed nesting (pairs nest via their
//! own kind's stack), we do this with a single recursive descent: when
//! we encounter a left-side, recurse to gather the body, expect a
//! matching right-side, and emit a lifted block.

use crate::envelope::StoreNode;
use crate::error::Result;
use crate::std_folds::{decode_fold, matches_fold};
use crate::std_links::{decode_link, decode_target, matches_link, matches_target, Side};
use crate::text_model::{decode_std_model, matches_std_model, Piece, TextModelBody};
use crate::text_rulers::{decode_std_ruler, matches_std_ruler, TextRulerAttributes};

#[derive(Debug)]
pub enum LiftedPiece {
    Text {
        attr_idx: usize,
        text: String,
        wide: bool,
    },
    Link {
        attr_idx: usize,
        w: i32,
        h: i32,
        version: u8,
        cmd: String,
        close: Option<i32>,
        body: Vec<LiftedPiece>,
    },
    Target {
        attr_idx: usize,
        w: i32,
        h: i32,
        version: u8,
        name: String,
        body: Vec<LiftedPiece>,
    },
    Fold {
        attr_idx: usize,
        w: i32,
        h: i32,
        version: u8,
        collapsed: bool,
        label: String,
        /// Hidden text owned by the left fold (decoded recursively to
        /// lifted form). `None` if the hidden child is `<nil>` or absent.
        hidden: Option<Vec<LiftedPiece>>,
        /// Visible content between the left fold and its matching right.
        body: Vec<LiftedPiece>,
    },
    Ruler {
        attr_idx: usize,
        w: i32,
        h: i32,
        attrs: TextRulerAttributes,
        /// Index into the parent's children for the ruler node itself.
        /// The emitter uses this to mark the ruler (and its style/attrs
        /// grandchildren) as already emitted.
        child_idx: usize,
    },
    View {
        attr_idx: usize,
        w: i32,
        h: i32,
        /// Index into the parent `StoreNode.children` so the YAML emitter
        /// can recurse and dump this view's structural form.
        child_idx: usize,
    },
    /// An unmatched right-pair piece. Indicates either a malformed file or
    /// a bug in the lift; emitter renders it as a structural view marker
    /// rather than throwing the rest of the document away.
    OrphanRight {
        attr_idx: usize,
        w: i32,
        h: i32,
        child_idx: usize,
        kind: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PairKind {
    Link,
    Target,
    Fold,
}

impl PairKind {
    fn label(self) -> &'static str {
        match self {
            PairKind::Link => "link",
            PairKind::Target => "target",
            PairKind::Fold => "fold",
        }
    }
}

#[derive(Debug)]
enum Classified {
    Text {
        attr_idx: usize,
        text: String,
        wide: bool,
    },
    PairLeft {
        kind: PairKind,
        attr_idx: usize,
        w: i32,
        h: i32,
        child_idx: usize,
        link: Option<LinkPayload>,
        target: Option<TargetPayload>,
        fold: Option<FoldPayload>,
    },
    PairRight {
        kind: PairKind,
        attr_idx: usize,
        w: i32,
        h: i32,
        child_idx: usize,
    },
    Ruler {
        attr_idx: usize,
        w: i32,
        h: i32,
        child_idx: usize,
        attrs: TextRulerAttributes,
    },
    Opaque {
        attr_idx: usize,
        w: i32,
        h: i32,
        child_idx: usize,
    },
}

#[derive(Debug, Clone)]
struct LinkPayload {
    version: u8,
    cmd: String,
    close: Option<i32>,
}

#[derive(Debug, Clone)]
struct TargetPayload {
    version: u8,
    name: String,
}

#[derive(Debug)]
struct FoldPayload {
    version: u8,
    collapsed: bool,
    label: String,
    hidden: Option<Vec<LiftedPiece>>,
}

pub fn lift_text_model(
    file: &[u8],
    parent: &StoreNode,
    body: &TextModelBody,
) -> Result<Vec<LiftedPiece>> {
    let classified: Vec<Classified> = body
        .pieces
        .iter()
        .map(|p| classify(file, parent, p))
        .collect::<Result<_>>()?;

    let mut i = 0usize;
    Ok(lift_walk(&classified, &mut i))
}

fn classify(file: &[u8], parent: &StoreNode, piece: &Piece) -> Result<Classified> {
    match piece {
        Piece::Text { attr_idx, text, wide, .. } => Ok(Classified::Text {
            attr_idx: *attr_idx,
            text: text.clone(),
            wide: *wide,
        }),
        Piece::View { attr_idx, w, h, child_idx, .. } => {
            let child = &parent.children[*child_idx];

            if matches_link(child) {
                let link = decode_link(file, child)?;
                return match link.side {
                    Side::Right => Ok(Classified::PairRight {
                        kind: PairKind::Link,
                        attr_idx: *attr_idx,
                        w: *w,
                        h: *h,
                        child_idx: *child_idx,
                    }),
                    Side::Left => Ok(Classified::PairLeft {
                        kind: PairKind::Link,
                        attr_idx: *attr_idx,
                        w: *w,
                        h: *h,
                        child_idx: *child_idx,
                        link: Some(LinkPayload {
                            version: link.version,
                            cmd: link.cmd.map(|s| s.to_string()).unwrap_or_default(),
                            close: link.close,
                        }),
                        target: None,
                        fold: None,
                    }),
                };
            }

            if matches_target(child) {
                let t = decode_target(file, child)?;
                return match t.side {
                    Side::Right => Ok(Classified::PairRight {
                        kind: PairKind::Target,
                        attr_idx: *attr_idx,
                        w: *w,
                        h: *h,
                        child_idx: *child_idx,
                    }),
                    Side::Left => Ok(Classified::PairLeft {
                        kind: PairKind::Target,
                        attr_idx: *attr_idx,
                        w: *w,
                        h: *h,
                        child_idx: *child_idx,
                        link: None,
                        target: Some(TargetPayload {
                            version: t.version,
                            name: t.ident.map(|s| s.to_string()).unwrap_or_default(),
                        }),
                        fold: None,
                    }),
                };
            }

            if matches_fold(child) {
                let f = decode_fold(file, child)?;
                if f.side == Side::Right {
                    return Ok(Classified::PairRight {
                        kind: PairKind::Fold,
                        attr_idx: *attr_idx,
                        w: *w,
                        h: *h,
                        child_idx: *child_idx,
                    });
                }
                // Left fold: decode its hidden TextModels.Model, if any,
                // and recursively lift it so YAML can show what's hidden.
                let hidden = if let Some(hidden_child) = child.children.first() {
                    if matches_std_model(hidden_child) {
                        let hidden_body = decode_std_model(file, hidden_child)?;
                        Some(lift_text_model(file, hidden_child, &hidden_body)?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                return Ok(Classified::PairLeft {
                    kind: PairKind::Fold,
                    attr_idx: *attr_idx,
                    w: *w,
                    h: *h,
                    child_idx: *child_idx,
                    link: None,
                    target: None,
                    fold: Some(FoldPayload {
                        version: f.version,
                        collapsed: f.collapsed,
                        label: f.label.to_string(),
                        hidden,
                    }),
                });
            }

            if matches_std_ruler(child) {
                if let Some(attrs) = decode_std_ruler(file, child) {
                    return Ok(Classified::Ruler {
                        attr_idx: *attr_idx,
                        w: *w,
                        h: *h,
                        child_idx: *child_idx,
                        attrs,
                    });
                }
            }

            Ok(Classified::Opaque {
                attr_idx: *attr_idx,
                w: *w,
                h: *h,
                child_idx: *child_idx,
            })
        }
    }
}

fn lift_walk(classified: &[Classified], i: &mut usize) -> Vec<LiftedPiece> {
    let mut out: Vec<LiftedPiece> = Vec::new();

    while *i < classified.len() {
        match &classified[*i] {
            Classified::Text { attr_idx, text, wide } => {
                out.push(LiftedPiece::Text {
                    attr_idx: *attr_idx,
                    text: text.clone(),
                    wide: *wide,
                });
                *i += 1;
            }
            Classified::Opaque { attr_idx, w, h, child_idx } => {
                out.push(LiftedPiece::View {
                    attr_idx: *attr_idx,
                    w: *w,
                    h: *h,
                    child_idx: *child_idx,
                });
                *i += 1;
            }
            Classified::Ruler { attr_idx, w, h, child_idx, attrs } => {
                out.push(LiftedPiece::Ruler {
                    attr_idx: *attr_idx,
                    w: *w,
                    h: *h,
                    child_idx: *child_idx,
                    attrs: attrs.clone(),
                });
                *i += 1;
            }
            Classified::PairRight { kind, attr_idx, w, h, child_idx } => {
                // We've reached a right-side without a matching open pair
                // at this level — return so the caller handles it.
                let _ = kind;
                let _ = attr_idx;
                let _ = w;
                let _ = h;
                let _ = child_idx;
                return out;
            }
            Classified::PairLeft { .. } => {
                // Take ownership of the left-side data, recurse for body,
                // then consume the matching right-side.
                let (kind, attr_idx, w, h, link, target, fold) = match &classified[*i] {
                    Classified::PairLeft {
                        kind,
                        attr_idx,
                        w,
                        h,
                        link,
                        target,
                        fold,
                        ..
                    } => (
                        *kind,
                        *attr_idx,
                        *w,
                        *h,
                        link.clone(),
                        target.clone(),
                        fold.as_ref().map(|f| FoldPayload {
                            version: f.version,
                            collapsed: f.collapsed,
                            label: f.label.clone(),
                            hidden: f.hidden.clone(),
                        }),
                    ),
                    _ => unreachable!(),
                };
                *i += 1;
                let body = lift_walk(classified, i);

                // Consume the matching right-side if present.
                if let Some(Classified::PairRight { kind: rk, .. }) = classified.get(*i) {
                    if *rk == kind {
                        *i += 1;
                    }
                }

                let lifted = match kind {
                    PairKind::Link => {
                        let p = link.unwrap_or(LinkPayload {
                            version: 0,
                            cmd: String::new(),
                            close: None,
                        });
                        LiftedPiece::Link {
                            attr_idx,
                            w,
                            h,
                            version: p.version,
                            cmd: p.cmd,
                            close: p.close,
                            body,
                        }
                    }
                    PairKind::Target => {
                        let p = target.unwrap_or(TargetPayload {
                            version: 0,
                            name: String::new(),
                        });
                        LiftedPiece::Target {
                            attr_idx,
                            w,
                            h,
                            version: p.version,
                            name: p.name,
                            body,
                        }
                    }
                    PairKind::Fold => {
                        let p = fold.unwrap_or(FoldPayload {
                            version: 0,
                            collapsed: false,
                            label: String::new(),
                            hidden: None,
                        });
                        LiftedPiece::Fold {
                            attr_idx,
                            w,
                            h,
                            version: p.version,
                            collapsed: p.collapsed,
                            label: p.label,
                            hidden: p.hidden,
                            body,
                        }
                    }
                };
                out.push(lifted);
            }
        }
    }

    out
}

// `LiftedPiece` is `Clone`-by-hand because of the recursive `Vec<Self>`
// fields; `Fold.hidden` clones whole sub-trees when classify hands the
// payload to `lift_walk`. Not used in steady state — only during the
// PairLeft → owned-payload move above. Kept narrow on purpose.
impl Clone for LiftedPiece {
    fn clone(&self) -> Self {
        match self {
            LiftedPiece::Text { attr_idx, text, wide } => LiftedPiece::Text {
                attr_idx: *attr_idx,
                text: text.clone(),
                wide: *wide,
            },
            LiftedPiece::Link {
                attr_idx,
                w,
                h,
                version,
                cmd,
                close,
                body,
            } => LiftedPiece::Link {
                attr_idx: *attr_idx,
                w: *w,
                h: *h,
                version: *version,
                cmd: cmd.clone(),
                close: *close,
                body: body.clone(),
            },
            LiftedPiece::Target {
                attr_idx,
                w,
                h,
                version,
                name,
                body,
            } => LiftedPiece::Target {
                attr_idx: *attr_idx,
                w: *w,
                h: *h,
                version: *version,
                name: name.clone(),
                body: body.clone(),
            },
            LiftedPiece::Fold {
                attr_idx,
                w,
                h,
                version,
                collapsed,
                label,
                hidden,
                body,
            } => LiftedPiece::Fold {
                attr_idx: *attr_idx,
                w: *w,
                h: *h,
                version: *version,
                collapsed: *collapsed,
                label: label.clone(),
                hidden: hidden.clone(),
                body: body.clone(),
            },
            LiftedPiece::Ruler { attr_idx, w, h, attrs, child_idx } => LiftedPiece::Ruler {
                attr_idx: *attr_idx,
                w: *w,
                h: *h,
                attrs: attrs.clone(),
                child_idx: *child_idx,
            },
            LiftedPiece::View { attr_idx, w, h, child_idx } => LiftedPiece::View {
                attr_idx: *attr_idx,
                w: *w,
                h: *h,
                child_idx: *child_idx,
            },
            LiftedPiece::OrphanRight {
                attr_idx,
                w,
                h,
                child_idx,
                kind,
            } => LiftedPiece::OrphanRight {
                attr_idx: *attr_idx,
                w: *w,
                h: *h,
                child_idx: *child_idx,
                kind,
            },
        }
    }
}
