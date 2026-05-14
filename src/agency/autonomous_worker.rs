use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutonomyLevel {
    Assisted,
    Standard,
    High,
    FullBounded,
    ReviewOnly,
    RepairOnly,
    Studio,
    Executive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerStatus {
    Completed,
    CompletedWithWarnings,
    Failed,
    Blocked,
    SafetyStopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousWorkerConfig {
    pub max_session_minutes: u64,
    pub max_tasks: usize,
    pub max_phases: usize,
    pub max_retries_per_task: usize,
    pub max_tool_actions: usize,
    pub allow_research: bool,
    pub allow_artifact_creation: bool,
    pub allow_project_modification: bool,
    pub require_human_approval: bool,
    pub autonomy_level: AutonomyLevel,
}

impl AutonomousWorkerConfig {
    pub fn for_level(level: AutonomyLevel) -> Self {
        let require_human_approval = level == AutonomyLevel::Assisted;
        Self {
            max_session_minutes: 30,
            max_tasks: 40,
            max_phases: 8,
            max_retries_per_task: 2,
            max_tool_actions: 80,
            allow_research: false,
            allow_artifact_creation: true,
            allow_project_modification: true,
            require_human_approval,
            autonomy_level: level,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousWorkerResult {
    pub session_id: String,
    pub goal_id: String,
    pub status: WorkerStatus,
    pub tasks_planned: usize,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub artifacts_created: Vec<String>,
    pub recovery_actions: Vec<String>,
    pub validation_passed: bool,
    pub reliability_score: f32,
    pub autonomy_score: f32,
    pub final_report_path: String,
}
