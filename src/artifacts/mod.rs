pub mod business_pack;
pub mod consistency;
pub mod cross_links;
pub mod dependency_graph;
pub mod documentation_pack;
pub mod generators;
pub mod markdown;
pub mod marketing_pack;
pub mod presentation;
pub mod project_pack;
pub mod release_kit;
pub mod report;
pub mod technical_pack;

pub use business_pack::*;
pub use consistency::*;
pub use cross_links::*;
pub use dependency_graph::*;
pub use documentation_pack::*;
pub use generators::*;
pub use markdown::*;
pub use marketing_pack::*;
pub use presentation::*;
pub use project_pack::*;
pub use release_kit::*;
pub use report::*;
pub use technical_pack::*;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactKind {
    PresentationMarkdown,
    SpeakerNotes,
    DesignGuide,
    StudyGuide,
    Quiz,
    Glossary,
    Checklist,
    ResearchBrief,
    Roadmap,
    RiskRegister,
    ArchitectureDocument,
    BudgetTable,
    LessonPlan,
    Faq,
    ComparisonTable,
    TestPlan,
    UserGuide,
    PitchDeck,
    LandingPageCopy,
    DemoScript,
    SocialPostSet,
    EmailAnnouncement,
    ExecutiveSummary,
    ProductSpec,
    TechnicalOverview,
    ArchitectureBrief,
    CompetitiveAnalysis,
    SwotAnalysis,
    MetricsPlan,
    ReleaseNotes,
    GitHubReleaseDraft,
    ContributorGuide,
    SecurityNotes,
    LaunchChecklist,
    FinalReport,
    Manifest,
    MarkdownDocument,
    JsonReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSummary {
    pub session_id: String,
    pub artifact_type: ArtifactKind,
    pub path: String,
    pub validation_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactIndex {
    #[serde(default)]
    pub artifacts: Vec<ArtifactSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub session_id: String,
    pub goal: String,
    pub artifacts: Vec<ArtifactSummary>,
    pub validation_score: f32,
    pub validation_passed: bool,
    pub repairs_performed: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactOverview {
    pub artifacts: Vec<ArtifactSummary>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactInspection {
    pub session_id: String,
    pub files: Vec<String>,
    pub manifest_path: Option<String>,
    pub validation_score: f32,
    pub validation_passed: bool,
    pub report_path: Option<String>,
}

pub fn artifact_index_path(store: &DiskStore) -> PathBuf {
    store.paths.indexes.join("artifact_index.json")
}

pub fn load_artifact_index(store: &DiskStore) -> Result<ArtifactIndex> {
    let path = artifact_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(ArtifactIndex::default())
    }
}

pub fn save_artifact_index(store: &DiskStore, index: &ArtifactIndex) -> Result<()> {
    save_json(&artifact_index_path(store), index)
}

pub fn artifact_session_dir(store: &DiskStore, session_id: &str) -> PathBuf {
    store.paths.sandbox.join("artifacts").join(session_id)
}

pub fn write_artifact(
    store: &DiskStore,
    session_id: &str,
    kind: ArtifactKind,
    file_name: &str,
    content: &str,
    validation_score: f32,
) -> Result<ArtifactSummary> {
    let dir = artifact_session_dir(store, session_id);
    fs::create_dir_all(&dir)?;
    let path = dir.join(file_name);
    fs::write(&path, content)?;
    let summary = ArtifactSummary {
        session_id: session_id.to_string(),
        artifact_type: kind,
        path: path.display().to_string(),
        validation_score,
        created_at: Utc::now(),
    };
    let mut index = load_artifact_index(store)?;
    index.artifacts.retain(|row| row.path != summary.path);
    index.artifacts.push(summary.clone());
    index
        .artifacts
        .sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if index.artifacts.len() > 512 {
        index.artifacts.truncate(512);
    }
    save_artifact_index(store, &index)?;
    Ok(summary)
}

pub fn save_manifest(store: &DiskStore, manifest: &ArtifactManifest) -> Result<ArtifactSummary> {
    let content = serde_json::to_string_pretty(manifest)?;
    write_artifact(
        store,
        &manifest.session_id,
        ArtifactKind::Manifest,
        "artifact_manifest.json",
        &content,
        manifest.validation_score,
    )
}

pub fn artifacts(store: &DiskStore) -> Result<ArtifactOverview> {
    let mut artifacts = load_artifact_index(store)?.artifacts;
    artifacts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if artifacts.len() > 25 {
        artifacts.truncate(25);
    }
    Ok(ArtifactOverview {
        count: load_artifact_index(store)?.artifacts.len(),
        artifacts,
    })
}

pub fn artifact_inspect(store: &DiskStore, selector: &str) -> Result<ArtifactInspection> {
    let index = load_artifact_index(store)?;
    let session_id = if selector.eq_ignore_ascii_case("latest") {
        index
            .artifacts
            .first()
            .map(|row| row.session_id.clone())
            .ok_or_else(|| anyhow::anyhow!("no artifacts found"))?
    } else {
        selector.to_string()
    };
    let mut files = Vec::new();
    let mut manifest_path = None;
    let mut report_path = None;
    let mut validation_score: f32 = 0.0;
    let mut validation_passed = false;
    for artifact in index
        .artifacts
        .into_iter()
        .filter(|row| row.session_id == session_id)
    {
        if artifact.artifact_type == ArtifactKind::Manifest {
            manifest_path = Some(artifact.path.clone());
            if let Ok(manifest) = load_json::<ArtifactManifest>(&PathBuf::from(&artifact.path)) {
                validation_score = validation_score.max(manifest.validation_score);
                validation_passed = manifest.validation_passed;
            }
        }
        if artifact.artifact_type == ArtifactKind::FinalReport {
            report_path = Some(artifact.path.clone());
        }
        files.push(artifact.path);
    }
    Ok(ArtifactInspection {
        session_id,
        files,
        manifest_path,
        validation_score,
        validation_passed,
        report_path,
    })
}

pub fn artifact_count(store: &DiskStore) -> usize {
    load_artifact_index(store)
        .map(|index| index.artifacts.len())
        .unwrap_or(0)
}

pub fn latest_artifact_session_id(store: &DiskStore) -> Option<String> {
    load_artifact_index(store)
        .ok()
        .and_then(|index| index.artifacts.first().map(|row| row.session_id.clone()))
}

pub fn artifact_report_name(prefix: &str) -> String {
    format!("{prefix}_{}.json", timestamp_slug())
}
