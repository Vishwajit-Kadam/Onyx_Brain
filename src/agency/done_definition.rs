use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{Deliverable, GoalUnderstanding},
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DoneDefinition {
    pub session_id: String,
    pub required_artifacts: Vec<String>,
    pub required_validations: Vec<String>,
    pub required_reports: Vec<String>,
    pub minimum_quality_score: f32,
    pub minimum_completeness_score: f32,
    pub allow_warnings: bool,
}

pub fn generate_done_definition(
    session_id: &str,
    understanding: &GoalUnderstanding,
) -> DoneDefinition {
    DoneDefinition {
        session_id: session_id.to_string(),
        required_artifacts: understanding
            .deliverables
            .iter()
            .filter(|deliverable| deliverable.required)
            .map(deliverable_name)
            .collect(),
        required_validations: vec![
            "artifact manifest exists".to_string(),
            "quality review completed".to_string(),
            "final audit completed".to_string(),
        ],
        required_reports: vec![
            "final_report.md".to_string(),
            "report_card.json".to_string(),
            "final_audit.json".to_string(),
        ],
        minimum_quality_score: 0.75,
        minimum_completeness_score: 0.85,
        allow_warnings: true,
    }
}

pub fn write_done_definition(store: &DiskStore, done: &DoneDefinition) -> Result<(String, String)> {
    let root = store
        .paths
        .sandbox
        .join("workspaces")
        .join(&done.session_id);
    fs::create_dir_all(&root)?;
    let json_path = root.join("done_definition.json");
    let md_path = root.join("done_definition.md");
    save_json(&json_path, done)?;
    fs::write(&md_path, render_done_definition(done))?;
    Ok((
        md_path.display().to_string(),
        json_path.display().to_string(),
    ))
}

fn deliverable_name(deliverable: &Deliverable) -> String {
    deliverable
        .path_hint
        .clone()
        .unwrap_or_else(|| deliverable.title.clone())
}

fn render_done_definition(done: &DoneDefinition) -> String {
    format!(
        "# Done Definition\n\n## Required Artifacts\n{}\n\n## Required Validations\n{}\n\n## Required Reports\n{}\n\nMinimum quality score: {:.2}\nMinimum completeness score: {:.2}\nWarnings allowed: {}\n",
        done.required_artifacts
            .iter()
            .map(|row| format!("- {row}"))
            .collect::<Vec<_>>()
            .join("\n"),
        done.required_validations
            .iter()
            .map(|row| format!("- {row}"))
            .collect::<Vec<_>>()
            .join("\n"),
        done.required_reports
            .iter()
            .map(|row| format!("- {row}"))
            .collect::<Vec<_>>()
            .join("\n"),
        done.minimum_quality_score,
        done.minimum_completeness_score,
        done.allow_warnings
    )
}
