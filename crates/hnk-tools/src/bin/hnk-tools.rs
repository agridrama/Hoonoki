use clap::Parser;
use hnk_tools::cli::{Cli, run};

fn main() {
    let cli = Cli::parse();
    if let Err(message) = run(cli) {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
