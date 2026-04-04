use colored::Colorize;
use std::path::Path;

use crate::deps;
use crate::errors::{Result, RixiError};
use crate::manifest::Manifest;
use crate::paths;
use crate::registry;
use crate::snapshot;
use crate::state::State;
use crate::wallpaper;

/// Apply a rice from the rixi store. `rice` is "author/theme".
pub fn run(rice: &str) -> Result<()> {
    let parts: Vec<&str> = rice.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(RixiError::Other(
            "Rice must be specified as author/theme".to_string(),
        ));
    }

    let rice_dir = paths::store_dir().join(parts[0]).join(parts[1]);
    let manifest_path = rice_dir.join("manifest.toml");
    let manifest = Manifest::load(&manifest_path)?;
    let configs_dir = rice_dir.join("configs");

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

    // 2. Validate that component dirs exist under configs/
    for component in &manifest.meta.components {
        let component_dir = configs_dir.join(component);
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
    let snapshot_id = snapshot::create_snapshot(&manifest.meta.components)?;
    println!("{}", "done".green());
    println!();

    // 5. Apply component config files per registry paths
    println!("{}", "Applying components:".bold());
    for component in &manifest.meta.components {
        let entry = &registry[component.as_str()];
        let src_dir = configs_dir.join(component);

        let override_path = manifest.overrides.get(component);

        if let Some(custom_path) = override_path {
            let dest = paths::expand_tilde(custom_path);
            paths::ensure_dir(&dest.parent().unwrap().to_path_buf())?;
            copy_component_files(&src_dir, &[custom_path.as_str()])?;
            println!(
                "  {} {:<12} → {}",
                "✓".green().bold(),
                component,
                custom_path
            );
        } else {
            copy_component_files(&src_dir, &entry.paths)?;
            let display_path = entry.paths[0];
            println!(
                "  {} {:<12} → {}",
                "✓".green().bold(),
                component,
                display_path
            );
        }
    }

    // 6. Set wallpaper
    if let Some(ref wall_config) = manifest.wallpaper {
        println!();
        println!("{}", "Wallpaper:".bold());
        wallpaper::apply(wall_config, &rice_dir)?;
    }

    // 7. Reload components that have a reload command
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

    // 8. Update state
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

/// Copy component files from the rice configs dir to their target XDG paths.
fn copy_component_files(src_dir: &Path, target_paths: &[&str]) -> Result<()> {
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
