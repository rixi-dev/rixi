use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// rixi — a terminal-first Linux rice manager
#[derive(Parser, Debug)]
#[command(name = "rixi", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold a manifest from your current setup
    Init {
        /// Directory to scan for components (defaults to ~/.config)
        path: Option<PathBuf>,
    },

    /// Apply a rice from a local directory containing a manifest.toml
    Apply {
        /// Path to the rice directory
        path: PathBuf,
    },

    /// Rollback to the previous state before the last apply
    Rollback,

    /// List locally installed rices
    List,
}
