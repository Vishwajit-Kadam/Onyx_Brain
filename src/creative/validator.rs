use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::creative::CreativeProject;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreativeValidationReport {
    pub passed: bool,
    pub issues: Vec<String>,
    pub score: f32,
}

pub fn validate_creative_workspace(
    project: &CreativeProject,
    artifacts: &Path,
) -> Result<CreativeValidationReport> {
    let required = [
        "creative_brief.md",
        "story_outline.md",
        "scene_breakdown.md",
        "shot_list.md",
        "timeline_plan.md",
        "edit_decision_list.md",
        "sound_design_plan.md",
        "vfx_plan.md",
        "color_grade_plan.md",
        "subtitle_script.md",
        "review_checklist.md",
    ];
    let mut issues = Vec::new();
    for file in required {
        if !artifacts.join(file).exists() {
            issues.push(format!("missing {file}"));
        }
    }
    if project.genre.as_deref().unwrap_or("").contains("sci-fi")
        && !artifacts.join("vfx_plan.md").exists()
    {
        issues.push("sci-fi package requires VFX notes".to_string());
    }
    let passed = issues.is_empty();
    Ok(CreativeValidationReport {
        passed,
        score: if passed { 0.95 } else { 0.55 },
        issues,
    })
}
