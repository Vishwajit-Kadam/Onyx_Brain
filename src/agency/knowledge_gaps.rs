use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    storage::{save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub question: String,
    pub impact: String,
    pub assumed_answer: Option<String>,
    pub requires_user_input: bool,
    pub workaround: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeGapReport {
    pub session_id: String,
    pub gaps: Vec<KnowledgeGap>,
    pub report_id: String,
}

pub fn create_knowledge_gap_report(
    store: &DiskStore,
    session_id: &str,
    prompt: &str,
) -> Result<KnowledgeGapReport> {
    let lower = prompt.to_lowercase();
    let mut gaps = Vec::new();
    if lower.contains("research") || lower.contains("citation") || lower.contains("market") {
        gaps.push(KnowledgeGap {
            question: "Which external facts or sources should be cited?".to_string(),
            impact:
                "Generated artifacts can include placeholders but cannot verify claims locally."
                    .to_string(),
            assumed_answer: Some("Use citation placeholders and verification notes.".to_string()),
            requires_user_input: false,
            workaround: "Verify externally before publication.".to_string(),
        });
    }
    if lower.contains("ppt") || lower.contains("powerpoint") {
        gaps.push(KnowledgeGap {
            question: "Should output be a binary .pptx?".to_string(),
            impact: "This version creates export-ready markdown decks only.".to_string(),
            assumed_answer: Some(
                "Create markdown deck, speaker notes, and design guide.".to_string(),
            ),
            requires_user_input: false,
            workaround: "Import markdown into a presentation tool manually.".to_string(),
        });
    }
    let report = KnowledgeGapReport {
        session_id: session_id.to_string(),
        gaps,
        report_id: format!("knowledge_gaps_{}", timestamp_slug()),
    };
    let root = store.paths.sandbox.join("workspaces").join(session_id);
    fs::create_dir_all(&root)?;
    save_json(&root.join("knowledge_gaps.json"), &report)?;
    fs::write(root.join("knowledge_gaps.md"), render_gaps(&report))?;
    Ok(report)
}

fn render_gaps(report: &KnowledgeGapReport) -> String {
    let body = if report.gaps.is_empty() {
        "- No blocking knowledge gaps detected.".to_string()
    } else {
        report
            .gaps
            .iter()
            .map(|gap| {
                format!(
                    "- Q: {}\n  - Impact: {}\n  - Assumed answer: {}\n  - Workaround: {}",
                    gap.question,
                    gap.impact,
                    gap.assumed_answer
                        .clone()
                        .unwrap_or_else(|| "none".to_string()),
                    gap.workaround
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!("# Knowledge Gaps\n\n{body}\n")
}
