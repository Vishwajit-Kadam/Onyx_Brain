use serde::{Deserialize, Serialize};

use crate::core::Task;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub task_id: String,
    pub description: String,
}

impl From<&Task> for Goal {
    fn from(task: &Task) -> Self {
        Self {
            task_id: task.id.clone(),
            description: task.input.clone(),
        }
    }
}
