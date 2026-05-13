use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hnk_idl::load_spec;
use hnk_tools::normalize;

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
        #[arg(long, default_value_t = true)]
        create_app_scaffold: bool,
    },
}

pub fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Generate {
            path,
            out,
            create_app_scaffold,
        } => {
            let spec = load_spec(&path).map_err(format_diagnostics)?;
            let normalized = normalize(&spec).map_err(format_diagnostics)?;
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
