use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{presentation_audience, GoalType, GoalUnderstanding},
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProfile {
    pub workspace_id: String,
    pub session_id: String,
    pub goal_title: String,
    pub goal_type: GoalType,
    pub audience: Option<String>,
    pub tone: Option<String>,
    pub output_style: Option<String>,
    pub deliverable_count: usize,
    pub artifact_types: Vec<String>,
    pub safety_level: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub fn write_workspace_profile(
    store: &DiskStore,
    session_id: &str,
    workspace_id: &str,
    understanding: &GoalUnderstanding,
) -> Result<WorkspaceProfile> {
    let lower = understanding.original_prompt.to_lowercase();
    let profile = WorkspaceProfile {
        workspace_id: workspace_id.to_string(),
        session_id: session_id.to_string(),
        goal_title: understanding.original_prompt.clone(),
        goal_type: understanding.goal_type.clone(),
        audience: Some(presentation_audience(&understanding.original_prompt)),
        tone: Some(if lower.contains("students") {
            "clear and instructional".to_string()
        } else if lower.contains("launch") || lower.contains("startup") {
            "practical and concise".to_string()
        } else {
            "neutral".to_string()
        }),
        output_style: Some("export-ready markdown".to_string()),
        deliverable_count: understanding.deliverables.len(),
        artifact_types: understanding
            .deliverables
            .iter()
            .map(|deliverable| format!("{:?}", deliverable.kind))
            .collect(),
        safety_level: "bounded autonomy; sandboxed writes; no network by default".to_string(),
        created_at: chrono::Utc::now(),
    };
    let root = store.paths.sandbox.join("workspaces").join(session_id);
    fs::create_dir_all(&root)?;
    save_json(&root.join("workspace_profile.json"), &profile)?;
    fs::write(
        root.join("workspace_profile.md"),
        format!(
            "# Workspace Profile\n\nGoal: {}\nGoal type: {:?}\nAudience: {}\nTone: {}\nOutput style: {}\nDeliverables: {}\nSafety: {}\n",
            profile.goal_title,
            profile.goal_type,
            profile.audience.clone().unwrap_or_default(),
            profile.tone.clone().unwrap_or_default(),
            profile.output_style.clone().unwrap_or_default(),
            profile.deliverable_count,
            profile.safety_level
        ),
    )?;
    Ok(profile)
}
