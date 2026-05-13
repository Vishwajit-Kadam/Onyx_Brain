use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    agency::ValidationResult,
    artifacts::{
        render_design_guide, render_presentation_markdown, render_speaker_notes, write_artifact,
        ArtifactKind, PresentationArtifact,
    },
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairLoopConfig {
    pub max_repair_rounds: usize,
    pub max_repairs_per_file: usize,
    pub stop_on_critical_safety_issue: bool,
}

impl Default for RepairLoopConfig {
    fn default() -> Self {
        Self {
            max_repair_rounds: 2,
            max_repairs_per_file: 2,
            stop_on_critical_safety_issue: true,
        }
    }
}

pub fn repair_presentation_artifacts(
    store: &DiskStore,
    session_id: &str,
    presentation: &PresentationArtifact,
    validation: &ValidationResult,
) -> Result<usize> {
    let mut repairs = 0;
    for issue in validation.issues.iter().filter(|issue| issue.auto_fixable) {
        if issue.issue_id.contains("presentation.md") || issue.issue_id == "missing_presentation.md"
        {
            write_artifact(
                store,
                session_id,
                ArtifactKind::PresentationMarkdown,
                "presentation.md",
                &render_presentation_markdown(presentation),
                0.8,
            )?;
            repairs += 1;
        } else if issue.issue_id.contains("speaker_notes") {
            write_artifact(
                store,
                session_id,
                ArtifactKind::SpeakerNotes,
                "speaker_notes.md",
                &render_speaker_notes(presentation),
                0.8,
            )?;
            repairs += 1;
        } else if issue.issue_id.contains("design_guide") {
            write_artifact(
                store,
                session_id,
                ArtifactKind::DesignGuide,
                "design_guide.md",
                &render_design_guide(presentation),
                0.8,
            )?;
            repairs += 1;
        }
    }
    Ok(repairs)
}
