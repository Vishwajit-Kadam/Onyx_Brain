use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::storage::DiskStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub task_id: String,
    pub current_goal: String,
    pub planned_steps: Vec<String>,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    pub status: String,
}

impl Checkpoint {
    pub fn save(&self, store: &DiskStore) -> Result<()> {
        store.write_checkpoint(&self.task_id, self)
    }
}
