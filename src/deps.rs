use colored::Colorize;
use std::io::{self, Write};
use std::process::Command;

use crate::distro::{self, PackageManager};
use crate::manifest::Dependencies;

/// Check dependencies and prompt the user to continue if any are missing.
/// Returns true if apply should proceed, false if user cancelled.
pub fn check_and_prompt(deps: &Dependencies) -> bool {
    let has_any = !deps.packages.is_empty() || !deps.fonts.is_empty() || !deps.icons.is_empty();
    if !has_any {
        return true;
    }

    println!("{}", "Checking dependencies...".bold());

    let mut missing_packages = Vec::new();
    let mut has_missing = false;

    // Check packages via `which`
    for pkg in &deps.packages {
        if is_package_installed(pkg) {
            println!("  {} {:<20} {}", "✓".green().bold(), pkg, "installed".dimmed());
        } else {
            println!("  {} {:<20} {}", "✗".red().bold(), pkg, "missing".red());
            missing_packages.push(pkg.as_str());
            has_missing = true;
        }
    }

    // Check fonts via fc-list
    if !deps.fonts.is_empty() {
        println!();
        for font in &deps.fonts {
            if is_font_installed(font) {
                println!("  {} {:<20} {}", "✓".green().bold(), font, "installed".dimmed());
            } else {
                println!(
                    "  {} {:<20} {}",
                    "✗".red().bold(),
                    font,
                    "not found (fc-list check failed)".red()
                );
                has_missing = true;
            }
        }
    }

    // Check icons (just report, no automated check)
    if !deps.icons.is_empty() {
        for icon in &deps.icons {
            println!(
                "  {} {:<20} {}",
                "✗".yellow().bold(),
                icon,
                "not verified".dimmed()
            );
            has_missing = true;
        }
    }

    if !has_missing {
        println!();
        return true;
    }

    // Print distro-aware install commands
    println!();
    let distro = distro::detect();
    println!("Detected distro: {}", distro.name.cyan().bold());
    println!();

    if !missing_packages.is_empty() {
        if let Some(cmd) = distro.package_manager.install_cmd() {
            match distro.package_manager {
                PackageManager::NixEnv => {
                    println!("{}", "Run this to install missing packages:".bold());
                    for pkg in &missing_packages {
                        println!("  {}{}", cmd, pkg);
                    }
                }
                _ => {
                    println!("{}", "Run this to install missing packages:".bold());
                    println!("  {} {}", cmd, missing_packages.join(" "));
                }
            }
        } else {
            println!("{}", "Missing packages (install manually):".bold());
            println!("  {}", missing_packages.join(" "));
        }
    }

    if !deps.fonts.is_empty() {
        let missing_fonts: Vec<&str> = deps
            .fonts
            .iter()
            .filter(|f| !is_font_installed(f))
            .map(|f| f.as_str())
            .collect();
        if !missing_fonts.is_empty() {
            println!();
            println!("{}", "Manual installs required:".bold());
            for font in &missing_fonts {
                println!("  Fonts    → https://nerdfonts.com (install {})", font);
            }
        }
    }

    if !deps.icons.is_empty() {
        println!("  Icons    → install manually: {}", deps.icons.join(", "));
    }

    println!();
    print!(
        "{}",
        "Continue applying without missing dependencies? [y/N] ".yellow()
    );
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Check if a package is installed by checking PATH via `which`.
fn is_package_installed(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check if a font is installed via `fc-list`.
fn is_font_installed(font_name: &str) -> bool {
    Command::new("fc-list")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lower = font_name.to_lowercase();
            stdout.to_lowercase().contains(&lower)
        })
        .unwrap_or(false)
}
