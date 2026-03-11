use std::fs;

/// Detected Linux distribution info.
#[derive(Debug, Clone)]
pub struct Distro {
    pub name: String,
    pub package_manager: PackageManager,
}

/// Supported package managers.
#[derive(Debug, Clone)]
pub enum PackageManager {
    Pacman,
    Dnf,
    Apt,
    Zypper,
    Xbps,
    NixEnv,
    Unknown,
}

impl PackageManager {
    /// Returns the install command prefix for this package manager.
    pub fn install_cmd(&self) -> Option<&'static str> {
        match self {
            PackageManager::Pacman => Some("sudo pacman -S"),
            PackageManager::Dnf => Some("sudo dnf install"),
            PackageManager::Apt => Some("sudo apt install"),
            PackageManager::Zypper => Some("sudo zypper install"),
            PackageManager::Xbps => Some("sudo xbps-install -S"),
            PackageManager::NixEnv => Some("nix-env -iA nixpkgs."),
            PackageManager::Unknown => None,
        }
    }
}

/// Detect the current Linux distribution from /etc/os-release.
pub fn detect() -> Distro {
    let content = fs::read_to_string("/etc/os-release").unwrap_or_default();

    let id = parse_field(&content, "ID");
    let id_like = parse_field(&content, "ID_LIKE");
    let name = parse_field(&content, "PRETTY_NAME")
        .or_else(|| parse_field(&content, "NAME"))
        .unwrap_or_else(|| "Unknown".to_string());

    let pm = match id.as_deref() {
        Some("arch" | "manjaro" | "endeavouros" | "cachyos" | "garuda") => PackageManager::Pacman,
        Some("fedora" | "rhel" | "centos" | "rocky" | "alma") => PackageManager::Dnf,
        Some("debian" | "ubuntu" | "linuxmint" | "pop" | "zorin" | "elementary") => {
            PackageManager::Apt
        }
        Some("opensuse" | "opensuse-leap" | "opensuse-tumbleweed") => PackageManager::Zypper,
        Some("void") => PackageManager::Xbps,
        Some("nixos") => PackageManager::NixEnv,
        _ => {
            // Fall back to ID_LIKE
            match id_like.as_deref() {
                Some(like) if like.contains("arch") => PackageManager::Pacman,
                Some(like) if like.contains("debian") || like.contains("ubuntu") => {
                    PackageManager::Apt
                }
                Some(like) if like.contains("fedora") || like.contains("rhel") => {
                    PackageManager::Dnf
                }
                Some(like) if like.contains("suse") => PackageManager::Zypper,
                _ => PackageManager::Unknown,
            }
        }
    };

    Distro {
        name,
        package_manager: pm,
    }
}

/// Parse a key=value field from os-release content.
fn parse_field(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(value) = line.strip_prefix(&format!("{key}=")) {
            return Some(value.trim_matches('"').to_string());
        }
    }
    None
}
