use colored::Colorize;
use std::path::Path;

use crate::errors::{Result, RixiError};
use crate::manifest::Manifest;
use crate::paths;
use crate::registry;
use crate::snapshot;
use crate::state::State;

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

    // Validate that all declared components exist in the built-in registry
    let registry = registry::builtin_registry();
    for component in &manifest.meta.components {
        if !registry.contains_key(component.as_str()) {
            return Err(RixiError::UnknownComponent(component.clone()));
        }
    }

    // Validate that the component files exist in the rice source directory
    for component in &manifest.meta.components {
        let component_dir = rice_path.join(component);
        if !component_dir.exists() {
            return Err(RixiError::ComponentFileMissing {
                component: component.clone(),
                path: component_dir,
            });
        }
    }

    // Print missing dependencies (don't install, just warn)
    print_missing_deps(&manifest);

    // Snapshot current state
    print!("{}", "Snapshotting current state... ".dimmed());
    let snapshot_id = snapshot::create_snapshot(&manifest.meta.components)?;
    println!("{}", "done".green());
    println!();

    // Apply components: copy files from rice directory to XDG paths
    println!("{}", "Applying components:".bold());
    for component in &manifest.meta.components {
        let entry = &registry[component.as_str()];

        // Check for override path
        let override_path = manifest.overrides.get(component);

        if let Some(custom_path) = override_path {
            // Override: copy from rice dir to custom path
            let src_dir = rice_path.join(component);
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
            // Standard: copy from rice dir to registry paths
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

    // Store rice in ~/.local/share/rixi/rices/author/theme/
    let store_dir = paths::rices_dir()
        .join(&manifest.meta.author)
        .join(&manifest.meta.name);
    if !store_dir.exists() {
        copy_dir_recursive(rice_path, &store_dir)?;
    }

    // Update state
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

/// Print dependency warnings to stdout.
fn print_missing_deps(manifest: &Manifest) {
    let deps = &manifest.dependencies;
    let has_any = !deps.packages.is_empty() || !deps.fonts.is_empty() || !deps.icons.is_empty();

    if !has_any {
        return;
    }

    println!(
        "{}",
        "Missing dependencies (install manually):".yellow().bold()
    );

    if !deps.packages.is_empty() {
        println!(
            "  {} sudo pacman -S {}",
            "[pacman]".dimmed(),
            deps.packages.join(" ")
        );
    }
    if !deps.fonts.is_empty() {
        for font in &deps.fonts {
            println!("  {} {} — https://nerdfonts.com", "[fonts]".dimmed(), font);
        }
    }
    if !deps.icons.is_empty() {
        for icon in &deps.icons {
            println!("  {} {}", "[icons]".dimmed(), icon);
        }
    }
    println!();
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

/// Copy component files from the rice source directory to a list of custom paths.
fn copy_component_files(src_dir: &Path, target_paths: &[&str]) -> Result<()> {
    copy_component_files_from_dir(src_dir, target_paths)
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
