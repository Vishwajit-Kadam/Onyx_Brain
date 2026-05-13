use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{AutonomousWorkerConfig, WorkerStatus},
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterOrchestrator {
    pub session_id: String,
    pub goal_id: String,
    pub config: AutonomousWorkerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrchestratorStatus {
    Completed,
    CompletedWithWarnings,
    Failed,
    Blocked,
    SafetyStopped,
}

impl From<&WorkerStatus> for OrchestratorStatus {
    fn from(value: &WorkerStatus) -> Self {
        match value {
            WorkerStatus::Completed => OrchestratorStatus::Completed,
            WorkerStatus::CompletedWithWarnings => OrchestratorStatus::CompletedWithWarnings,
            WorkerStatus::Failed => OrchestratorStatus::Failed,
            WorkerStatus::Blocked => OrchestratorStatus::Blocked,
            WorkerStatus::SafetyStopped => OrchestratorStatus::SafetyStopped,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorResult {
    pub session_id: String,
    pub goal_id: String,
    pub status: OrchestratorStatus,
    pub tasks_planned: usize,
    pub tasks_completed: usize,
    pub artifacts_created: usize,
    pub validation_score: f32,
    pub quality_score: f32,
    pub consistency_score: f32,
    pub report_card_grade: String,
    pub export_path: Option<String>,
    pub final_report_path: String,
}

pub fn save_orchestrator_result(store: &DiskStore, result: &OrchestratorResult) -> Result<()> {
    let reports = store
        .paths
        .sandbox
        .join("workspaces")
        .join(&result.session_id)
        .join("reports");
    fs::create_dir_all(&reports)?;
    save_json(&reports.join("orchestrator_result.json"), result)?;
    fs::write(
        reports.join("orchestrator_result.md"),
        format!(
            "# Master Orchestrator Result\n\nSession: {}\nStatus: {:?}\nTasks planned: {}\nTasks completed: {}\nArtifacts created: {}\nValidation score: {:.2}\nQuality score: {:.2}\nConsistency score: {:.2}\nReport grade: {}\nFinal report: {}\n",
            result.session_id,
            result.status,
            result.tasks_planned,
            result.tasks_completed,
            result.artifacts_created,
            result.validation_score,
            result.quality_score,
            result.consistency_score,
            result.report_card_grade,
            result.final_report_path
        ),
    )?;
    Ok(())
}
