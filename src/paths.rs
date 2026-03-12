use std::path::PathBuf;

/// Returns the rixi data directory: ~/.local/share/rixi/
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .expect("Could not determine XDG data directory")
        .join("rixi")
}

/// Returns the store directory: ~/.local/share/rixi/store/
pub fn store_dir() -> PathBuf {
    data_dir().join("store")
}

/// Returns the snapshots directory: ~/.local/share/rixi/snapshots/
pub fn snapshots_dir() -> PathBuf {
    data_dir().join("snapshots")
}

/// Returns the state file path: ~/.local/share/rixi/state.toml
pub fn state_file() -> PathBuf {
    data_dir().join("state.toml")
}

/// Expands a leading `~` in a path string to the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(rest)
    } else if path == "~" {
        dirs::home_dir().expect("Could not determine home directory")
    } else {
        PathBuf::from(path)
    }
}

/// Ensure a directory exists, creating it (and parents) if needed.
pub fn ensure_dir(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
