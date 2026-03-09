use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

use crate::errors::Result;
use crate::manifest::{Manifest, Meta, Dependencies, Hooks};
use crate::paths;
use crate::registry;

/// Scan ~/.config for known components, detect the WM, and scaffold a manifest.toml
/// in the current directory. Also copies detected config files into component
/// subdirectories so the directory is a self-contained rice.
pub fn run() -> Result<()> {
    let registry = registry::builtin_registry();

    println!();
    println!("{}", "Scanning for known components...".bold());
    println!();

    let mut detected: Vec<String> = Vec::new();
    let mut not_found: Vec<String> = Vec::new();

    // Sort component names for consistent output
    let mut component_names: Vec<&&str> = registry.keys().collect();
    component_names.sort();

    for name in component_names {
        let entry = &registry[*name];
        // A component is "detected" if at least one of its config files exists
        let found = entry.paths.iter().any(|p| paths::expand_tilde(p).exists());

        if found {
            let first_path = entry.paths[0];
            println!(
                "  {} {:<12} {}",
                "✓".green().bold(),
                name,
                first_path.dimmed()
            );
            detected.push(name.to_string());
        } else {
            println!("  {} {:<12} {}", "✗".red(), name, "not found".dimmed());
            not_found.push(name.to_string());
        }
    }

    if detected.is_empty() {
        println!();
        println!(
            "{}",
            "No known components detected. Nothing to scaffold.".yellow()
        );
        return Ok(());
    }

    // Detect window manager from environment
    let wm = detect_wm(&detected);

    // Determine display server
    let display_server = detect_display_server(&detected);

    // Build a scaffold manifest
    let manifest = Manifest {
        meta: Meta {
            name: "my-rice".to_string(),
            author: whoami::username(),
            version: "0.1.0".to_string(),
            wm: wm.clone(),
            display_server,
            colorscheme: None,
            components: detected,
            tags: Vec::new(),
            description: Some("TODO: describe your rice".to_string()),
        },
        dependencies: Dependencies::default(),
        overrides: HashMap::new(),
        hooks: Hooks::default(),
    };

    let toml_str = manifest.to_toml_string()?;
    std::fs::write("manifest.toml", &toml_str)?;

    // Copy detected component files into subdirectories alongside the manifest
    println!();
    println!("{}", "Copying component files...".bold());
    for component in &manifest.meta.components {
        let entry = &registry[component.as_str()];
        let component_dir = Path::new(".").join(component);
        paths::ensure_dir(&component_dir)?;

        for raw_path in &entry.paths {
            let src = paths::expand_tilde(raw_path);
            if src.exists() {
                let filename = src.file_name().expect("config file should have a filename");
                let dest = component_dir.join(filename);
                std::fs::copy(&src, &dest)?;
                println!(
                    "  {} {}/{}",
                    "✓".green().bold(),
                    component,
                    filename.to_string_lossy()
                );
            }
        }
    }

    println!();
    println!(
        "{}",
        "Scaffolded manifest.toml — fill in your metadata and run rixi apply"
            .green()
            .bold()
    );

    Ok(())
}

/// Try to detect the window manager from the detected components or environment.
fn detect_wm(detected: &[String]) -> Option<String> {
    // Check environment variable first
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        let lower = desktop.to_lowercase();
        if lower.contains("bspwm") { return Some("bspwm".to_string()); }
        if lower.contains("hyprland") { return Some("hyprland".to_string()); }
        if lower.contains("sway") { return Some("sway".to_string()); }
    }

    // Fall back to detected components
    let wms = ["bspwm", "hyprland", "sway"];
    for wm in &wms {
        if detected.iter().any(|c| c == *wm) {
            return Some(wm.to_string());
        }
    }

    None
}

/// Infer display server from detected components.
fn detect_display_server(detected: &[String]) -> Vec<String> {
    let wayland_wms = ["hyprland", "sway", "waybar"];
    let x11_wms = ["bspwm", "polybar", "picom"];

    let has_wayland = detected.iter().any(|c| wayland_wms.contains(&c.as_str()));
    let has_x11 = detected.iter().any(|c| x11_wms.contains(&c.as_str()));

    match (has_wayland, has_x11) {
        (true, true) => vec!["wayland".to_string(), "x11".to_string()],
        (true, false) => vec!["wayland".to_string()],
        (false, true) => vec!["x11".to_string()],
        (false, false) => Vec::new(),
    }
}
