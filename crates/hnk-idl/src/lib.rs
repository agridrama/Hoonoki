//! Syntax-level types and parsing for Hoonoki IDL.
//!
//! The key distinction in this crate is:
//! - AST: what the YAML literally says
//! - normalized IR: a later, resolved representation built by `hnk-tools`

pub mod ast;
pub mod diagnostics;
pub mod parse;
pub mod schema;
pub mod semantics;

pub use ast::Spec;
pub use diagnostics::{Diagnostic, DiagnosticSet, Severity, SourceLocation};
pub use parse::{load_spec, parse_spec};
