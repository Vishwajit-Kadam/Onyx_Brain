use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::storage::DiskStore;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextDiscoveryResult {
    pub files_considered: Vec<String>,
    pub summaries: Vec<String>,
    pub relevant_memories: Vec<String>,
    pub risks: Vec<String>,
    pub skipped_paths: Vec<String>,
}

pub fn discover_local_context(
    store: &DiskStore,
    max_files: usize,
) -> Result<ContextDiscoveryResult> {
    let mut result = ContextDiscoveryResult::default();
    for root in [
        &store.paths.root.join("docs"),
        &store.paths.root.join("src"),
        &store.paths.sandbox,
    ] {
        collect_files(root, &store.paths.root, max_files, &mut result)?;
        if result.files_considered.len() >= max_files {
            break;
        }
    }
    result.summaries.push(format!(
        "considered {} local files inside allowed roots",
        result.files_considered.len()
    ));
    result
        .risks
        .push("network and external connectors skipped".to_string());
    Ok(result)
}

fn collect_files(
    root: &Path,
    allowed_root: &Path,
    max_files: usize,
    result: &mut ContextDiscoveryResult,
) -> Result<()> {
    if !root.exists() || result.files_considered.len() >= max_files {
        return Ok(());
    }
    let root = root.canonicalize()?;
    if !root.starts_with(allowed_root.canonicalize()?) {
        result.skipped_paths.push(root.display().to_string());
        return Ok(());
    }
    for entry in fs::read_dir(&root)? {
        if result.files_considered.len() >= max_files {
            break;
        }
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name.contains("secret") || name == "target" || name == ".git" {
            result.skipped_paths.push(path.display().to_string());
            continue;
        }
        if path.is_dir() {
            collect_files(&path, allowed_root, max_files, result)?;
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("rs" | "md" | "json" | "toml")
        ) {
            result.files_considered.push(path.display().to_string());
        }
    }
    Ok(())
}
