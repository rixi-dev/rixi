use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde_json::Value;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::errors::{Result, RixiError};
use crate::paths;

const GITHUB_API_BASE: &str = "https://api.github.com";
const THEMES_REPO: &str = "rixi-dev/themes";

struct RegistryFile {
    relative_path: String,
    download_url: String,
}

pub fn run(rice: &str) -> Result<()> {
    let (author, theme) = parse_rice(rice)?;
    let target_dir = rices_dir().join(author).join(theme);

    if target_dir.exists() {
        print!("{}/{} already exists locally. Overwrite? [y/N] ", author, theme);
        io::stdout().flush().map_err(RixiError::Io)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(RixiError::Io)?;
        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            return Ok(());
        }

        std::fs::remove_dir_all(&target_dir)?;
    }

    println!("Fetching {}/{} from rixi-dev/themes...", author, theme);

    let client = github_client()?;
    let mut files = Vec::new();
    fetch_tree_recursive(&client, author, theme, "", &mut files)?;

    for file in &files {
        let destination = target_dir.join(&file.relative_path);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let response = client
            .get(&file.download_url)
            .send()
            .map_err(|e| RixiError::Other(format!("Failed to download {}: {}", file.relative_path, e)))?;

        if !response.status().is_success() {
            return Err(RixiError::Other(format!(
                "Failed to download {}",
                file.relative_path
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| RixiError::Other(format!("Failed to read downloaded data for {}: {}", file.relative_path, e)))?;

        std::fs::write(&destination, &bytes)?;
        println!("  ✓ {}", file.relative_path);
    }

    if !validate_downloaded_rice(&target_dir) {
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)?;
        }
        return Err(RixiError::Other(
            "Downloaded rice is missing required files. Removed.".to_string(),
        ));
    }

    println!();
    println!(
        "Pulled {}/{}. Run rixi apply {}/{} to apply it.",
        author, theme, author, theme
    );

    Ok(())
}

fn parse_rice(rice: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = rice.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(RixiError::Other(
            "Rice must be specified as author/theme".to_string(),
        ));
    }

    Ok((parts[0], parts[1]))
}

fn rices_dir() -> PathBuf {
    paths::data_dir().join("rices")
}

fn github_client() -> Result<Client> {
    Client::builder()
        .build()
        .map_err(|e| RixiError::Other(format!("Failed to initialize HTTP client: {}", e)))
}

fn fetch_tree_recursive(
    client: &Client,
    author: &str,
    theme: &str,
    sub_path: &str,
    files: &mut Vec<RegistryFile>,
) -> Result<()> {
    let encoded_path = if sub_path.is_empty() {
        format!("{}/{}", author, theme)
    } else {
        format!("{}/{}/{}", author, theme, sub_path)
    };

    let url = format!(
        "{}/repos/{}/contents/{}",
        GITHUB_API_BASE, THEMES_REPO, encoded_path
    );

    let response = client
        .get(&url)
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .send()
        .map_err(|e| RixiError::Other(format!("GitHub API request failed: {}", e)))?;

    if response.status().as_u16() == 404 && sub_path.is_empty() {
        return Err(RixiError::Other(format!(
            "{}/{} not found in rixi-dev/themes.",
            author, theme
        )));
    }

    if !response.status().is_success() {
        return Err(RixiError::Other(format!(
            "GitHub API error ({}): {}",
            response.status().as_u16(),
            url
        )));
    }

    let json: Value = response
        .json()
        .map_err(|e| RixiError::Other(format!("Invalid GitHub API response: {}", e)))?;

    let entries = json.as_array().ok_or_else(|| {
        RixiError::Other("GitHub API returned an unexpected response format".to_string())
    })?;

    for entry in entries {
        let entry_type = entry.get("type").and_then(Value::as_str).unwrap_or_default();
        let entry_path = entry.get("path").and_then(Value::as_str).unwrap_or_default();

        let prefix = format!("{}/{}/", author, theme);
        let rel = entry_path
            .strip_prefix(&prefix)
            .unwrap_or(entry_path)
            .to_string();

        if entry_type == "dir" {
            fetch_tree_recursive(client, author, theme, &rel, files)?;
            continue;
        }

        if entry_type == "file" {
            let download_url = entry
                .get("download_url")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    RixiError::Other(format!("Missing download URL for {}", rel))
                })?;

            files.push(RegistryFile {
                relative_path: rel,
                download_url: download_url.to_string(),
            });
        }
    }

    Ok(())
}

fn validate_downloaded_rice(root: &Path) -> bool {
    let manifest = root.join("manifest.toml");
    let configs = root.join("configs");
    let preview = root.join("preview.png");

    manifest.is_file() && configs.is_dir() && preview.is_file()
}
