mod commands;
mod output;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "archdoc")]
#[command(about = "Generate architecture documentation for Python projects")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize archdoc in the project
    Init {
        #[arg(short, long, default_value = ".")]
        root: String,
        #[arg(short, long, default_value = "docs/architecture")]
        out: String,
    },
    /// Generate or update documentation
    Generate {
        #[arg(short, long, default_value = ".")]
        root: String,
        #[arg(short, long, default_value = "docs/architecture")]
        out: String,
        #[arg(short, long, default_value = "archdoc.toml")]
        config: String,
    },
    /// Check if documentation is up to date
    Check {
        #[arg(short, long, default_value = ".")]
        root: String,
        #[arg(short, long, default_value = "archdoc.toml")]
        config: String,
    },
    /// Show project statistics
    Stats {
        #[arg(short, long, default_value = ".")]
        root: String,
        #[arg(short, long, default_value = "archdoc.toml")]
        config: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { root, out } => {
            commands::init::init_project(root, out)?;
        }
        Commands::Generate { root, out, config } => {
            let config = commands::generate::load_config(config)?;
            let model = commands::generate::analyze_project(root, &config)?;
            commands::generate::generate_docs(&model, out, cli.verbose)?;
            output::print_generate_summary(&model);
        }
        Commands::Check { root, config } => {
            let config = commands::generate::load_config(config)?;
            commands::check::check_docs_consistency(root, &config)?;
        }
        Commands::Stats { root, config } => {
            let config = commands::generate::load_config(config)?;
            let model = commands::generate::analyze_project(root, &config)?;
            commands::stats::print_stats(&model);
        }
    }

    Ok(())
}
