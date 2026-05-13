use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hnk_idl::load_spec;

use crate::graph::render_mermaid;
use crate::inspect::render_inspect_summary;
use crate::lint::lint;
use crate::normalize::normalize;
use crate::validate::validate;

#[derive(Debug, Parser)]
#[command(name = "hnk-tools")]
#[command(about = "Normalization and validation tools for Hoonoki IDL")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Validate { path: PathBuf },
    Normalize { path: PathBuf },
    Lint { path: PathBuf },
    Graph { path: PathBuf },
    Inspect { path: PathBuf },
}

pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Validate { path } => {
            let spec = load_spec(&path).map_err(format_diagnostics)?;
            let diagnostics = validate(&spec);
            if diagnostics.is_empty() {
                println!("Validation OK");
                Ok(())
            } else {
                Err(format_diagnostics(diagnostics))
            }
        }
        Command::Normalize { path } => {
            let spec = load_spec(&path).map_err(format_diagnostics)?;
            let normalized = normalize(&spec).map_err(format_diagnostics)?;
            println!(
                "Normalized spec: {} events, {} components, {} connections, {} actors",
                normalized.events.len(),
                normalized.components.len(),
                normalized.connections.len(),
                normalized.actors.len()
            );
            Ok(())
        }
        Command::Lint { path } => {
            let spec = load_spec(&path).map_err(format_diagnostics)?;
            let normalized = normalize(&spec).map_err(format_diagnostics)?;
            let diagnostics = lint(&normalized);
            if diagnostics.is_empty() {
                println!("Lint OK");
            } else {
                println!("{}", format_diagnostics(diagnostics));
            }
            Ok(())
        }
        Command::Graph { path } => {
            let spec = load_spec(&path).map_err(format_diagnostics)?;
            let normalized = normalize(&spec).map_err(format_diagnostics)?;
            println!("{}", render_mermaid(&normalized));
            Ok(())
        }
        Command::Inspect { path } => {
            let spec = load_spec(&path).map_err(format_diagnostics)?;
            let normalized = normalize(&spec).map_err(format_diagnostics)?;
            println!("{}", render_inspect_summary(&normalized));
            Ok(())
        }
    }
}

fn format_diagnostics(diagnostics: hnk_idl::DiagnosticSet) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let prefix = match diagnostic.severity {
                hnk_idl::Severity::Error => "error",
                hnk_idl::Severity::Warning => "warning",
            };
            format!("{prefix} {}: {}", diagnostic.code, diagnostic.message)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
