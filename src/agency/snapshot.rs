use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    agency::{load_project_registry, load_project_state, quick_journal, ActionType},
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

const MAX_SNAPSHOT_FILE_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFile {
    pub relative_path: String,
    pub content_path: String,
    pub hash: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub snapshot_id: String,
    pub project_id: String,
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub reason: String,
    pub files: Vec<SnapshotFile>,
    pub project_state_ref: Option<String>,
    pub task_queue_ref: Option<String>,
    pub total_bytes: u64,
    #[serde(default)]
    pub skipped_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapshotOverview {
    pub snapshots: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapshotRestoreReport {
    pub snapshot_id: String,
    pub project_name: String,
    pub files_restored: u64,
    pub pre_restore_snapshot_id: Option<String>,
    pub status: String,
    pub errors: Vec<String>,
}

pub fn snapshot_create(
    store: &DiskStore,
    project_name: &str,
    reason: &str,
) -> Result<ProjectSnapshot> {
    store.ensure_layout()?;
    let registry = load_project_registry(store)?;
    let record = registry
        .find_by_name(project_name)
        .ok_or_else(|| anyhow::anyhow!("project not found in registry: {project_name}"))?;
    let project_root = PathBuf::from(&record.root_path).canonicalize()?;
    let sandbox = store.paths.sandbox.canonicalize()?;
    if !project_root.starts_with(&sandbox) {
        return Err(anyhow::anyhow!("refusing to snapshot outside sandbox"));
    }
    let snapshot_id = format!("snapshot_{}_{}", timestamp_slug(), Uuid::new_v4());
    let snapshot_root = snapshot_root(store, &record.goal_id, &snapshot_id);
    let files_root = snapshot_root.join("files");
    fs::create_dir_all(&files_root)?;

    let mut files = Vec::new();
    let mut skipped_files = Vec::new();
    let mut total_bytes = 0;
    collect_snapshot_files(
        &project_root,
        &project_root,
        &files_root,
        &mut files,
        &mut skipped_files,
        &mut total_bytes,
    )?;

    let state_path = store
        .paths
        .projects
        .join(&record.goal_id)
        .join("project_state.json");
    let project_state_ref = if state_path.exists() {
        let dest = snapshot_root.join("project_state.json");
        fs::copy(&state_path, &dest)?;
        Some(dest.display().to_string())
    } else {
        None
    };
    let queue_path = store
        .paths
        .projects
        .join(&record.goal_id)
        .join("task_queue.json");
    let task_queue_ref = if queue_path.exists() {
        let dest = snapshot_root.join("task_queue.json");
        fs::copy(&queue_path, &dest)?;
        Some(dest.display().to_string())
    } else {
        None
    };
    let snapshot = ProjectSnapshot {
        snapshot_id: snapshot_id.clone(),
        project_id: record.goal_id.clone(),
        project_name: project_name.to_string(),
        created_at: Utc::now(),
        reason: reason.to_string(),
        files,
        project_state_ref,
        task_queue_ref,
        total_bytes,
        skipped_files,
    };
    save_json(&snapshot_root.join("snapshot.json"), &snapshot)?;
    let _ = quick_journal(
        store,
        "system",
        ActionType::CreateSnapshot,
        Some(record.goal_id),
        Some(project_name.to_string()),
        None,
        Some(snapshot_id.clone()),
    );
    Ok(snapshot)
}

pub fn snapshot_restore(store: &DiskStore, snapshot_id: &str) -> Result<SnapshotRestoreReport> {
    let snapshot = load_snapshot_by_id(store, snapshot_id)?;
    let pre = snapshot_create(
        store,
        &snapshot.project_name,
        &format!("pre-restore before {snapshot_id}"),
    )
    .ok();
    let registry = load_project_registry(store)?;
    let record = registry
        .find_by_name(&snapshot.project_name)
        .ok_or_else(|| anyhow::anyhow!("project not found: {}", snapshot.project_name))?;
    let root = PathBuf::from(&record.root_path).canonicalize()?;
    let sandbox = store.paths.sandbox.canonicalize()?;
    if !root.starts_with(&sandbox) {
        return Err(anyhow::anyhow!("refusing to restore outside sandbox"));
    }
    let mut files_restored = 0;
    let mut errors = Vec::new();
    for file in &snapshot.files {
        let src = PathBuf::from(&file.content_path);
        let dest = root.join(&file.relative_path);
        if !dest.starts_with(&root) {
            errors.push(format!("refused unsafe path {}", file.relative_path));
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::copy(&src, &dest) {
            Ok(_) => files_restored += 1,
            Err(error) => errors.push(error.to_string()),
        }
    }
    if let Some(state_ref) = &snapshot.project_state_ref {
        let dest = store
            .paths
            .projects
            .join(&snapshot.project_id)
            .join("project_state.json");
        let _ = fs::copy(state_ref, dest);
    }
    if let Some(queue_ref) = &snapshot.task_queue_ref {
        let dest = store
            .paths
            .projects
            .join(&snapshot.project_id)
            .join("task_queue.json");
        let _ = fs::copy(queue_ref, dest);
    }
    let status = if errors.is_empty() {
        "Completed"
    } else {
        "Partial"
    }
    .to_string();
    let _ = quick_journal(
        store,
        "system",
        ActionType::RestoreSnapshot,
        Some(snapshot.project_id.clone()),
        Some(snapshot.project_name.clone()),
        None,
        Some(snapshot_id.to_string()),
    );
    Ok(SnapshotRestoreReport {
        snapshot_id: snapshot_id.to_string(),
        project_name: snapshot.project_name,
        files_restored,
        pre_restore_snapshot_id: pre.map(|snapshot| snapshot.snapshot_id),
        status,
        errors,
    })
}

pub fn snapshots(store: &DiskStore) -> Result<SnapshotOverview> {
    let mut rows = Vec::new();
    if store.paths.snapshots.exists() {
        for project in fs::read_dir(&store.paths.snapshots)? {
            let project = project?.path();
            if !project.is_dir() {
                continue;
            }
            for entry in fs::read_dir(project)? {
                let path = entry?.path().join("snapshot.json");
                if path.exists() {
                    if let Ok(snapshot) = load_json::<ProjectSnapshot>(&path) {
                        rows.push(format!(
                            "{} | {} | {} | {} bytes | {}",
                            snapshot.project_name,
                            snapshot.snapshot_id,
                            snapshot.created_at,
                            snapshot.total_bytes,
                            snapshot.reason
                        ));
                    }
                }
            }
        }
    }
    rows.sort();
    rows.reverse();
    Ok(SnapshotOverview {
        count: rows.len(),
        snapshots: rows,
    })
}

pub fn snapshot_count(store: &DiskStore) -> Result<usize> {
    Ok(snapshots(store)?.count)
}

pub fn load_snapshot_by_id(store: &DiskStore, snapshot_id: &str) -> Result<ProjectSnapshot> {
    if store.paths.snapshots.exists() {
        for project in fs::read_dir(&store.paths.snapshots)? {
            let project = project?.path();
            if !project.is_dir() {
                continue;
            }
            let path = project.join(snapshot_id).join("snapshot.json");
            if path.exists() {
                return load_json(&path);
            }
        }
    }
    Err(anyhow::anyhow!("snapshot not found: {snapshot_id}"))
}

fn snapshot_root(store: &DiskStore, project_id: &str, snapshot_id: &str) -> PathBuf {
    store.paths.snapshots.join(project_id).join(snapshot_id)
}

fn collect_snapshot_files(
    root: &Path,
    current: &Path,
    files_root: &Path,
    files: &mut Vec<SnapshotFile>,
    skipped: &mut Vec<String>,
    total_bytes: &mut u64,
) -> Result<()> {
    for entry in fs::read_dir(current).with_context(|| format!("reading {}", current.display()))? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if matches!(name, "target" | ".git") {
                skipped.push(path.display().to_string());
                continue;
            }
            collect_snapshot_files(root, &path, files_root, files, skipped, total_bytes)?;
            continue;
        }
        if name.contains(".bak.") {
            skipped.push(path.display().to_string());
            continue;
        }
        let size = fs::metadata(&path)?.len();
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        if size > MAX_SNAPSHOT_FILE_BYTES {
            skipped.push(relative);
            continue;
        }
        let content = fs::read(&path)?;
        let dest = files_root.join(&relative);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, &content)?;
        let hash = simple_hash(&content);
        *total_bytes += size;
        files.push(SnapshotFile {
            relative_path: relative,
            content_path: dest.display().to_string(),
            hash,
            size_bytes: size,
        });
    }
    Ok(())
}

fn simple_hash(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn project_state_exists(store: &DiskStore, project_id: &str) -> bool {
    load_project_state(store, project_id).is_ok()
}
