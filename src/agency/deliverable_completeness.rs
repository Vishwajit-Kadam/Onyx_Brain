use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{
    artifacts::{artifact_pack_inspect, ArtifactPackInspection},
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeliverableCompletenessReport {
    pub session_id: String,
    pub required_deliverables: Vec<String>,
    pub completed_deliverables: Vec<String>,
    pub missing_deliverables: Vec<String>,
    pub incomplete_deliverables: Vec<String>,
    pub completion_score: f32,
}

pub fn check_deliverable_completeness(
    store: &DiskStore,
    selector: &str,
    required: &[String],
) -> Result<DeliverableCompletenessReport> {
    let inspection = artifact_pack_inspect(store, selector)?;
    Ok(check_pack_completeness(inspection, required))
}

pub fn check_pack_completeness(
    inspection: ArtifactPackInspection,
    required: &[String],
) -> DeliverableCompletenessReport {
    let mut completed = Vec::new();
    let mut missing = Vec::new();
    let mut incomplete = Vec::new();
    for required_name in required {
        let found = inspection
            .artifacts
            .iter()
            .find(|row| row.contains(required_name));
        if let Some(row) = found {
            let path = row.split('|').next().unwrap_or("").trim();
            if std::fs::read_to_string(path)
                .map(|content| content.trim().is_empty())
                .unwrap_or(true)
            {
                incomplete.push(required_name.clone());
            } else {
                completed.push(required_name.clone());
            }
        } else {
            missing.push(required_name.clone());
        }
    }
    if !PathBuf::from(&inspection.manifest_path).exists() {
        incomplete.push("artifact_pack.json".to_string());
    }
    let total = required.len().max(1) as f32;
    let score = (completed.len() as f32 / total).clamp(0.0, 1.0);
    DeliverableCompletenessReport {
        session_id: session_from_manifest(&inspection.manifest_path),
        required_deliverables: required.to_vec(),
        completed_deliverables: completed,
        missing_deliverables: missing,
        incomplete_deliverables: incomplete,
        completion_score: score,
    }
}

fn session_from_manifest(path: &str) -> String {
    PathBuf::from(path)
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}
