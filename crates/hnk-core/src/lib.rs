//! Rust scaffold generation for Hoonoki normalized specs.

pub mod cli;
pub mod generate;
pub mod runtime_contracts;
pub mod rust;

pub use generate::{
    GenerateError, GenerateOptions, GenerateReport, GeneratedFileStatus, generate,
};
