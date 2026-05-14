use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hnk_idl::{DiagnosticSet, load_bundle};
use hnk_tools::normalize_bundle;

use crate::generate::{GenerateOptions, generate};

#[derive(Debug, Parser)]
#[command(name = "hnk-core")]
#[command(about = "Rust scaffold generation for Hoonoki IDL")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Generate {
        path: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        import_root: Option<PathBuf>,
        #[arg(long, default_value_t = true)]
        create_app_scaffold: bool,
    },
}

pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Generate {
            path,
            out,
            import_root,
            create_app_scaffold,
        } => {
            let normalized = load_and_normalize(&path, import_root.as_deref()).map_err(format_diagnostics)?;
            let report = generate(
                &normalized,
                &out,
                GenerateOptions {
                    create_app_scaffold,
                    ..GenerateOptions::default()
                },
            )
            .map_err(|error| error.to_string())?;

            println!(
                "Generated {} components into {}",
                report.component_count,
                out.display()
            );
            println!("Created files: {}", report.created_files.len());
            println!("Updated files: {}", report.updated_files.len());
            println!("Preserved files: {}", report.preserved_files.len());
            Ok(())
        }
    }
}

fn load_and_normalize(
    path: &PathBuf,
    import_root: Option<&std::path::Path>,
) -> Result<hnk_tools::NormalizedSpec, DiagnosticSet> {
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
