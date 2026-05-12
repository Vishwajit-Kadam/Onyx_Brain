use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::ids::TaskId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskType {
    Chat,
    Code,
    FileOperation,
    Planning,
    Reasoning,
    MemoryQuery,
    ToolUse,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub input: String,
    pub task_type: TaskType,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub constraints: Vec<String>,
    pub required_tools: Vec<String>,
}

impl Task {
    pub fn new(input: String, task_type: TaskType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            input,
            task_type,
            priority: Priority::Normal,
            created_at: Utc::now(),
            constraints: Vec::new(),
            required_tools: Vec::new(),
        }
    }
}
