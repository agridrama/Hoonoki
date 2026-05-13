//! Semantics notes for syntax-level structures.
//!
//! This module intentionally stays light in `hnk-idl`.
//! The AST should preserve structure without committing to resolved meaning.
//! Later crates may interpret:
//! - events as inputs, observations, or timeout-driven triggers
//! - transitions as declared stateful steps
//! - contracts as either static obligations or runtime concerns
