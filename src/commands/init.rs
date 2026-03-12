use colored::Colorize;
use dialoguer::{Input, theme::ColorfulTheme};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::errors::Result;
use crate::manifest::{Manifest, Meta, Dependencies, Hooks, WallpaperConfig};
use crate::paths;
use crate::registry;

/// Known component directory names that rixi recognizes in ~/.config/.
const KNOWN_COMPONENTS: &[&str] = &[
    "bspwm", "i3", "openbox", "awesome", "herbstluftwm", "hypr", "sway", "river", "niri",
    "waybar", "polybar", "eww", "rofi", "wofi", "tofi", "fuzzel", "kitty", "alacritty",
    "wezterm", "foot", "dunst", "mako", "swaync", "picom", "swaylock", "fish", "starship",
];

/// A detected component: name + the source directory holding its config files.
struct DetectedComponent {
    name: String,
    source_dir: PathBuf,
}

/// Interactively scaffold a rice into ~/.local/share/rixi/store/<author>/<theme>/.
pub fn run(_scan_path: Option<&Path>) -> Result<()> {
    let registry = registry::builtin_registry();
    let theme = ColorfulTheme::default();
    let config_dir = paths::expand_tilde("~/.config");

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
    print!("{}", "Scanning ~/.config for components...".bold());

    let detected = scan_components(&config_dir);

    println!();
    if detected.is_empty() {
        println!("  {}", "No known components detected.".yellow());
    } else {
        for comp in &detected {
            println!("  {} {}", "✓".green().bold(), comp.name);
        }
    }

    let detected_names: Vec<String> = detected.iter().map(|c| c.name.clone()).collect();

    // ── Parse inputs ──────────────────────────────────────────────────────────
    let tags: Vec<String> = tags_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let wallpaper = if wallpaper_input.trim().is_empty() {
        None
    } else {
        let setter = detect_wallpaper_setter();
        Some(WallpaperConfig {
            file: wallpaper_input.trim().to_string(),
            setter,
        })
    };

    let wm = detect_wm(&detected_names);
    let display_server = detect_display_server(&detected_names, &registry);

    // ── Build rice directory in store ──────────────────────────────────────────
    let rice_dir = paths::store_dir().join(&author).join(&name);
    let configs_dir = rice_dir.join("configs");
    let walls_dir = rice_dir.join("walls");
    paths::ensure_dir(&rice_dir)?;
    paths::ensure_dir(&configs_dir)?;
    paths::ensure_dir(&walls_dir)?;

    // ── Build manifest ────────────────────────────────────────────────────────
    let manifest = Manifest {
        meta: Meta {
            name: name.clone(),
            author: author.clone(),
            version: "0.2.0".to_string(),
            wm,
            display_server,
            colorscheme: Some(colorscheme),
            components: detected_names,
            tags,
            description: Some(description),
        },
        dependencies: Dependencies::default(),
        shell: None,
        wallpaper: wallpaper.clone(),
        overrides: HashMap::new(),
        hooks: Hooks::default(),
    };

    let toml_str = manifest.to_toml_string()?;
    std::fs::write(rice_dir.join("manifest.toml"), &toml_str)?;

    // ── Copy component config files into configs/<component>/ ─────────────────
    for comp in &detected {
        let dest_dir = configs_dir.join(&comp.name);
        paths::ensure_dir(&dest_dir)?;
        copy_dir_recursive(&comp.source_dir, &dest_dir)?;
    }

    // ── Copy wallpaper into walls/ ────────────────────────────────────────────
    if let Some(ref wall) = wallpaper {
        let src = PathBuf::from(&wall.file);
        if src.exists() {
            if let Some(fname) = src.file_name() {
                std::fs::copy(&src, walls_dir.join(fname))?;
            }
        }
    }

    println!();
    println!(
        "{}",
        format!(
            "Scaffolded {}/{} into {}",
            author,
            name,
            rice_dir.display()
        )
        .green()
        .bold()
    );

    Ok(())
}

// ── Directory-name-based component detection ──────────────────────────────────

/// Scan `config_dir` (typically ~/.config) ONE level deep.
/// Matches directory names against the known component list.
/// Special handling for `hypr/` → splits into hyprland/hyprlock by filename.
fn scan_components(config_dir: &Path) -> Vec<DetectedComponent> {
    let mut result = Vec::new();

    let entries = match std::fs::read_dir(config_dir) {
        Ok(e) => e,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let dir_name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Handle top-level files (e.g. ~/.config/starship.toml)
        if path.is_file() {
            if dir_name == "starship.toml" {
                result.push(DetectedComponent {
                    name: "starship".to_string(),
                    source_dir: config_dir.to_path_buf(),
                });
            }
            continue;
        }

        if !path.is_dir() {
            continue;
        }

        // Special case: hypr/ contains both hyprland.conf and hyprlock.conf
        if dir_name == "hypr" {
            if path.join("hyprland.conf").exists() {
                result.push(DetectedComponent {
                    name: "hyprland".to_string(),
                    source_dir: path.clone(),
                });
            }
            if path.join("hyprlock.conf").exists() {
                result.push(DetectedComponent {
                    name: "hyprlock".to_string(),
                    source_dir: path.clone(),
                });
            }
            continue;
        }

        // Standard match: directory name in known list
        if KNOWN_COMPONENTS.contains(&dir_name.as_str()) {
            result.push(DetectedComponent {
                name: dir_name,
                source_dir: path,
            });
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// Recursively copy all files and subdirectories from src to dest.
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            paths::ensure_dir(&dest_path)?;
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn detect_wallpaper_setter() -> String {
    let setters = ["swww", "hyprpaper", "swaybg", "feh", "nitrogen"];
    for setter in &setters {
        if which_exists(setter) {
            return setter.to_string();
        }
    }
    "feh".to_string()
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
