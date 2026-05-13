use serde::{Deserialize, Serialize};

use crate::agency::WorkerStatus;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutonomyScore {
    pub goal_understanding_score: f32,
    pub plan_completion_score: f32,
    pub artifact_completion_score: f32,
    pub validation_score: f32,
    pub repair_score: f32,
    pub safety_score: f32,
    pub reliability_score: f32,
    pub overall: f32,
}

pub fn calculate_autonomy_score(
    status: &WorkerStatus,
    tasks_planned: usize,
    tasks_completed: usize,
    artifact_count: usize,
    validation_score: f32,
    repairs_performed: usize,
    reliability_score: f32,
) -> AutonomyScore {
    let plan_completion_score = if tasks_planned == 0 {
        0.0
    } else {
        tasks_completed as f32 / tasks_planned as f32
    }
    .clamp(0.0, 1.0);
    let artifact_completion_score = (artifact_count as f32 / 5.0).clamp(0.0, 1.0);
    let repair_score = if repairs_performed == 0 { 0.9 } else { 1.0 };
    let safety_score = match status {
        WorkerStatus::SafetyStopped => 0.2,
        WorkerStatus::Failed => 0.5,
        WorkerStatus::Blocked => 0.6,
        WorkerStatus::CompletedWithWarnings => 0.85,
        WorkerStatus::Completed => 1.0,
    };
    let goal_understanding_score = 0.9;
    let overall = goal_understanding_score * 0.15
        + plan_completion_score * 0.2
        + artifact_completion_score * 0.15
        + validation_score * 0.2
        + repair_score * 0.1
        + safety_score * 0.1
        + reliability_score * 0.1;
    AutonomyScore {
        goal_understanding_score,
        plan_completion_score,
        artifact_completion_score,
        validation_score,
        repair_score,
        safety_score,
        reliability_score,
        overall: overall.clamp(0.0, 1.0),
    }
}
