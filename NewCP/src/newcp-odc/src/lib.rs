//! Reader for BlackBox `.odc` (Component Pascal Document) files.
//!
//! Reproduces the `Stores.Reader` byte-level protocol from the legacy
//! BlackBox Component Builder so that documents can be parsed outside the
//! IDE. The current scope decodes the Stores envelope and the
//! `TextModels.StdModel` / `TextModels.Attributes` bodies that carry the
//! actual document text. Other view bodies (StdLinks, StdFolds, Controls,
//! …) remain opaque sub-stores for now and are referenced from the
//! decoded piece list by their type name and store id.

mod controls;
mod envelope;
mod error;
mod lifted;
mod primitives;
mod std_folds;
mod std_links;
mod text_attributes;
mod text_model;
mod text_rulers;
mod text_views;
mod yaml;

pub use controls::{decode_control, matches_control, Control};
pub use envelope::{read_bytes, read_document, Document, StoreKind, StoreNode};
pub use error::{OdcError, Result};
pub use lifted::{lift_text_model, LiftedPiece};
pub use std_folds::{decode_fold, matches_fold, Fold};
pub use std_links::{close_label, decode_link, decode_target, matches_link, matches_target, Link, Side, Target};
pub use text_attributes::{decode_attributes, matches_attributes, TextAttributes};
pub use text_model::{decode_std_model, matches_std_model, Piece, TextModelBody};
pub use text_rulers::{
    decode_ruler_attributes, decode_std_ruler, matches_ruler_attributes, matches_std_ruler,
    matches_std_style, Tab, TextRulerAttributes,
};
pub use text_views::{decode_std_view, matches_std_view, StdViewBody};
pub use yaml::document_to_yaml;
