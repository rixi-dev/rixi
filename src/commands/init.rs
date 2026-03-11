use colored::Colorize;
use dialoguer::{Input, theme::ColorfulTheme};
use std::collections::HashMap;
use std::path::Path;

use crate::errors::Result;
use crate::manifest::{Manifest, Meta, Dependencies, Hooks, WallpaperConfig};
use crate::paths;
use crate::registry;

/// Interactively scaffold a manifest from the user's current setup.
pub fn run(scan_path: Option<&Path>) -> Result<()> {
    let registry = registry::builtin_registry();
    let theme = ColorfulTheme::default();

    let config_dir = match scan_path {
        Some(p) => p.to_path_buf(),
        None => paths::expand_tilde("~/.config"),
    };

    println!();

    // ── Interactive prompts ───────────────────────────────────────────────────
    let name: String = Input::with_theme(&theme)
        .with_prompt("Theme name")
        .interact_text()
        .map_err(|e| crate::errors::RixiError::Other(e.to_string()))?;

    let author: String = Input::with_theme(&theme)
        .with_prompt("Author")
        .default(whoami::username())
        .interact_text()
        .map_err(|e| crate::errors::RixiError::Other(e.to_string()))?;

    let description: String = Input::with_theme(&theme)
        .with_prompt("Description")
        .interact_text()
        .map_err(|e| crate::errors::RixiError::Other(e.to_string()))?;

    let colorscheme: String = Input::with_theme(&theme)
        .with_prompt("Color scheme")
        .interact_text()
        .map_err(|e| crate::errors::RixiError::Other(e.to_string()))?;

    let tags_input: String = Input::with_theme(&theme)
        .with_prompt("Tags (comma separated)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| crate::errors::RixiError::Other(e.to_string()))?;

    let wallpaper_input: String = Input::with_theme(&theme)
        .with_prompt("Wallpaper path (leave blank to skip)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| crate::errors::RixiError::Other(e.to_string()))?;

    // ── Scan for components ───────────────────────────────────────────────────
    println!();
    print!("{}", "Scanning installed components... ".bold());

    let mut detected: Vec<String> = Vec::new();
    let mut component_names: Vec<&&str> = registry.keys().collect();
    component_names.sort();

    for name_key in &component_names {
        let entry = &registry[**name_key];
        let found = entry
            .paths
            .iter()
            .any(|p| resolve_component_path(p, &config_dir).exists());
        if found {
            detected.push(name_key.to_string());
        }
    }

    println!();
    if detected.is_empty() {
        println!(
            "  {}",
            "No known components detected.".yellow()
        );
    } else {
        let line: Vec<String> = detected
            .iter()
            .map(|c| format!("  {} {}", "✓".green().bold(), c))
            .collect();
        println!("{}", line.join("  "));
    }

    // ── Parse inputs ──────────────────────────────────────────────────────────
    let tags: Vec<String> = tags_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let wallpaper = if wallpaper_input.trim().is_empty() {
        None
    } else {
        // Auto-detect setter from installed tools
        let setter = detect_wallpaper_setter();
        Some(WallpaperConfig {
            file: wallpaper_input.trim().to_string(),
            setter,
        })
    };

    let wm = detect_wm(&detected);
    let display_server = detect_display_server(&detected, &registry);

    // ── Build manifest ────────────────────────────────────────────────────────
    let manifest = Manifest {
        meta: Meta {
            name: name.clone(),
            author: author.clone(),
            version: "0.2.0".to_string(),
            wm,
            display_server,
            colorscheme: Some(colorscheme),
            components: detected.clone(),
            tags,
            description: Some(description),
        },
        dependencies: Dependencies::default(),
        shell: None,  // detected at apply time from $SHELL
        wallpaper,
        overrides: HashMap::new(),
        hooks: Hooks::default(),
    };

    let toml_str = manifest.to_toml_string()?;
    std::fs::write("manifest.toml", &toml_str)?;

    // ── Copy component config files ───────────────────────────────────────────
    for component in &detected {
        let entry = &registry[component.as_str()];
        let component_dir = Path::new(".").join(component);
        paths::ensure_dir(&component_dir)?;

        for raw_path in &entry.paths {
            let src = resolve_component_path(raw_path, &config_dir);
            if src.exists() {
                let filename = src.file_name().expect("config file should have a filename");
                std::fs::copy(&src, component_dir.join(filename))?;
            }
        }
    }

    println!();
    println!(
        "{}",
        "Scaffolding manifest.toml... done".green().bold()
    );

    Ok(())
}

/// Try to detect the window manager from the detected components or environment.
fn detect_wm(detected: &[String]) -> Option<String> {
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        let lower = desktop.to_lowercase();
        let wms = ["bspwm", "hyprland", "sway", "i3", "openbox", "awesome",
                    "herbstluftwm", "niri", "river"];
        for wm in &wms {
            if lower.contains(wm) {
                return Some(wm.to_string());
            }
        }
    }

    let wms = ["bspwm", "hyprland", "sway", "i3", "openbox", "awesome",
               "herbstluftwm", "niri", "river"];
    for wm in &wms {
        if detected.iter().any(|c| c == *wm) {
            return Some(wm.to_string());
        }
    }

    None
}

/// Infer display server from detected components using registry display field.
fn detect_display_server(
    detected: &[String],
    registry: &HashMap<&str, crate::registry::ComponentEntry>,
) -> Vec<String> {
    let mut has_wayland = false;
    let mut has_x11 = false;

    for comp in detected {
        if let Some(entry) = registry.get(comp.as_str()) {
            match entry.display {
                "wayland" => has_wayland = true,
                "x11" => has_x11 = true,
                "both" => {
                    has_wayland = true;
                    has_x11 = true;
                }
                _ => {}
            }
        }
    }

    match (has_wayland, has_x11) {
        (true, true) => vec!["wayland".to_string(), "x11".to_string()],
        (true, false) => vec!["wayland".to_string()],
        (false, true) => vec!["x11".to_string()],
        (false, false) => Vec::new(),
    }
}

/// Detect an available wallpaper setter from PATH.
fn detect_wallpaper_setter() -> String {
    let setters = ["swww", "hyprpaper", "swaybg", "feh", "nitrogen"];
    for setter in &setters {
        if which_exists(setter) {
            return setter.to_string();
        }
    }
    "feh".to_string()
}

/// Check if a command exists in PATH.
fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Resolve a registry path (e.g. `~/.config/bspwm/bspwmrc`) against a scan directory.
/// Strips the `~/.config/` prefix and re-roots under `scan_dir`.
fn resolve_component_path(registry_path: &str, scan_dir: &Path) -> std::path::PathBuf {
    let expanded = paths::expand_tilde(registry_path);
    let config_default = paths::expand_tilde("~/.config");
    if let Ok(relative) = expanded.strip_prefix(&config_default) {
        scan_dir.join(relative)
    } else {
        expanded
    }
}
