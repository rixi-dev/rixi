use colored::Colorize;
use std::path::Path;
use std::process::Command;

use crate::errors::Result;
use crate::manifest::WallpaperConfig;

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
    let dest = crate::paths::data_dir().join("wallpaper");
    crate::paths::ensure_dir(&dest)?;
    let filename = wallpaper_src
        .file_name()
        .expect("wallpaper should have a filename");
    let dest_file = dest.join(filename);
    std::fs::copy(&wallpaper_src, &dest_file)?;

    let wall_path = dest_file.to_string_lossy().to_string();

    let result = match config.setter.as_str() {
        "feh" => run_setter("feh", &["--bg-scale", &wall_path]),
        "nitrogen" => run_setter("nitrogen", &["--set-zoom-fill", "--save", &wall_path]),
        "hyprpaper" => {
            // hyprpaper uses its config file, so we just signal a reload
            run_setter("hyprctl", &["hyprpaper", "reload"])
        }
        "swww" => run_setter("swww", &["img", &wall_path]),
        "swaybg" => {
            // Kill existing swaybg, start new one
            let _ = Command::new("pkill")
                .arg("swaybg")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            run_setter("swaybg", &["-i", &wall_path])
        }
        other => {
            println!(
                "  {} Unknown wallpaper setter: {}",
                "✗".yellow(),
                other
            );
            return Ok(());
        }
    };

    match result {
        Ok(true) => {
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

/// Run a wallpaper setter command. Returns Ok(true) on success.
fn run_setter(cmd: &str, args: &[&str]) -> std::io::Result<bool> {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
}
