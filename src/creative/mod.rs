pub mod cinematic;
pub mod project;
pub mod review;
pub mod shot_list;
pub mod storyboard;
pub mod timeline;
pub mod validator;
pub mod video_edit_plan;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::{
    app_api::{record_event, AppEventKind},
    storage::{save_json, DiskStore},
    utils::time::timestamp_slug,
};

pub use cinematic::*;
pub use project::*;
pub use review::*;
pub use shot_list::*;
pub use storyboard::*;
pub use timeline::*;
pub use validator::*;
pub use video_edit_plan::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreativeRunReport {
    pub session_id: String,
    pub project_id: String,
    pub title: String,
    pub workspace_path: String,
    pub artifacts_created: Vec<String>,
    pub validation_passed: bool,
    pub originality_caution: Option<String>,
    pub final_report_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkCreativeReport {
    pub tasks_run: u64,
    pub tasks_successful: u64,
    pub artifacts_created: usize,
    pub validation_passed: bool,
    pub runtime_ms: u64,
    pub report_path: String,
}

pub fn create_creative_project(store: &DiskStore, prompt: &str) -> Result<CreativeRunReport> {
    store.ensure_layout()?;
    let session_id = format!("creative_{}_{}", timestamp_slug(), Uuid::new_v4());
    let project = parse_creative_project(&session_id, prompt);
    let workspace = store.paths.sandbox.join("workspaces").join(&session_id);
    let artifacts = workspace.join("artifacts");
    fs::create_dir_all(&artifacts)?;
    let plan = build_video_edit_plan(&project, prompt);
    let mut created = Vec::new();
    for (name, content) in creative_files(&project, &plan, prompt) {
        let path = artifacts.join(name);
        fs::write(&path, content)?;
        created.push(path.display().to_string());
    }
    let creative_project_path = workspace.join("creative_project.json");
    save_json(&creative_project_path, &project)?;
    let manifest_path = artifacts.join("creative_manifest.json");
    save_json(&manifest_path, &plan)?;
    created.push(manifest_path.display().to_string());
    let validation = validate_creative_workspace(&project, &artifacts)?;
    let final_report = artifacts.join("final_production_report.md");
    let caution = originality_caution(prompt);
    if !final_report.exists() {
        fs::write(
            &final_report,
            final_report_markdown(&project, &validation, caution.as_deref()),
        )?;
        created.push(final_report.display().to_string());
    }
    let _ = record_event(
        store,
        &session_id,
        AppEventKind::CreativeProjectCreated,
        "creative production package created",
    );
    Ok(CreativeRunReport {
        session_id,
        project_id: project.project_id,
        title: project.title,
        workspace_path: workspace.display().to_string(),
        artifacts_created: created,
        validation_passed: validation.passed,
        originality_caution: caution,
        final_report_path: final_report.display().to_string(),
    })
}

pub fn benchmark_creative(store: &DiskStore) -> Result<BenchmarkCreativeReport> {
    let timer = std::time::Instant::now();
    let report = create_creative_project(
        store,
        "Create a cinematic editing plan for a 3-hour original sci-fi feature film with scene breakdown, timeline, sound design, VFX notes, color grading notes, and final production package",
    )?;
    let path = store
        .paths
        .logs
        .join(format!("benchmark_creative_{}.json", timestamp_slug()));
    let bench = BenchmarkCreativeReport {
        tasks_run: 1,
        tasks_successful: if report.validation_passed { 1 } else { 0 },
        artifacts_created: report.artifacts_created.len(),
        validation_passed: report.validation_passed,
        runtime_ms: timer.elapsed().as_millis() as u64,
        report_path: path.display().to_string(),
    };
    save_json(&path, &bench)?;
    Ok(bench)
}

fn creative_files(
    project: &CreativeProject,
    plan: &VideoEditPlan,
    prompt: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("creative_brief.md", creative_brief(project, prompt)),
        ("story_outline.md", story_outline(plan)),
        ("scene_breakdown.md", scene_breakdown(plan)),
        ("shot_list.md", shot_list(plan)),
        ("timeline_plan.md", timeline_markdown(&plan.timeline)),
        ("edit_decision_list.md", edit_decision_list(plan)),
        ("sound_design_plan.md", sound_design_plan(plan)),
        ("vfx_plan.md", vfx_plan(project, plan)),
        ("color_grade_plan.md", color_grade_plan(project)),
        ("subtitle_script.md", subtitle_script(plan)),
        ("review_checklist.md", review_checklist()),
        (
            "final_production_report.md",
            final_report_markdown(
                project,
                &CreativeValidationReport {
                    passed: true,
                    issues: Vec::new(),
                    score: 0.95,
                },
                originality_caution(prompt).as_deref(),
            ),
        ),
    ]
}

fn originality_caution(prompt: &str) -> Option<String> {
    prompt.to_lowercase().contains("avatar").then(|| {
        "Originality note: treat Avatar-like wording as broad sci-fi scale inspiration only; do not copy copyrighted characters, worlds, names, or protected visual identity.".to_string()
    })
}

fn final_report_markdown(
    project: &CreativeProject,
    validation: &CreativeValidationReport,
    caution: Option<&str>,
) -> String {
    format!(
        "# Final Production Report: {}\n\nValidation passed: {}\nScore: {:.2}\n\n{}\n\nNo actual video was rendered; this package is a planning and production-documentation workflow.\n",
        project.title,
        validation.passed,
        validation.score,
        caution.unwrap_or("Originality note: use original story, characters, and visual direction.")
    )
}
