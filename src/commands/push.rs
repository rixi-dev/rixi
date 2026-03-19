use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use crate::errors::{Result, RixiError};
use crate::paths;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_CLIENT_ID: &str = "Iv23livPJ5LxZBmldnBb";

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppConfig {
    #[serde(default)]
    github: GithubConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct GithubConfig {
    token: Option<String>,
    username: Option<String>,
}

pub fn run(rice: &str) -> Result<()> {
    let (author, theme) = parse_rice(rice)?;
    let rice_root = rices_dir().join(author).join(theme);

    validate_local_rice(&rice_root)?;

    let client = github_client()?;

    let (token, first_time_auth) = match load_saved_token()? {
        Some(token) => (token, false),
        None => {
            let token = run_device_flow(&client)?;
            (token, true)
        }
    };

    let username = fetch_username(&client, &token)?;
    save_github_config(&token, &username)?;
    println!("  → Logged in as {}", username);
    println!();

    fork_themes_repo(&client, &token)?;

    let branch_name = format!("rixi-push-{}-{}", author, theme);
    let branch_tip_sha = create_branch(&client, &token, &username, &branch_name)?;

    upload_rice_files(
        &client,
        &token,
        &username,
        &branch_tip_sha,
        &branch_name,
        author,
        theme,
        &rice_root,
    )?;

    let pr_url = format!(
        "https://github.com/rixi-dev/themes/compare/main...{}:themes:{}",
        username, branch_name
    );

    println!();
    println!("Opening PR in your browser...");
    println!("  → {}", pr_url);

    let _ = std::process::Command::new("xdg-open").arg(&pr_url).status();

    println!();
    println!("Complete the pull request in your browser.");
    if first_time_auth {
        println!(
            "You are now logged in as {}. You will not be asked to authorize again.",
            username
        );
    }

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

fn config_file_path() -> PathBuf {
    paths::data_dir().join("config.toml")
}

fn github_client() -> Result<Client> {
    Client::builder()
        .build()
        .map_err(|e| RixiError::Other(format!("Failed to initialize HTTP client: {}", e)))
}

fn validate_local_rice(rice_root: &Path) -> Result<()> {
    println!();

    if !rice_root.exists() {
        return Err(RixiError::Other(format!(
            "Local rice not found at {}",
            rice_root.display()
        )));
    }

    print!("Validating rice structure... ");
    let configs_dir = rice_root.join("configs");
    if !configs_dir.is_dir() {
        return Err(RixiError::Other(
            "configs/ directory is missing".to_string(),
        ));
    }

    let mut has_component_subdir = false;
    for entry in std::fs::read_dir(&configs_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            has_component_subdir = true;
            break;
        }
    }

    if !has_component_subdir {
        return Err(RixiError::Other(
            "configs/ must contain at least one component directory".to_string(),
        ));
    }
    println!("done");

    print!("Checking manifest... ");
    if !rice_root.join("manifest.toml").is_file() {
        return Err(RixiError::Other(
            "manifest.toml is missing. Add manifest before pushing.".to_string(),
        ));
    }
    println!("done");

    print!("preview.png present... ");
    if !rice_root.join("preview.png").is_file() {
        return Err(RixiError::Other(
            "preview.png is missing. Add a screenshot before pushing.".to_string(),
        ));
    }
    println!("done");
    println!();

    Ok(())
}

fn load_saved_token() -> Result<Option<String>> {
    let path = config_file_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    let config: AppConfig = toml::from_str(&content)
        .map_err(|e| RixiError::Other(format!("Failed to parse config.toml: {}", e)))?;

    Ok(config.github.token)
}

fn save_github_config(token: &str, username: &str) -> Result<()> {
    let path = config_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let config = AppConfig {
        github: GithubConfig {
            token: Some(token.to_string()),
            username: Some(username.to_string()),
        },
    };

    let content = toml::to_string_pretty(&config)
        .map_err(|e| RixiError::Other(format!("Failed to serialize config.toml: {}", e)))?;

    std::fs::write(path, content)?;
    Ok(())
}

fn run_device_flow(client: &Client) -> Result<String> {
    let device_code_response = client
        .post("https://github.com/login/device/code")
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/json")
        .json(&json!({
            "client_id": GITHUB_CLIENT_ID,
            "scope": "public_repo"
        }))
        .send()
        .map_err(|e| RixiError::Other(format!("Failed to request device code: {}", e)))?;

    if !device_code_response.status().is_success() {
        return Err(RixiError::Other(format!(
            "Failed to request device code (status {})",
            device_code_response.status().as_u16()
        )));
    }

    let payload: Value = device_code_response
        .json()
        .map_err(|e| RixiError::Other(format!("Invalid device code response: {}", e)))?;

    let device_code = payload
        .get("device_code")
        .and_then(Value::as_str)
        .ok_or_else(|| RixiError::Other("Missing device_code in response".to_string()))?;
    let user_code = payload
        .get("user_code")
        .and_then(Value::as_str)
        .ok_or_else(|| RixiError::Other("Missing user_code in response".to_string()))?;
    let interval = payload
        .get("interval")
        .and_then(Value::as_u64)
        .unwrap_or(5);
    let expires_in = payload
        .get("expires_in")
        .and_then(Value::as_u64)
        .unwrap_or(900);

    println!("To authorize rixi, open this URL in your browser:");
    println!("  https://github.com/login/device");
    println!();
    println!("Enter this code when prompted:");
    println!("  {}", user_code);
    println!();
    println!("Waiting for authorization...");

    let _ = std::process::Command::new("xdg-open")
        .arg("https://github.com/login/device")
        .status();

    poll_for_access_token(client, device_code, interval, expires_in)
}

fn poll_for_access_token(
    client: &Client,
    device_code: &str,
    initial_interval_seconds: u64,
    expires_in_seconds: u64,
) -> Result<String> {
    let started = Instant::now();
    let mut interval = initial_interval_seconds;

    loop {
        if started.elapsed() >= Duration::from_secs(expires_in_seconds) {
            return Err(RixiError::Other(
                "Device code expired before authorization completed".to_string(),
            ));
        }

        thread::sleep(Duration::from_secs(interval));

        let response = client
            .post("https://github.com/login/oauth/access_token")
            .header(USER_AGENT, "rixi")
            .header(ACCEPT, "application/json")
            .json(&json!({
                "client_id": GITHUB_CLIENT_ID,
                "device_code": device_code,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code"
            }))
            .send()
            .map_err(|e| RixiError::Other(format!("Token polling failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(RixiError::Other(format!(
                "Token polling failed (status {})",
                response.status().as_u16()
            )));
        }

        let payload: Value = response
            .json()
            .map_err(|e| RixiError::Other(format!("Invalid token polling response: {}", e)))?;

        if let Some(token) = payload.get("access_token").and_then(Value::as_str) {
            println!("  ✓ Authorized");
            return Ok(token.to_string());
        }

        let error = payload
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or_default();

        match error {
            "authorization_pending" => {}
            "slow_down" => {
                interval += 5;
            }
            "expired_token" => {
                return Err(RixiError::Other(
                    "Authorization code expired. Run rixi push again.".to_string(),
                ));
            }
            "access_denied" => {
                return Err(RixiError::Other(
                    "Authorization was denied. Run rixi push again.".to_string(),
                ));
            }
            _ => {
                return Err(RixiError::Other(
                    "Unexpected OAuth response during authorization".to_string(),
                ));
            }
        }
    }
}

fn fetch_username(client: &Client, token: &str) -> Result<String> {
    let response = client
        .get(format!("{}/user", GITHUB_API_BASE))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .map_err(|e| RixiError::Other(format!("GitHub authentication failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(RixiError::Other(
            "GitHub authentication failed. Try running rixi push again.".to_string(),
        ));
    }

    let payload: Value = response
        .json()
        .map_err(|e| RixiError::Other(format!("Invalid /user response: {}", e)))?;

    let username = payload
        .get("login")
        .and_then(Value::as_str)
        .ok_or_else(|| RixiError::Other("GitHub username missing from response".to_string()))?;

    Ok(username.to_string())
}

fn fork_themes_repo(client: &Client, token: &str) -> Result<()> {
    let response = client
        .post(format!("{}/repos/rixi-dev/themes/forks", GITHUB_API_BASE))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({}))
        .send()
        .map_err(|e| RixiError::Other(format!("Failed to fork repository: {}", e)))?;

    if !response.status().is_success() {
        return Err(RixiError::Other(format!(
            "Failed to fork rixi-dev/themes (status {})",
            response.status().as_u16()
        )));
    }

    println!("Forking rixi-dev/themes... done");
    thread::sleep(Duration::from_secs(3));

    Ok(())
}

fn create_branch(client: &Client, token: &str, username: &str, branch_name: &str) -> Result<String> {
    let ref_response = client
        .get(format!(
            "{}/repos/{}/themes/git/ref/heads/main",
            GITHUB_API_BASE, username
        ))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .map_err(|e| RixiError::Other(format!("Failed to read fork main branch: {}", e)))?;

    if !ref_response.status().is_success() {
        return Err(RixiError::Other(format!(
            "Failed to read fork main branch (status {})",
            ref_response.status().as_u16()
        )));
    }

    let ref_payload: Value = ref_response
        .json()
        .map_err(|e| RixiError::Other(format!("Invalid ref response: {}", e)))?;

    let sha = ref_payload
        .get("object")
        .and_then(|o| o.get("sha"))
        .and_then(Value::as_str)
        .ok_or_else(|| RixiError::Other("Main branch SHA missing in response".to_string()))?;

    let create_response = client
        .post(format!(
            "{}/repos/{}/themes/git/refs",
            GITHUB_API_BASE, username
        ))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "ref": format!("refs/heads/{}", branch_name),
            "sha": sha
        }))
        .send()
        .map_err(|e| RixiError::Other(format!("Failed to create branch: {}", e)))?;

    if !create_response.status().is_success() {
        if create_response.status() != StatusCode::UNPROCESSABLE_ENTITY {
            let status = create_response.status().as_u16();
            let body = create_response.text().unwrap_or_default();
            return Err(RixiError::Other(format!(
                "Failed to create branch {} (status {}). {}",
                branch_name,
                status,
                body
            )));
        }
    }

    let branch_ref_response = client
        .get(format!(
            "{}/repos/{}/themes/git/ref/heads/{}",
            GITHUB_API_BASE, username, branch_name
        ))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .map_err(|e| RixiError::Other(format!("Failed to read branch tip: {}", e)))?;

    if !branch_ref_response.status().is_success() {
        return Err(RixiError::Other(format!(
            "Failed to read branch tip for {} (status {})",
            branch_name,
            branch_ref_response.status().as_u16()
        )));
    }

    let branch_ref_payload: Value = branch_ref_response
        .json()
        .map_err(|e| RixiError::Other(format!("Invalid branch ref response: {}", e)))?;

    let branch_tip_sha = branch_ref_payload
        .get("object")
        .and_then(|o| o.get("sha"))
        .and_then(Value::as_str)
        .ok_or_else(|| RixiError::Other("Branch tip SHA missing in response".to_string()))?
        .to_string();

    println!("Creating branch {}... done", branch_name);
    println!();

    Ok(branch_tip_sha)
}

fn upload_rice_files(
    client: &Client,
    token: &str,
    username: &str,
    branch_tip_sha: &str,
    branch_name: &str,
    author: &str,
    theme: &str,
    rice_root: &Path,
) -> Result<()> {
    let mut files = Vec::new();
    collect_files_recursive(rice_root, rice_root, &mut files)?;
    files.sort();

    println!("Preparing files...");

    let mut blob_entries = Vec::new();

    for relative in &files {
        let path = rice_root.join(relative);
        let bytes = std::fs::read(&path)?;
        let encoded = STANDARD.encode(bytes);
        let relative_display = relative.replace('\\', "/");

        let blob_response = client
            .post(format!(
                "{}/repos/{}/themes/git/blobs",
                GITHUB_API_BASE, username
            ))
            .header(USER_AGENT, "rixi")
            .header(ACCEPT, "application/vnd.github+json")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .json(&json!({
                "content": encoded,
                "encoding": "base64"
            }))
            .send()
            .map_err(|e| RixiError::Other(format!("Step 6A failed for {}: {}", relative_display, e)))?;

        if !blob_response.status().is_success() {
            let status = blob_response.status().as_u16();
            let body = blob_response.text().unwrap_or_default();
            return Err(RixiError::Other(format!(
                "Step 6A failed for {} (status {}). {}",
                relative_display,
                status,
                body
            )));
        }

        let blob_payload: Value = blob_response
            .json()
            .map_err(|e| RixiError::Other(format!("Step 6A failed for {}: {}", relative_display, e)))?;

        let blob_sha = blob_payload
            .get("sha")
            .and_then(Value::as_str)
            .ok_or_else(|| RixiError::Other(format!("Step 6A failed for {}: missing blob sha", relative_display)))?
            .to_string();

        blob_entries.push((relative_display.clone(), blob_sha));
        println!("  ✓ {}", relative_display);
    }

    let tree_entries: Vec<Value> = blob_entries
        .iter()
        .map(|(relative_path, blob_sha)| {
            json!({
                "path": format!("{}/{}/{}", author, theme, relative_path),
                "mode": "100644",
                "type": "blob",
                "sha": blob_sha
            })
        })
        .collect();

    let tree_response = client
        .post(format!(
            "{}/repos/{}/themes/git/trees",
            GITHUB_API_BASE, username
        ))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "base_tree": branch_tip_sha,
            "tree": tree_entries
        }))
        .send()
        .map_err(|e| RixiError::Other(format!("Step 6B failed: {}", e)))?;

    if !tree_response.status().is_success() {
        let status = tree_response.status().as_u16();
        let body = tree_response.text().unwrap_or_default();
        return Err(RixiError::Other(format!(
            "Step 6B failed (status {}). {}",
            status,
            body
        )));
    }

    let tree_payload: Value = tree_response
        .json()
        .map_err(|e| RixiError::Other(format!("Step 6B failed: {}", e)))?;

    let tree_sha = tree_payload
        .get("sha")
        .and_then(Value::as_str)
        .ok_or_else(|| RixiError::Other("Step 6B failed: missing tree sha".to_string()))?;

    let commit_response = client
        .post(format!(
            "{}/repos/{}/themes/git/commits",
            GITHUB_API_BASE, username
        ))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "message": format!("add {}/{}", author, theme),
            "tree": tree_sha,
            "parents": [branch_tip_sha]
        }))
        .send()
        .map_err(|e| RixiError::Other(format!("Step 6C failed: {}", e)))?;

    if !commit_response.status().is_success() {
        let status = commit_response.status().as_u16();
        let body = commit_response.text().unwrap_or_default();
        return Err(RixiError::Other(format!(
            "Step 6C failed (status {}). {}",
            status,
            body
        )));
    }

    let commit_payload: Value = commit_response
        .json()
        .map_err(|e| RixiError::Other(format!("Step 6C failed: {}", e)))?;

    let commit_sha = commit_payload
        .get("sha")
        .and_then(Value::as_str)
        .ok_or_else(|| RixiError::Other("Step 6C failed: missing commit sha".to_string()))?;

    let update_ref_response = client
        .patch(format!(
            "{}/repos/{}/themes/git/refs/heads/{}",
            GITHUB_API_BASE, username, branch_name
        ))
        .header(USER_AGENT, "rixi")
        .header(ACCEPT, "application/vnd.github+json")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&json!({
            "sha": commit_sha,
            "force": false
        }))
        .send()
        .map_err(|e| RixiError::Other(format!("Step 6D failed: {}", e)))?;

    if !update_ref_response.status().is_success() {
        let status = update_ref_response.status().as_u16();
        let body = update_ref_response.text().unwrap_or_default();
        return Err(RixiError::Other(format!(
            "Step 6D failed (status {}). {}",
            status,
            body
        )));
    }

    println!("Uploading rice... done");
    println!("  → 1 commit, {} files", files.len());

    Ok(())
}

fn collect_files_recursive(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect_files_recursive(root, &path, out)?;
            continue;
        }

        if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| RixiError::Other(format!("Failed to build relative path: {}", e)))?;
            out.push(relative.to_string_lossy().to_string());
        }
    }

    Ok(())
}
