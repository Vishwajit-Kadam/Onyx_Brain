use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};
use uuid::Uuid;

use crate::{
    artifacts::{ArtifactKind, ArtifactSummary},
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactType {
    PresentationMarkdown,
    SpeakerNotes,
    DesignGuide,
    StudyGuide,
    Quiz,
    Glossary,
    Checklist,
    ResearchBrief,
    CodeProject,
    Roadmap,
    RiskRegister,
    ArchitectureDocument,
    BudgetTable,
    Faq,
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
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactStatus {
    Planned,
    Created,
    Validated,
    Repaired,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub artifact_id: String,
    pub artifact_type: ArtifactType,
    pub title: String,
    pub relative_path: String,
    pub status: ArtifactStatus,
    pub required: bool,
    pub validation_score: f32,
    pub word_count: Option<usize>,
    pub section_count: Option<usize>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDependency {
    pub from_artifact_id: String,
    pub to_artifact_id: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPack {
    pub pack_id: String,
    pub session_id: String,
    pub title: String,
    pub goal_id: Option<String>,
    pub artifacts: Vec<ArtifactRecord>,
    pub dependencies: Vec<ArtifactDependency>,
    pub manifest_path: String,
    pub validation_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPackSummary {
    pub pack_id: String,
    pub session_id: String,
    pub title: String,
    pub manifest_path: String,
    pub validation_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactPackIndex {
    #[serde(default)]
    pub packs: Vec<ArtifactPackSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactPackOverview {
    pub packs: Vec<ArtifactPackSummary>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactPackInspection {
    pub pack_title: String,
    pub artifacts: Vec<String>,
    pub dependency_graph: Vec<String>,
    pub validation_scores: Vec<String>,
    pub failed_or_missing_artifacts: Vec<String>,
    pub manifest_path: String,
}

pub fn artifact_pack_index_path(store: &DiskStore) -> PathBuf {
    store.paths.indexes.join("artifact_pack_index.json")
}

pub fn load_artifact_pack_index(store: &DiskStore) -> Result<ArtifactPackIndex> {
    let path = artifact_pack_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(ArtifactPackIndex::default())
    }
}

pub fn workspace_artifacts_dir(store: &DiskStore, session_id: &str) -> PathBuf {
    store
        .paths
        .sandbox
        .join("workspaces")
        .join(session_id)
        .join("artifacts")
}

pub fn artifact_type_from_kind(kind: &ArtifactKind) -> ArtifactType {
    match kind {
        ArtifactKind::PresentationMarkdown => ArtifactType::PresentationMarkdown,
        ArtifactKind::SpeakerNotes => ArtifactType::SpeakerNotes,
        ArtifactKind::DesignGuide => ArtifactType::DesignGuide,
        ArtifactKind::StudyGuide => ArtifactType::StudyGuide,
        ArtifactKind::Quiz => ArtifactType::Quiz,
        ArtifactKind::Glossary => ArtifactType::Glossary,
        ArtifactKind::Checklist => ArtifactType::Checklist,
        ArtifactKind::ResearchBrief => ArtifactType::ResearchBrief,
        ArtifactKind::Roadmap => ArtifactType::Roadmap,
        ArtifactKind::RiskRegister => ArtifactType::RiskRegister,
        ArtifactKind::ArchitectureDocument => ArtifactType::ArchitectureDocument,
        ArtifactKind::BudgetTable => ArtifactType::BudgetTable,
        ArtifactKind::Faq => ArtifactType::Faq,
        ArtifactKind::UserGuide => ArtifactType::UserGuide,
        ArtifactKind::PitchDeck => ArtifactType::PitchDeck,
        ArtifactKind::LandingPageCopy => ArtifactType::LandingPageCopy,
        ArtifactKind::DemoScript => ArtifactType::DemoScript,
        ArtifactKind::SocialPostSet => ArtifactType::SocialPostSet,
        ArtifactKind::EmailAnnouncement => ArtifactType::EmailAnnouncement,
        ArtifactKind::ExecutiveSummary => ArtifactType::ExecutiveSummary,
        ArtifactKind::ProductSpec => ArtifactType::ProductSpec,
        ArtifactKind::TechnicalOverview => ArtifactType::TechnicalOverview,
        ArtifactKind::ArchitectureBrief => ArtifactType::ArchitectureBrief,
        ArtifactKind::CompetitiveAnalysis => ArtifactType::CompetitiveAnalysis,
        ArtifactKind::SwotAnalysis => ArtifactType::SwotAnalysis,
        ArtifactKind::MetricsPlan => ArtifactType::MetricsPlan,
        ArtifactKind::ReleaseNotes => ArtifactType::ReleaseNotes,
        ArtifactKind::GitHubReleaseDraft => ArtifactType::GitHubReleaseDraft,
        ArtifactKind::ContributorGuide => ArtifactType::ContributorGuide,
        ArtifactKind::SecurityNotes => ArtifactType::SecurityNotes,
        ArtifactKind::LaunchChecklist => ArtifactType::LaunchChecklist,
        ArtifactKind::FinalReport => ArtifactType::FinalReport,
        ArtifactKind::Manifest => ArtifactType::Manifest,
        _ => ArtifactType::Unknown,
    }
}

pub fn build_artifact_pack(
    store: &DiskStore,
    session_id: &str,
    title: &str,
    goal_id: Option<String>,
    summaries: &[ArtifactSummary],
    dependencies: Vec<ArtifactDependency>,
    validation_score: f32,
) -> Result<ArtifactPack> {
    let root = workspace_artifacts_dir(store, session_id);
    fs::create_dir_all(&root)?;
    let artifacts = summaries
        .iter()
        .map(|summary| {
            let source_path = PathBuf::from(&summary.path);
            let file_name = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("artifact.md");
            let workspace_path = root.join(file_name);
            if source_path != workspace_path {
                let _ = fs::copy(&source_path, &workspace_path);
            }
            let stored_path = workspace_path.display().to_string();
            let content = fs::read_to_string(&stored_path).unwrap_or_default();
            ArtifactRecord {
                artifact_id: format!("artifact_{}", Uuid::new_v4()),
                artifact_type: artifact_type_from_kind(&summary.artifact_type),
                title: file_name.to_string(),
                relative_path: stored_path,
                status: if summary.validation_score >= 0.8 {
                    ArtifactStatus::Validated
                } else {
                    ArtifactStatus::Created
                },
                required: true,
                validation_score: summary.validation_score,
                word_count: Some(content.split_whitespace().count()),
                section_count: Some(content.lines().filter(|line| line.starts_with('#')).count()),
                metadata: serde_json::json!({}),
            }
        })
        .collect::<Vec<_>>();
    let manifest_path = root.join("artifact_pack.json");
    let pack = ArtifactPack {
        pack_id: format!("pack_{}_{}", timestamp_slug(), Uuid::new_v4()),
        session_id: session_id.to_string(),
        title: title.to_string(),
        goal_id,
        artifacts,
        dependencies,
        manifest_path: manifest_path.display().to_string(),
        validation_score,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    save_json(&manifest_path, &pack)?;
    let mut index = load_artifact_pack_index(store)?;
    index.packs.retain(|row| row.pack_id != pack.pack_id);
    index.packs.push(ArtifactPackSummary {
        pack_id: pack.pack_id.clone(),
        session_id: pack.session_id.clone(),
        title: pack.title.clone(),
        manifest_path: pack.manifest_path.clone(),
        validation_score: pack.validation_score,
        created_at: pack.created_at,
    });
    index.packs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if index.packs.len() > 256 {
        index.packs.truncate(256);
    }
    save_json(&artifact_pack_index_path(store), &index)?;
    Ok(pack)
}

pub fn artifact_packs(store: &DiskStore) -> Result<ArtifactPackOverview> {
    let mut packs = load_artifact_pack_index(store)?.packs;
    packs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if packs.len() > 25 {
        packs.truncate(25);
    }
    Ok(ArtifactPackOverview {
        count: load_artifact_pack_index(store)?.packs.len(),
        packs,
    })
}

pub fn artifact_pack_inspect(store: &DiskStore, selector: &str) -> Result<ArtifactPackInspection> {
    let index = load_artifact_pack_index(store)?;
    let summary = if selector.eq_ignore_ascii_case("latest") {
        index
            .packs
            .first()
            .ok_or_else(|| anyhow::anyhow!("no artifact packs found"))?
            .clone()
    } else {
        index
            .packs
            .into_iter()
            .find(|row| row.pack_id == selector || row.session_id == selector)
            .ok_or_else(|| anyhow::anyhow!("artifact pack not found"))?
    };
    let pack: ArtifactPack = load_json(&PathBuf::from(&summary.manifest_path))?;
    Ok(ArtifactPackInspection {
        pack_title: pack.title,
        artifacts: pack
            .artifacts
            .iter()
            .map(|artifact| {
                format!(
                    "{} | {:?} | {:?}",
                    artifact.relative_path, artifact.artifact_type, artifact.status
                )
            })
            .collect(),
        dependency_graph: pack
            .dependencies
            .iter()
            .map(|edge| {
                format!(
                    "{} -> {} ({})",
                    edge.from_artifact_id, edge.to_artifact_id, edge.dependency_type
                )
            })
            .collect(),
        validation_scores: pack
            .artifacts
            .iter()
            .map(|artifact| format!("{} {:.2}", artifact.title, artifact.validation_score))
            .collect(),
        failed_or_missing_artifacts: pack
            .artifacts
            .iter()
            .filter(|artifact| {
                matches!(
                    artifact.status,
                    ArtifactStatus::Failed | ArtifactStatus::Skipped
                )
            })
            .map(|artifact| artifact.title.clone())
            .collect(),
        manifest_path: pack.manifest_path,
    })
}
