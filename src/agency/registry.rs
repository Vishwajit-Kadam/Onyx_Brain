use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::{load_json, save_json, DiskStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub goal_id: String,
    pub project_name: String,
    pub root_path: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_report_path: Option<String>,
    pub tags: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectRecord>,
}

impl ProjectRegistry {
    pub fn find_by_name(&self, name: &str) -> Option<ProjectRecord> {
        self.projects
            .iter()
            .find(|project| project.project_name.eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn upsert(&mut self, record: ProjectRecord) {
        if let Some(existing) = self
            .projects
            .iter_mut()
            .find(|project| project.project_name == record.project_name)
        {
            *existing = record;
        } else {
            self.projects.push(record);
        }
        self.projects.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then(a.project_name.cmp(&b.project_name))
        });
    }
}

pub fn registry_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.projects.join("project_registry.json")
}

pub fn load_project_registry(store: &DiskStore) -> Result<ProjectRegistry> {
    let path = registry_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(ProjectRegistry::default())
    }
}

pub fn save_project_registry(store: &DiskStore, registry: &ProjectRegistry) -> Result<()> {
    save_json(&registry_path(store), registry)
}

pub fn register_project(store: &DiskStore, record: ProjectRecord) -> Result<()> {
    let mut registry = load_project_registry(store)?;
    registry.upsert(record);
    save_project_registry(store, &registry)
}
