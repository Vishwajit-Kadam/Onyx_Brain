use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{load_session, WorkSession},
    artifacts::artifact_inspect,
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionDashboardReport {
    pub session_id: String,
    pub goal: String,
    pub status: String,
    pub phases: Vec<String>,
    pub tasks_completed: usize,
    pub artifacts_created: Vec<String>,
    pub validation_score: f32,
    pub repairs_performed: usize,
    pub reliability_score: f32,
    pub final_report_path: String,
    pub markdown_report_path: String,
    pub json_report_path: String,
}

pub fn write_session_report(
    store: &DiskStore,
    selector: &str,
    goal: Option<String>,
    phases: Vec<String>,
    tasks_completed: usize,
    repairs_performed: usize,
    reliability_score: f32,
) -> Result<SessionDashboardReport> {
    let session: WorkSession = load_session(store, selector)?;
    let artifacts = artifact_inspect(store, &session.session_id).unwrap_or_default();
    let session_dir = store.paths.sessions.join(&session.session_id);
    fs::create_dir_all(&session_dir)?;
    let markdown_path = session_dir.join("session_report.md");
    let json_path = session_dir.join("session_report.json");
    let report = SessionDashboardReport {
        session_id: session.session_id.clone(),
        goal: goal.unwrap_or(session.title.clone()),
        status: format!("{:?}", session.status),
        phases,
        tasks_completed,
        artifacts_created: artifacts.files.clone(),
        validation_score: artifacts.validation_score,
        repairs_performed,
        reliability_score,
        final_report_path: artifacts.report_path.clone().unwrap_or_default(),
        markdown_report_path: markdown_path.display().to_string(),
        json_report_path: json_path.display().to_string(),
    };
    let markdown = format!(
        "# Session Report\n\nSession: {}\nGoal: {}\nStatus: {}\nTasks completed: {}\nValidation score: {:.2}\nRepairs performed: {}\nReliability score: {:.2}\nFinal report: {}\n\nArtifacts:\n{}\n",
        report.session_id,
        report.goal,
        report.status,
        report.tasks_completed,
        report.validation_score,
        report.repairs_performed,
        report.reliability_score,
        report.final_report_path,
        report
            .artifacts_created
            .iter()
            .map(|path| format!("- {path}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(&markdown_path, markdown)?;
    save_json(&json_path, &report)?;
    Ok(report)
}
