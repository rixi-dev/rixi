use serde::{Deserialize, Serialize};

use crate::errors::{Result, RixiError};
use crate::paths;

/// Tracks what rice is currently applied.
/// Stored at ~/.local/share/rixi/state.toml
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub current: Option<CurrentRice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentRice {
    pub author: String,
    pub theme: String,
    pub applied_at: String,
    pub snapshot: String,
}

impl State {
    /// Load the state file, returning a default (empty) state if it doesn't exist.
    pub fn load() -> Result<Self> {
        let path = paths::state_file();
        if !path.exists() {
            return Ok(State::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let state: State =
            toml::from_str(&content).map_err(|e| RixiError::StateError(e.to_string()))?;
        Ok(state)
    }

    /// Save the state to disk.
    pub fn save(&self) -> Result<()> {
        let path = paths::state_file();
        paths::ensure_dir(&path.parent().unwrap().to_path_buf())?;

        let content =
            toml::to_string_pretty(self).map_err(|e| RixiError::StateError(e.to_string()))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Clear the current applied rice (used after rollback).
    pub fn clear_current(&mut self) {
        self.current = None;
    }

    /// Set the currently applied rice.
    pub fn set_current(&mut self, author: String, theme: String, snapshot: String) {
        self.current = Some(CurrentRice {
            author,
            theme,
            applied_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            snapshot,
        });
    }
}
