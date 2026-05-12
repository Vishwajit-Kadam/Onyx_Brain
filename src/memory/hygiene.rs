use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    storage::{save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHygienePolicy {
    pub max_duplicate_project_memories_per_project: usize,
    pub max_backups_per_file: usize,
    pub archive_project_memories_not_in_registry: bool,
    pub archive_low_importance_old_memories: bool,
    pub dedup_after_project_run: bool,
    pub dedup_after_benchmark: bool,
}

impl Default for MemoryHygienePolicy {
    fn default() -> Self {
        Self {
            max_duplicate_project_memories_per_project: 1,
            max_backups_per_file: 3,
            archive_project_memories_not_in_registry: true,
            archive_low_importance_old_memories: false,
            dedup_after_project_run: true,
            dedup_after_benchmark: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupCleanupReport {
    pub backups_seen: usize,
    pub backups_removed: usize,
    pub report_path: String,
}

pub fn cleanup_backups(store: &DiskStore, keep_latest: usize) -> Result<BackupCleanupReport> {
    let mut groups: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let root = store.paths.sandbox.join("projects");
    if root.exists() {
        collect_backups(&root, &mut groups)?;
    }
    let backups_seen = groups.values().map(Vec::len).sum();
    let mut backups_removed = 0;
    for backups in groups.values_mut() {
        backups.sort_by(|a, b| modified(b).cmp(&modified(a)));
        for old in backups.iter().skip(keep_latest) {
            if old.exists() {
                fs::remove_file(old)?;
                backups_removed += 1;
            }
        }
    }
    let report_name = format!("backup_cleanup_report_{}", timestamp_slug());
    let report_path = store.paths.logs.join(format!("{report_name}.json"));
    let report = BackupCleanupReport {
        backups_seen,
        backups_removed,
        report_path: report_path.display().to_string(),
    };
    save_json(&report_path, &report)?;
    Ok(report)
}

fn collect_backups(dir: &Path, groups: &mut BTreeMap<String, Vec<PathBuf>>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_backups(&path, groups)?;
        } else if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            if let Some((base, _)) = name.split_once(".bak.") {
                groups
                    .entry(path.with_file_name(base).display().to_string())
                    .or_default()
                    .push(path);
            }
        }
    }
    Ok(())
}

fn modified(path: &Path) -> Option<std::time::SystemTime> {
    path.metadata().and_then(|meta| meta.modified()).ok()
}
