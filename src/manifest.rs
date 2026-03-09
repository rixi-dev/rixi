use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::errors::{Result, RixiError};

/// The full manifest.toml structure for a rice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub meta: Meta,

    #[serde(default)]
    pub dependencies: Dependencies,

    #[serde(default)]
    pub overrides: HashMap<String, String>,

    #[serde(default)]
    pub hooks: Hooks,
}

/// Metadata about the rice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub author: String,

    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub wm: Option<String>,

    #[serde(default)]
    pub display_server: Vec<String>,

    #[serde(default)]
    pub colorscheme: Option<String>,

    pub components: Vec<String>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub description: Option<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Declared dependencies — printed but not auto-installed in v0.1.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dependencies {
    #[serde(default)]
    pub packages: Vec<String>,

    #[serde(default)]
    pub fonts: Vec<String>,

    #[serde(default)]
    pub icons: Vec<String>,
}

/// Hook commands — parsed but not executed in v0.1.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Hooks {
    #[serde(default)]
    pub post_apply: Vec<String>,
}

impl Manifest {
    /// Load and parse a manifest.toml from the given path.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(RixiError::ManifestNotFound(path.to_path_buf()));
        }

        let content = std::fs::read_to_string(path)?;
        let manifest: Manifest =
            toml::from_str(&content).map_err(|e| RixiError::ManifestParse(e.to_string()))?;

        Ok(manifest)
    }

    /// Serialize the manifest to a TOML string.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| RixiError::ManifestParse(e.to_string()))
    }

    /// Returns the namespace string: author/theme
    pub fn namespace(&self) -> String {
        format!("{}/{}", self.meta.author, self.meta.name)
    }
}
