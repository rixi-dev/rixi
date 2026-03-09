use crate::errors::{Result, RixiError};
use crate::paths;
use crate::registry;

/// Create a snapshot of all files that are about to be overwritten.
/// Returns the snapshot timestamp string used as the directory name.
pub fn create_snapshot(components: &[String]) -> Result<String> {
    let registry = registry::builtin_registry();
    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let snapshot_dir = paths::snapshots_dir().join(&timestamp);

    paths::ensure_dir(&snapshot_dir)?;

    for component in components {
        let entry = registry
            .get(component.as_str())
            .ok_or_else(|| RixiError::UnknownComponent(component.clone()))?;

        for raw_path in &entry.paths {
            let src = paths::expand_tilde(raw_path);
            if src.exists() {
                // Preserve the component directory structure inside the snapshot
                let dest = snapshot_dir.join(component).join(
                    src.file_name()
                        .expect("config file should have a filename"),
                );
                paths::ensure_dir(&dest.parent().unwrap().to_path_buf())?;
                std::fs::copy(&src, &dest)?;
            }
        }
    }

    Ok(timestamp)
}

/// Restore files from a snapshot back to their XDG paths.
pub fn restore_snapshot(snapshot_id: &str) -> Result<Vec<String>> {
    let registry = registry::builtin_registry();
    let snapshot_dir = paths::snapshots_dir().join(snapshot_id);

    if !snapshot_dir.exists() {
        return Err(RixiError::SnapshotNotFound(snapshot_dir));
    }

    let mut restored = Vec::new();

    // Each subdirectory in the snapshot is a component name
    for entry in std::fs::read_dir(&snapshot_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let component_name = entry.file_name().to_string_lossy().to_string();

        if let Some(reg_entry) = registry.get(component_name.as_str()) {
            // Restore each file from the snapshot back to its XDG path
            for raw_path in &reg_entry.paths {
                let dest = paths::expand_tilde(raw_path);
                let src = snapshot_dir.join(&component_name).join(
                    dest.file_name()
                        .expect("config file should have a filename"),
                );

                if src.exists() {
                    paths::ensure_dir(&dest.parent().unwrap().to_path_buf())?;
                    std::fs::copy(&src, &dest)?;
                }
            }
            restored.push(component_name);
        }
    }

    Ok(restored)
}
