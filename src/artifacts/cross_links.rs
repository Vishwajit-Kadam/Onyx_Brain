use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{
    artifacts::ArtifactPack,
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossLinkReport {
    pub links_added: usize,
    pub missing_targets: Vec<String>,
    pub score: f32,
}

pub fn add_cross_links(store: &DiskStore, pack: &ArtifactPack) -> Result<CrossLinkReport> {
    let mut by_name = BTreeMap::new();
    for artifact in &pack.artifacts {
        let path = PathBuf::from(&artifact.relative_path);
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            by_name.insert(name.to_string(), path);
        }
    }
    let mut links_added = 0;
    for (name, path) in &by_name {
        if !path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "md")
        {
            continue;
        }
        let mut related = Vec::new();
        if name.contains("pitch") || name.contains("presentation") || name.contains("slide") {
            related.push("speaker_notes.md");
        }
        if name.contains("technical") {
            related.push("architecture_brief.md");
        }
        if name.contains("launch_checklist") {
            related.push("demo_script.md");
            related.push("release_notes.md");
        }
        if name.contains("product_spec") {
            related.push("roadmap.md");
            related.push("metrics_plan.md");
        }
        if name.contains("final_report") {
            related.extend(by_name.keys().map(String::as_str));
        }
        related.sort();
        related.dedup();
        let existing = fs::read_to_string(path).unwrap_or_default();
        if existing.contains("## Related Artifacts") || related.is_empty() {
            continue;
        }
        let links = related
            .into_iter()
            .filter(|target| *target != name && by_name.contains_key(*target))
            .map(|target| format!("- [{target}](./{target})"))
            .collect::<Vec<_>>();
        if links.is_empty() {
            continue;
        }
        fs::write(
            path,
            format!("{existing}\n\n## Related Artifacts\n{}\n", links.join("\n")),
        )?;
        links_added += links.len();
    }
    let report = CrossLinkReport {
        links_added,
        missing_targets: Vec::new(),
        score: if links_added > 0 { 1.0 } else { 0.85 },
    };
    let reports = store
        .paths
        .sandbox
        .join("workspaces")
        .join(&pack.session_id)
        .join("reports");
    fs::create_dir_all(&reports)?;
    save_json(&reports.join("cross_link_report.json"), &report)?;
    fs::write(
        reports.join("cross_link_report.md"),
        format!(
            "# Cross-Link Report\n\nLinks added: {}\nScore: {:.2}\n",
            report.links_added, report.score
        ),
    )?;
    Ok(report)
}
