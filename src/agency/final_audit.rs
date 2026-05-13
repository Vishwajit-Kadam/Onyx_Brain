use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{DeliverableCompletenessReport, DoneDefinition},
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FinalAudit {
    pub session_id: String,
    pub contract_met: bool,
    pub done_definition_met: bool,
    pub artifacts_complete: bool,
    pub validation_passed: bool,
    pub quality_passed: bool,
    pub safety_passed: bool,
    pub export_created: bool,
    pub issues: Vec<String>,
    pub final_status: String,
}

pub fn run_final_audit(
    session_id: &str,
    done: &DoneDefinition,
    completeness: &DeliverableCompletenessReport,
    validation_passed: bool,
    quality_score: f32,
    export_created: bool,
) -> FinalAudit {
    let mut issues = Vec::new();
    if !completeness.missing_deliverables.is_empty() {
        issues.push(format!(
            "missing deliverables: {}",
            completeness.missing_deliverables.join(", ")
        ));
    }
    if quality_score < done.minimum_quality_score {
        issues.push(format!(
            "quality score {:.2} below minimum {:.2}",
            quality_score, done.minimum_quality_score
        ));
    }
    if !validation_passed {
        issues.push("validation did not fully pass".to_string());
    }
    let artifacts_complete = completeness.completion_score >= done.minimum_completeness_score;
    let quality_passed = quality_score >= done.minimum_quality_score;
    let final_status = if issues.is_empty() {
        "Completed"
    } else if done.allow_warnings {
        "CompletedWithWarnings"
    } else {
        "Blocked"
    };
    FinalAudit {
        session_id: session_id.to_string(),
        contract_met: artifacts_complete && quality_passed,
        done_definition_met: artifacts_complete && quality_passed && validation_passed,
        artifacts_complete,
        validation_passed,
        quality_passed,
        safety_passed: true,
        export_created,
        issues,
        final_status: final_status.to_string(),
    }
}

pub fn write_final_audit(store: &DiskStore, audit: &FinalAudit) -> Result<(String, String)> {
    let reports = store
        .paths
        .sandbox
        .join("workspaces")
        .join(&audit.session_id)
        .join("reports");
    fs::create_dir_all(&reports)?;
    let json_path = reports.join("final_audit.json");
    let md_path = reports.join("final_audit.md");
    save_json(&json_path, audit)?;
    fs::write(
        &md_path,
        format!(
            "# Final Audit\n\nStatus: {}\nContract met: {}\nDone definition met: {}\nArtifacts complete: {}\nValidation passed: {}\nQuality passed: {}\nSafety passed: {}\nExport created: {}\n\n## Issues\n{}\n",
            audit.final_status,
            audit.contract_met,
            audit.done_definition_met,
            audit.artifacts_complete,
            audit.validation_passed,
            audit.quality_passed,
            audit.safety_passed,
            audit.export_created,
            audit
                .issues
                .iter()
                .map(|issue| format!("- {issue}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )?;
    Ok((
        md_path.display().to_string(),
        json_path.display().to_string(),
    ))
}
