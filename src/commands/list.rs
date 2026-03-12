use colored::Colorize;

use crate::errors::Result;
use crate::paths;
use crate::state::State;

/// List all locally stored rices under ~/.local/share/rixi/store/.
pub fn run() -> Result<()> {
    let rices_dir = paths::store_dir();

    if !rices_dir.exists() {
        println!();
        println!("{}", "No rices installed yet.".dimmed());
        return Ok(());
    }

    let state = State::load()?;
    let current_namespace = state
        .current
        .as_ref()
        .map(|c| format!("{}/{}", c.author, c.theme));

    println!();
    println!("{}", "Installed rices:".bold());

    // Iterate author dirs
    let mut found_any = false;
    for author_entry in std::fs::read_dir(&rices_dir)? {
        let author_entry = author_entry?;
        if !author_entry.file_type()?.is_dir() {
            continue;
        }
        let author = author_entry.file_name().to_string_lossy().to_string();

        // Iterate theme dirs inside each author
        for theme_entry in std::fs::read_dir(author_entry.path())? {
            let theme_entry = theme_entry?;
            if !theme_entry.file_type()?.is_dir() {
                continue;
            }
            let theme = theme_entry.file_name().to_string_lossy().to_string();
            let namespace = format!("{}/{}", author, theme);

            let marker = if current_namespace.as_deref() == Some(namespace.as_str()) {
                "  [current]".green().bold().to_string()
            } else {
                String::new()
            };

            println!("  {}{}", namespace, marker);
            found_any = true;
        }
    }

    if !found_any {
        println!("  {}", "No rices found.".dimmed());
    }

    Ok(())
}
