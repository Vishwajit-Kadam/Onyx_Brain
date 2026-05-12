use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::{
    core::{Priority, TaskType},
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    pub id: String,
    pub parent_goal_id: String,
    pub title: String,
    pub description: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub dependencies: Vec<String>,
    pub result_summary: Option<String>,
    pub error_summary: Option<String>,
}

impl QueuedTask {
    pub fn new(
        goal_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        task_type: TaskType,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            parent_goal_id: goal_id.into(),
            title: title.into(),
            description: description.into(),
            task_type,
            status: TaskStatus::Pending,
            priority: Priority::Normal,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            attempts: 0,
            max_attempts: 2,
            dependencies: Vec::new(),
            result_summary: None,
            error_summary: None,
        }
    }
}

pub fn queue_path(store: &DiskStore, goal_id: &str) -> std::path::PathBuf {
    store.paths.projects.join(goal_id).join("task_queue.json")
}

pub fn save_task_queue(store: &DiskStore, goal_id: &str, queue: &[QueuedTask]) -> Result<()> {
    save_json(&queue_path(store, goal_id), &queue)
}

pub fn load_task_queue(store: &DiskStore, goal_id: &str) -> Result<Vec<QueuedTask>> {
    let path = queue_path(store, goal_id);
    if Path::new(&path).exists() {
        load_json(&path)
    } else {
        Ok(Vec::new())
    }
}
