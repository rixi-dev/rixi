use clap::Parser;

use rixi::cli::{Cli, Commands};
use rixi::commands;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path } => commands::init::run(path.as_deref()),
        Commands::Apply { path } => commands::apply::run(&path),
        Commands::Rollback => commands::rollback::run(),
        Commands::List => commands::list::run(),
    };

    if let Err(e) = result {
        eprintln!("\n  {} {}\n", colored::Colorize::red("error:"), e);
        std::process::exit(1);
    }
}
