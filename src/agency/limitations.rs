use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::{
    agency::Severity,
    artifacts::workspace_artifacts_dir,
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limitation {
    pub id: String,
    pub description: String,
    pub reason: String,
    pub workaround: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitationLog {
    pub session_id: String,
    pub limitations: Vec<Limitation>,
}

pub fn default_limitations(session_id: &str) -> LimitationLog {
    LimitationLog {
        session_id: session_id.to_string(),
        limitations: vec![
            limitation(
                "Binary PPTX export is not supported in v0.0.2.",
                "artifact generator is markdown-only",
                "export markdown with an external presentation tool",
                Severity::Info,
            ),
            limitation(
                "Network research is disabled by default.",
                "safety policy",
                "add citations manually or enable a future reviewed adapter",
                Severity::Warning,
            ),
            limitation(
                "LLM generation is not included by default.",
                "deterministic runtime design",
                "edit generated markdown manually for nuance",
                Severity::Info,
            ),
            limitation(
                "Complex design rendering is represented as markdown guidance.",
                "no binary renderer in this release",
                "use the design guide in a slide tool",
                Severity::Info,
            ),
        ],
    }
}

pub fn write_limitations(store: &DiskStore, log: &LimitationLog) -> Result<(String, String)> {
    let dir = workspace_artifacts_dir(store, &log.session_id);
    fs::create_dir_all(&dir)?;
    let json_path = dir.join("limitations.json");
    let md_path = dir.join("limitations.md");
    save_json(&json_path, log)?;
    fs::write(&md_path, render_limitations(log))?;
    Ok((
        md_path.display().to_string(),
        json_path.display().to_string(),
    ))
}

fn render_limitations(log: &LimitationLog) -> String {
    let mut out = format!("# Limitations\n\nSession: {}\n\n", log.session_id);
    for item in &log.limitations {
        out.push_str(&format!(
            "- {:?}: {}\n  - Reason: {}\n  - Workaround: {}\n",
            item.severity, item.description, item.reason, item.workaround
        ));
    }
    out
}

fn limitation(description: &str, reason: &str, workaround: &str, severity: Severity) -> Limitation {
    Limitation {
        id: format!("limitation_{}", Uuid::new_v4()),
        description: description.to_string(),
        reason: reason.to_string(),
        workaround: workaround.to_string(),
        severity,
    }
}
