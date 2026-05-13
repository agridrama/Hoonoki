//! Normalization and validation for Hoonoki IDL.

pub mod cli;
pub mod contracts;
pub mod graph;
pub mod ir;
pub mod inspect;
pub mod lint;
pub mod normalize;
pub mod validate;

pub use ir::NormalizedSpec;
pub use graph::render_mermaid;
pub use inspect::render_inspect_summary;
pub use lint::lint;
pub use normalize::normalize;
pub use validate::{validate, validate_normalized};
