use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceStatus {
    Created,
    Active,
    Completed,
    Archived,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub workspace_id: String,
    pub session_id: String,
    pub root_path: String,
    pub artifacts_path: String,
    pub reports_path: String,
    pub temp_path: String,
    pub status: WorkspaceStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceIndex {
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceOverview {
    pub workspaces: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceInspection {
    pub workspace_id: String,
    pub session_id: String,
    pub root_path: String,
    pub files: Vec<String>,
    pub status: String,
}

pub fn workspace_index_path(store: &DiskStore) -> PathBuf {
    store.paths.indexes.join("workspace_index.json")
}

pub fn load_workspace_index(store: &DiskStore) -> Result<WorkspaceIndex> {
    let path = workspace_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(WorkspaceIndex::default())
    }
}

pub fn create_workspace(store: &DiskStore, session_id: &str) -> Result<Workspace> {
    let root = store.paths.sandbox.join("workspaces").join(session_id);
    let artifacts = root.join("artifacts");
    let reports = root.join("reports");
    let temp = root.join("temp");
    for dir in [&artifacts, &reports, &temp] {
        fs::create_dir_all(dir)?;
    }
    let workspace = Workspace {
        workspace_id: format!("workspace_{}", timestamp_slug()),
        session_id: session_id.to_string(),
        root_path: root.display().to_string(),
        artifacts_path: artifacts.display().to_string(),
        reports_path: reports.display().to_string(),
        temp_path: temp.display().to_string(),
        status: WorkspaceStatus::Active,
        created_at: Utc::now(),
    };
    save_json(&root.join("manifest.json"), &workspace)?;
    fs::write(
        root.join("workspace_report.md"),
        format!("# Workspace Report\n\nSession: {session_id}\nStatus: Active\n"),
    )?;
    let mut index = load_workspace_index(store)?;
    index.workspaces.retain(|row| row.session_id != session_id);
    index.workspaces.push(workspace.clone());
    index
        .workspaces
        .sort_by(|a, b| b.created_at.cmp(&a.created_at));
    save_json(&workspace_index_path(store), &index)?;
    Ok(workspace)
}

pub fn workspaces(store: &DiskStore) -> Result<WorkspaceOverview> {
    let index = load_workspace_index(store)?;
    Ok(WorkspaceOverview {
        count: index.workspaces.len(),
        workspaces: index
            .workspaces
            .iter()
            .take(25)
            .map(|workspace| {
                format!(
                    "{} | {:?} | {}",
                    workspace.session_id, workspace.status, workspace.root_path
                )
            })
            .collect(),
    })
}

pub fn workspace_inspect(store: &DiskStore, selector: &str) -> Result<WorkspaceInspection> {
    let index = load_workspace_index(store)?;
    let workspace = if selector.eq_ignore_ascii_case("latest") {
        index
            .workspaces
            .first()
            .ok_or_else(|| anyhow::anyhow!("no workspaces found"))?
            .clone()
    } else {
        index
            .workspaces
            .into_iter()
            .find(|row| row.session_id == selector || row.workspace_id == selector)
            .ok_or_else(|| anyhow::anyhow!("workspace not found"))?
    };
    let mut files = Vec::new();
    for entry in fs::read_dir(&workspace.root_path)? {
        files.push(entry?.path().display().to_string());
    }
    Ok(WorkspaceInspection {
        workspace_id: workspace.workspace_id,
        session_id: workspace.session_id,
        root_path: workspace.root_path,
        files,
        status: format!("{:?}", workspace.status),
    })
}
