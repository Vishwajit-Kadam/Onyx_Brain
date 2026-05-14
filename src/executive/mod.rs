pub mod attention;
pub mod continuity;
pub mod control_loop;
pub mod goals;
pub mod metacognition;
pub mod reflection;
pub mod self_model;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::storage::DiskStore;

pub use attention::*;
pub use continuity::*;
pub use control_loop::*;
pub use goals::*;
pub use metacognition::*;
pub use reflection::*;
pub use self_model::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutiveStatus {
    pub active_goal: Option<String>,
    pub active_task: Option<String>,
    pub recent_decisions: Vec<String>,
    pub confidence: f32,
    pub safety_state: String,
}

pub fn executive_status(store: &DiskStore) -> Result<ExecutiveStatus> {
    let attention = attention_state(Some("No active executive goal".to_string()), None);
    let decisions = recent_decisions(store, 5)?;
    Ok(ExecutiveStatus {
        active_goal: attention.active_goal,
        active_task: attention.active_task,
        recent_decisions: decisions.into_iter().map(|row| row.chosen_action).collect(),
        confidence: attention.focus_score,
        safety_state: "bounded; sandbox and allowlist preserved".to_string(),
    })
}
