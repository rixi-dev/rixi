use colored::Colorize;
use std::path::Path;

use crate::deps;
use crate::errors::{Result, RixiError};
use crate::manifest::Manifest;
use crate::paths;
use crate::registry;
use crate::shell;
use crate::snapshot;
use crate::state::State;
use crate::wallpaper;

/// Apply a rice from a local directory containing a manifest.toml.
pub fn run(rice_path: &Path) -> Result<()> {
    let manifest_path = rice_path.join("manifest.toml");
    let manifest = Manifest::load(&manifest_path)?;

    println!();
    println!(
        "{}",
        format!("Applying {}...", manifest.namespace()).bold()
    );
    println!();

    // 1. Validate that all declared components exist in the built-in registry
    let registry = registry::builtin_registry();
    for component in &manifest.meta.components {
        if !registry.contains_key(component.as_str()) {
            return Err(RixiError::UnknownComponent(component.clone()));
        }
    }

    // 2. Validate that the component files exist in the rice source directory
    for component in &manifest.meta.components {
        let component_dir = rice_path.join(component);
        if !component_dir.exists() {
            return Err(RixiError::ComponentFileMissing {
                component: component.clone(),
                path: component_dir,
            });
        }
    }

    // 3. Run dependency check and prompt user
    if !deps::check_and_prompt(&manifest.dependencies) {
        println!();
        println!("{}", "Apply cancelled.".yellow().bold());
        return Ok(());
    }
    println!();

    // 4. Snapshot current state
    print!("{}", "Snapshotting current state... ".dimmed());
    // Detect shell early so we know whether to include shell files in snapshot
    let shell_config = manifest.shell.clone().or_else(detect_shell);
    let has_shell = shell_config.is_some();
    let snapshot_id = snapshot::create_snapshot(&manifest.meta.components, has_shell)?;
    println!("{}", "done".green());
    println!();

    // 5. Apply component config files per registry paths
    println!("{}", "Applying components:".bold());
    for component in &manifest.meta.components {
        let entry = &registry[component.as_str()];

        let override_path = manifest.overrides.get(component);

        if let Some(custom_path) = override_path {
            let src_dir = rice_path.join(component);
            let dest = paths::expand_tilde(custom_path);
            paths::ensure_dir(&dest.parent().unwrap().to_path_buf())?;
            copy_component_files_from_dir(&src_dir, &[custom_path.as_str()])?;
            println!(
                "  {} {:<12} → {}",
                "✓".green().bold(),
                component,
                custom_path
            );
        } else {
            let src_dir = rice_path.join(component);
            copy_component_files_from_dir(&src_dir, &entry.paths)?;
            let display_path = entry.paths[0];
            println!(
                "  {} {:<12} → {}",
                "✓".green().bold(),
                component,
                display_path
            );
        }
    }

    // 6. Handle shell configuration — use manifest [shell] if present,
    //    otherwise use the already-detected shell config.
    if let Some(ref sc) = shell_config {
        println!();
        println!("{}", "Shell configuration:".bold());
        shell::apply(sc, &manifest.namespace())?;
    }

    // 7. Set wallpaper
    if let Some(ref wall_config) = manifest.wallpaper {
        println!();
        println!("{}", "Wallpaper:".bold());
        wallpaper::apply(wall_config, rice_path)?;
    }

    // 8. Reload components that have a reload command
    println!();
    println!("{}", "Reloading components:".bold());
    for component in &manifest.meta.components {
        let entry = &registry[component.as_str()];
        if entry.reload.is_empty() {
            println!(
                "  {} {:<12} {}",
                "–".dimmed(),
                component,
                "auto-reloads".dimmed()
            );
            continue;
        }

        let status = std::process::Command::new("sh")
            .args(["-c", entry.reload])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(exit) if exit.success() => {
                println!(
                    "  {} {:<12} {}",
                    "✓".green().bold(),
                    component,
                    entry.reload.dimmed()
                );
            }
            _ => {
                println!(
                    "  {} {:<12} {}",
                    "✗".yellow(),
                    component,
                    "reload failed (is it running?)".dimmed()
                );
            }
        }
    }

    // 9. Store rice in ~/.local/share/rixi/rices/author/theme/
    let store_dir = paths::rices_dir()
        .join(&manifest.meta.author)
        .join(&manifest.meta.name);
    if !store_dir.exists() {
        copy_dir_recursive(rice_path, &store_dir)?;
    }

    // 10. Update state
    let mut state = State::load()?;
    state.set_current(
        manifest.meta.author.clone(),
        manifest.meta.name.clone(),
        snapshot_id,
    );
    state.save()?;

    println!();
    println!(
        "{}",
        format!(
            "Applied {}. Run {} to undo.",
            manifest.namespace(),
            "rixi rollback".bold()
        )
        .green()
    );

    Ok(())
}

/// Copy component files from the rice source directory to their target XDG paths.
fn copy_component_files_from_dir(src_dir: &Path, target_paths: &[&str]) -> Result<()> {
    for raw_path in target_paths {
        let dest = paths::expand_tilde(raw_path);
        let filename = dest
            .file_name()
            .expect("target path should have a filename");
        let src = src_dir.join(filename);

        if src.exists() {
            paths::ensure_dir(&dest.parent().unwrap().to_path_buf())?;
            std::fs::copy(&src, &dest)?;
        }
    }
    Ok(())
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
