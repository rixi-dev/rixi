use colored::Colorize;
use std::path::PathBuf;

use crate::errors::Result;
use crate::manifest::ShellConfig;
use crate::paths;

/// The managed shell script for zsh/bash: ~/.local/share/rixi/shell.sh
pub fn shell_script_path() -> PathBuf {
    paths::data_dir().join("shell.sh")
}

/// The managed fish config: ~/.config/fish/conf.d/rixi.fish
pub fn fish_config_path() -> PathBuf {
    paths::expand_tilde("~/.config/fish/conf.d/rixi.fish")
}

/// Apply shell configuration for a rice.
pub fn apply(config: &ShellConfig, namespace: &str) -> Result<()> {
    match config.shell_type.as_str() {
        "zsh" => apply_posix(config, namespace, "zsh"),
        "bash" => apply_posix(config, namespace, "bash"),
        "fish" => apply_fish(config, namespace),
        other => {
            println!(
                "  {} Unknown shell type: {}",
                "✗".yellow(),
                other
            );
            Ok(())
        }
    }
}

/// Handle zsh/bash: inject source line into rc file, write shell.sh
fn apply_posix(config: &ShellConfig, namespace: &str, shell: &str) -> Result<()> {
    let rc_path = match shell {
        "zsh" => paths::expand_tilde("~/.zshrc"),
        "bash" => paths::expand_tilde("~/.bashrc"),
        _ => unreachable!(),
    };

    let source_line = "source ~/.local/share/rixi/shell.sh";
    let marker = "# rixi shell theme — do not remove";

    // Inject source line if not already present
    if rc_path.exists() {
        let content = std::fs::read_to_string(&rc_path)?;
        if !content.contains(source_line) {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&rc_path)?;
            use std::io::Write;
            writeln!(file)?;
            writeln!(file, "{marker}")?;
            writeln!(file, "{source_line}")?;
            println!(
                "  {} Injected source line into {}",
                "✓".green().bold(),
                rc_path.display()
            );
        }
    }

    // Write the managed shell.sh
    let script = generate_posix_script(config, namespace, shell);
    let script_path = shell_script_path();
    paths::ensure_dir(&script_path.parent().unwrap().to_path_buf())?;
    std::fs::write(&script_path, script)?;
    println!(
        "  {} Wrote {}",
        "✓".green().bold(),
        script_path.display()
    );

    Ok(())
}

/// Handle fish: write rixi.fish to conf.d
fn apply_fish(config: &ShellConfig, namespace: &str) -> Result<()> {
    let script = generate_fish_script(config, namespace);
    let fish_path = fish_config_path();
    paths::ensure_dir(&fish_path.parent().unwrap().to_path_buf())?;
    std::fs::write(&fish_path, script)?;
    println!(
        "  {} Wrote {}",
        "✓".green().bold(),
        fish_path.display()
    );

    Ok(())
}

/// Generate the content of ~/.local/share/rixi/shell.sh
fn generate_posix_script(config: &ShellConfig, namespace: &str, shell: &str) -> String {
    let mut lines = vec![
        format!("# managed by rixi — current rice: {namespace}"),
        "# do not edit manually".to_string(),
    ];

    match config.prompt.as_str() {
        "starship" => {
            lines.push(format!("eval \"$(starship init {shell})\""));
        }
        "p10k" => {
            lines.push("# powerlevel10k — ensure p10k is in your plugin manager".to_string());
        }
        "oh-my-zsh" => {
            lines.push("# oh-my-zsh — ensure it is sourced in your .zshrc".to_string());
        }
        "none" | _ => {}
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Generate the content of ~/.config/fish/conf.d/rixi.fish
fn generate_fish_script(config: &ShellConfig, namespace: &str) -> String {
    let mut lines = vec![
        format!("# managed by rixi — current rice: {namespace}"),
        "# do not edit manually".to_string(),
    ];

    match config.prompt.as_str() {
        "starship" => {
            lines.push("starship init fish | source".to_string());
        }
        "none" | _ => {}
    }

    lines.push(String::new());
    lines.join("\n")
}

/// Snapshot shell config files before apply.
/// Returns paths that were snapshotted (for restore later).
pub fn snapshot_shell_files(snapshot_dir: &std::path::Path) -> Result<Vec<PathBuf>> {
    let mut snapshotted = Vec::new();

    let shell_sh = shell_script_path();
    if shell_sh.exists() {
        let dest = snapshot_dir.join("shell.sh");
        std::fs::copy(&shell_sh, &dest)?;
        snapshotted.push(shell_sh);
    }

    let fish_conf = fish_config_path();
    if fish_conf.exists() {
        let dest_dir = snapshot_dir.join("fish_conf_d");
        paths::ensure_dir(&dest_dir)?;
        let dest = dest_dir.join("rixi.fish");
        std::fs::copy(&fish_conf, &dest)?;
        snapshotted.push(fish_conf);
    }

    Ok(snapshotted)
}

/// Restore shell config files from a snapshot.
pub fn restore_shell_files(snapshot_dir: &std::path::Path) -> Result<()> {
    let shell_sh_snapshot = snapshot_dir.join("shell.sh");
    if shell_sh_snapshot.exists() {
        let dest = shell_script_path();
        paths::ensure_dir(&dest.parent().unwrap().to_path_buf())?;
        std::fs::copy(&shell_sh_snapshot, &dest)?;
    }

    let fish_snapshot = snapshot_dir.join("fish_conf_d").join("rixi.fish");
    if fish_snapshot.exists() {
        let dest = fish_config_path();
        paths::ensure_dir(&dest.parent().unwrap().to_path_buf())?;
        std::fs::copy(&fish_snapshot, &dest)?;
    }

    Ok(())
}
