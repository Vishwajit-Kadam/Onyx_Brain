use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::{
    artifacts::workspace_artifacts_dir,
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub id: String,
    pub statement: String,
    pub reason: String,
    pub confidence: f32,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionLog {
    pub session_id: String,
    pub assumptions: Vec<Assumption>,
}

pub fn default_assumptions(session_id: &str, prompt: &str, slide_count: usize) -> AssumptionLog {
    let mut assumptions = vec![
        assumption(
            "Markdown output is acceptable because binary PPTX export is not supported.",
            "format limitation",
            0.95,
            "export-ready markdown will be produced",
        ),
        assumption(
            "Network research is disabled by default.",
            "safety policy",
            1.0,
            "citation placeholders may be used instead of researched citations",
        ),
    ];
    if prompt.to_lowercase().contains("student") {
        assumptions.push(assumption(
            "Audience is beginner students because the prompt mentioned students.",
            "prompt signal",
            0.9,
            "examples and explanations use classroom framing",
        ));
    }
    assumptions.push(assumption(
        &format!("Slide count is {slide_count} based on the prompt or default."),
        "prompt parsing",
        0.85,
        "deck structure follows this count",
    ));
    AssumptionLog {
        session_id: session_id.to_string(),
        assumptions,
    }
}

pub fn write_assumptions(store: &DiskStore, log: &AssumptionLog) -> Result<(String, String)> {
    let dir = workspace_artifacts_dir(store, &log.session_id);
    fs::create_dir_all(&dir)?;
    let json_path = dir.join("assumptions.json");
    let md_path = dir.join("assumptions.md");
    save_json(&json_path, log)?;
    fs::write(&md_path, render_assumptions(log))?;
    Ok((
        md_path.display().to_string(),
        json_path.display().to_string(),
    ))
}

fn render_assumptions(log: &AssumptionLog) -> String {
    let mut out = format!("# Assumptions\n\nSession: {}\n\n", log.session_id);
    for item in &log.assumptions {
        out.push_str(&format!(
            "- {} (confidence {:.2})\n  - Reason: {}\n  - Impact: {}\n",
            item.statement, item.confidence, item.reason, item.impact
        ));
    }
    out
}

fn assumption(statement: &str, reason: &str, confidence: f32, impact: &str) -> Assumption {
    Assumption {
        id: format!("assumption_{}", Uuid::new_v4()),
        statement: statement.to_string(),
        reason: reason.to_string(),
        confidence,
        impact: impact.to_string(),
    }
}
