use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hnk_idl::{DiagnosticSet, load_bundle};

use crate::graph::render_mermaid;
use crate::inspect::render_inspect_summary;
use crate::lint::lint;
use crate::normalize::normalize_bundle;
use crate::validate::validate_normalized;

#[derive(Debug, Parser)]
#[command(name = "hnk-tools")]
#[command(about = "Normalization and validation tools for Hoonoki IDL")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Validate {
        path: PathBuf,
        #[arg(long)]
        import_root: Option<PathBuf>,
    },
    Normalize {
        path: PathBuf,
        #[arg(long)]
        import_root: Option<PathBuf>,
    },
    Lint {
        path: PathBuf,
        #[arg(long)]
        import_root: Option<PathBuf>,
    },
    Graph {
        path: PathBuf,
        #[arg(long)]
        import_root: Option<PathBuf>,
    },
    Inspect {
        path: PathBuf,
        #[arg(long)]
        import_root: Option<PathBuf>,
    },
}

pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Validate { path, import_root } => {
            let normalized = load_and_normalize(&path, import_root.as_deref()).map_err(format_diagnostics)?;
            let diagnostics = validate_normalized(&normalized);
            if diagnostics.is_empty() {
                println!("Validation OK");
                Ok(())
            } else {
                Err(format_diagnostics(diagnostics))
            }
        }
        Command::Normalize { path, import_root } => {
            let normalized = load_and_normalize(&path, import_root.as_deref()).map_err(format_diagnostics)?;
            println!(
                "Normalized spec: {} events, {} components, {} connections",
                normalized.events.len(),
                normalized.components.len(),
                normalized.connections.len()
            );
            Ok(())
        }
        Command::Lint { path, import_root } => {
            let normalized = load_and_normalize(&path, import_root.as_deref()).map_err(format_diagnostics)?;
            let diagnostics = lint(&normalized);
            if diagnostics.is_empty() {
                println!("Lint OK");
            } else {
                println!("{}", format_diagnostics(diagnostics));
            }
            Ok(())
        }
        Command::Graph { path, import_root } => {
            let normalized = load_and_normalize(&path, import_root.as_deref()).map_err(format_diagnostics)?;
            println!("{}", render_mermaid(&normalized));
            Ok(())
        }
        Command::Inspect { path, import_root } => {
            let normalized = load_and_normalize(&path, import_root.as_deref()).map_err(format_diagnostics)?;
            println!("{}", render_inspect_summary(&normalized));
            Ok(())
        }
    }
}

fn load_and_normalize(
    path: &PathBuf,
    import_root: Option<&std::path::Path>,
) -> Result<crate::NormalizedSpec, DiagnosticSet> {
    let default_root = std::env::current_dir().map_err(|error| {
        DiagnosticSet::singleton(hnk_idl::Diagnostic::parse_error(
            "HNK1005",
            Some(path),
            format!("Failed to determine the current working directory: {error}"),
            None,
            None,
            Some("Pass `--import-root` explicitly.".to_string()),
        ))
    })?;
    let bundle = load_bundle(path, import_root.unwrap_or(default_root.as_path()))?;
    normalize_bundle(&bundle)
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
