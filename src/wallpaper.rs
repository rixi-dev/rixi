use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::errors::Result;
use crate::manifest::WallpaperConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WallpaperState {
    file_name: String,
    setter: String,
}

fn wallpaper_dir() -> std::path::PathBuf {
    crate::paths::data_dir().join("wallpaper")
}

fn wallpaper_state_file() -> std::path::PathBuf {
    wallpaper_dir().join("state.toml")
}

/// Apply wallpaper using the configured setter.
/// `rice_path` is the rice source directory (wallpaper file paths are relative to it).
pub fn apply(config: &WallpaperConfig, rice_path: &Path) -> Result<()> {
    let wallpaper_src = rice_path.join(&config.file);

    if !wallpaper_src.exists() {
        println!(
            "  {} Wallpaper file not found: {}",
            "✗".yellow(),
            wallpaper_src.display()
        );
        return Ok(());
    }

    // Copy wallpaper to rixi data dir for persistence
    let dest = wallpaper_dir();
    crate::paths::ensure_dir(&dest)?;
    let filename = wallpaper_src
        .file_name()
        .expect("wallpaper should have a filename");
    let dest_file = dest.join(filename);
    std::fs::copy(&wallpaper_src, &dest_file)?;

    let wall_path = dest_file.to_string_lossy().to_string();

    let result = run_configured_setter(&config.setter, &wall_path);

    match result {
        Ok(true) => {
            save_wallpaper_state(filename.to_string_lossy().as_ref(), &config.setter)?;
            println!(
                "  {} Wallpaper set via {}",
                "✓".green().bold(),
                config.setter
            );
        }
        Ok(false) => {
            println!(
                "  {} Wallpaper setter {} failed (is it installed?)",
                "✗".yellow(),
                config.setter
            );
        }
        Err(_) => {
            println!(
                "  {} Could not run wallpaper setter: {}",
                "✗".yellow(),
                config.setter
            );
        }
    }

    Ok(())
}

/// Snapshot the current managed wallpaper state and file, if available.
pub fn snapshot_current(snapshot_dir: &Path) -> Result<()> {
    let state_path = wallpaper_state_file();
    if !state_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&state_path)?;
    let state: WallpaperState = match toml::from_str(&content) {
        Ok(state) => state,
        Err(_) => return Ok(()),
    };

    let wallpaper_file = wallpaper_dir().join(&state.file_name);
    if !wallpaper_file.exists() {
        return Ok(());
    }

    let snapshot_wallpaper_dir = snapshot_dir.join("wallpaper");
    crate::paths::ensure_dir(&snapshot_wallpaper_dir)?;
    std::fs::copy(&wallpaper_file, snapshot_wallpaper_dir.join(&state.file_name))?;
    std::fs::copy(&state_path, snapshot_wallpaper_dir.join("state.toml"))?;

    Ok(())
}

/// Restore wallpaper from a snapshot and re-apply it.
/// Returns true if wallpaper was restored and re-applied.
pub fn restore_from_snapshot(snapshot_dir: &Path) -> Result<bool> {
    let snapshot_wallpaper_dir = snapshot_dir.join("wallpaper");
    let snapshot_state = snapshot_wallpaper_dir.join("state.toml");
    if !snapshot_state.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&snapshot_state)?;
    let state: WallpaperState = match toml::from_str(&content) {
        Ok(state) => state,
        Err(_) => return Ok(false),
    };

    let snap_wallpaper_file = snapshot_wallpaper_dir.join(&state.file_name);
    if !snap_wallpaper_file.exists() {
        return Ok(false);
    }

    let live_wallpaper_dir = wallpaper_dir();
    crate::paths::ensure_dir(&live_wallpaper_dir)?;
    let live_wallpaper_file = live_wallpaper_dir.join(&state.file_name);
    std::fs::copy(&snap_wallpaper_file, &live_wallpaper_file)?;

    let live_state = toml::to_string_pretty(&state)
        .map_err(|e| crate::errors::RixiError::Other(format!("Failed to serialize wallpaper state: {}", e)))?;
    std::fs::write(wallpaper_state_file(), live_state)?;

    let wall_path = live_wallpaper_file.to_string_lossy().to_string();
    let _ = run_configured_setter(&state.setter, &wall_path);

    Ok(true)
}

fn save_wallpaper_state(file_name: &str, setter: &str) -> Result<()> {
    let state = WallpaperState {
        file_name: file_name.to_string(),
        setter: setter.to_string(),
    };
    let state_toml = toml::to_string_pretty(&state)
        .map_err(|e| crate::errors::RixiError::Other(format!("Failed to serialize wallpaper state: {}", e)))?;
    std::fs::write(wallpaper_state_file(), state_toml)?;
    Ok(())
}

fn run_configured_setter(setter: &str, wall_path: &str) -> std::io::Result<bool> {
    match setter {
        "feh" => run_setter("feh", &["--bg-scale", wall_path]),
        "nitrogen" => run_setter("nitrogen", &["--set-zoom-fill", "--save", wall_path]),
        "hyprpaper" => {
            // hyprpaper uses its config file, so we just signal a reload
            run_setter("hyprctl", &["hyprpaper", "reload"])
        }
        "swww" => run_setter("swww", &["img", wall_path]),
        "swaybg" => {
            // Kill existing swaybg, start new one
            let _ = Command::new("pkill")
                .arg("swaybg")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            run_setter("swaybg", &["-i", wall_path])
        }
        _ => Ok(false),
    }
}

/// Run a wallpaper setter command. Returns Ok(true) on success.
fn run_setter(cmd: &str, args: &[&str]) -> std::io::Result<bool> {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
}
