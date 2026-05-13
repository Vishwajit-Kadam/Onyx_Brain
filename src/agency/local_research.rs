use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::GoalUnderstanding,
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalResearchPack {
    pub session_id: String,
    pub sources_considered: Vec<String>,
    pub findings: Vec<String>,
    pub uncertainty_notes: Vec<String>,
    pub citation_placeholders: Vec<String>,
    pub verification_needed: Vec<String>,
}

pub fn create_local_research_pack(
    store: &DiskStore,
    session_id: &str,
    understanding: &GoalUnderstanding,
) -> Result<LocalResearchPack> {
    let mut sources = vec!["user prompt".to_string()];
    for path in ["README.md", "docs", "src", "tests"] {
        if store.paths.root.join(path).exists() {
            sources.push(path.to_string());
        }
    }
    let pack = LocalResearchPack {
        session_id: session_id.to_string(),
        sources_considered: sources,
        findings: vec![
            format!("Goal type inferred as {:?}.", understanding.goal_type),
            "Generation is deterministic and local-only.".to_string(),
        ],
        uncertainty_notes: vec![
            "No web lookup was performed.".to_string(),
            "Factual claims should be externally verified before publication.".to_string(),
        ],
        citation_placeholders: vec!["[citation needed]".to_string()],
        verification_needed: vec![
            "Confirm factual/statistical claims outside Onyx Brain.".to_string()
        ],
    };
    let dir = store
        .paths
        .sandbox
        .join("workspaces")
        .join(session_id)
        .join("reports");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join("local_research_brief.json"), &pack)?;
    fs::write(&dir.join("local_research_brief.md"), render_research(&pack))?;
    Ok(pack)
}

fn render_research(pack: &LocalResearchPack) -> String {
    format!(
        "# Local Research Brief\n\n## Sources Considered\n{}\n\n## Findings\n{}\n\n## Uncertainty Notes\n{}\n\n## Citation Placeholders\n{}\n\n## Verification Needed\n{}\n",
        bullets(&pack.sources_considered),
        bullets(&pack.findings),
        bullets(&pack.uncertainty_notes),
        bullets(&pack.citation_placeholders),
        bullets(&pack.verification_needed)
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
