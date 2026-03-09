use std::path::PathBuf;

/// All error types for rixi operations.
#[derive(Debug, thiserror::Error)]
pub enum RixiError {
    #[error("Manifest not found at {0}")]
    ManifestNotFound(PathBuf),

    #[error("Failed to parse manifest: {0}")]
    ManifestParse(String),

    #[error("Unknown component: {0}")]
    UnknownComponent(String),

    #[error("Component files missing in rice directory: {component} — expected {path}")]
    ComponentFileMissing { component: String, path: PathBuf },

    #[error("No rice currently applied — nothing to rollback")]
    NothingToRollback,

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(PathBuf),

    #[error("State file error: {0}")]
    StateError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, RixiError>;
