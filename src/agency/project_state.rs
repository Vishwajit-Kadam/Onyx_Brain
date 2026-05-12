use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::storage::{load_json, save_json, DiskStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub goal_id: String,
    pub project_name: String,
    pub root_path: String,
    pub original_prompt: String,
    pub status: String,
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
    pub commands_run: Vec<String>,
    pub errors_seen: Vec<String>,
    pub checkpoints: Vec<String>,
    pub final_summary: Option<String>,
    pub final_report_path: Option<String>,
    pub retries_used: u32,
    #[serde(default)]
    pub self_evaluation: Option<crate::core::SelfEvaluation>,
}

impl ProjectState {
    pub fn new(
        goal_id: impl Into<String>,
        project_name: impl Into<String>,
        root_path: impl Into<String>,
        original_prompt: impl Into<String>,
    ) -> Self {
        Self {
            goal_id: goal_id.into(),
            project_name: project_name.into(),
            root_path: root_path.into(),
            original_prompt: original_prompt.into(),
            status: "Running".to_string(),
            files_created: Vec::new(),
            files_modified: Vec::new(),
            commands_run: Vec::new(),
            errors_seen: Vec::new(),
            checkpoints: Vec::new(),
            final_summary: None,
            final_report_path: None,
            retries_used: 0,
            self_evaluation: None,
        }
    }

    pub fn remember_checkpoint(&mut self, checkpoint: impl Into<String>) {
        self.checkpoints.push(checkpoint.into());
    }
}

pub fn project_dir(store: &DiskStore, goal_id: &str) -> PathBuf {
    store.paths.projects.join(goal_id)
}

pub fn project_state_path(store: &DiskStore, goal_id: &str) -> PathBuf {
    project_dir(store, goal_id).join("project_state.json")
}

pub fn save_project_state(store: &DiskStore, state: &ProjectState) -> Result<()> {
    save_json(&project_state_path(store, &state.goal_id), state)
}

pub fn load_project_state(store: &DiskStore, goal_id: &str) -> Result<ProjectState> {
    load_json(&project_state_path(store, goal_id))
}
