use colored::Colorize;

use crate::errors::{Result, RixiError};
use crate::snapshot;
use crate::state::State;

/// Rollback to the state before the last `rixi apply`.
pub fn run() -> Result<()> {
    let mut state = State::load()?;

    let current = state
        .current
        .as_ref()
        .ok_or(RixiError::NothingToRollback)?;

    let snapshot_id = current.snapshot.clone();

    println!();
    println!(
        "{}",
        format!("Rolling back to snapshot {}...", snapshot_id).bold()
    );

    let restored = snapshot::restore_snapshot(&snapshot_id)?;

    for component in &restored {
        println!("  {} {:<12} {}", "✓".green().bold(), component, "restored");
    }

    // Clear the current applied rice
    state.clear_current();
    state.save()?;

    println!();
    println!("{}", "Rollback complete.".green().bold());

    Ok(())
}
