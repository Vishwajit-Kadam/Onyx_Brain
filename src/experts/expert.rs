use serde::{Deserialize, Serialize};

use crate::{core::Task, memory::MemoryItem};

#[derive(Debug, Clone)]
pub struct ExpertContext {
    pub task: Task,
    pub memories: Vec<MemoryItem>,
    pub active_neurons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertResult {
    pub expert_name: String,
    pub summary: String,
    pub suggested_actions: Vec<String>,
    pub success: bool,
    pub estimated_cost: f32,
}

pub trait Expert {
    fn name(&self) -> &'static str;
    fn can_handle(&self, task: &Task) -> f32;
    fn estimate_cost(&self, task: &Task) -> f32;
    fn run(&self, context: &ExpertContext) -> ExpertResult;
}
