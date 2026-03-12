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
    let shell_config = manifest.shell.clone().or_else(detect_shell);
    let has_shell = shell_config.is_some();
    let snapshot_id = snapshot::create_snapshot(&manifest.meta.components, has_shell)?;
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

    // 6. Handle shell configuration
    if let Some(ref sc) = shell_config {
        println!();
        println!("{}", "Shell configuration:".bold());
        shell::apply(sc, &manifest.namespace())?;
    }

    // 7. Set wallpaper
    if let Some(ref wall_config) = manifest.wallpaper {
        println!();
        println!("{}", "Wallpaper:".bold());
        wallpaper::apply(wall_config, &rice_dir)?;
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

    // 9. Update state
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

/// Detect the user's shell from $SHELL and return a ShellConfig.
fn detect_shell() -> Option<crate::manifest::ShellConfig> {
    let shell_var = std::env::var("SHELL").ok()?;
    let shell_type = if shell_var.contains("zsh") {
        "zsh"
    } else if shell_var.contains("bash") {
        "bash"
    } else if shell_var.contains("fish") {
        "fish"
    } else {
        return None;
    };

    let prompt = if which_exists("starship") { "starship" } else { "none" };

    Some(crate::manifest::ShellConfig {
        shell_type: shell_type.to_string(),
        prompt: prompt.to_string(),
    })
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
