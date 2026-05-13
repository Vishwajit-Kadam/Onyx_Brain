use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    agency::{Deliverable, DeliverableKind},
    artifacts::{ArtifactDependency, ArtifactType},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactNode {
    pub artifact_id: String,
    pub title: String,
    pub artifact_type: ArtifactType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactDependencyGraph {
    pub nodes: Vec<ArtifactNode>,
    pub edges: Vec<ArtifactEdge>,
}

impl ArtifactDependencyGraph {
    pub fn topological_order(&self) -> Vec<String> {
        let mut incoming: BTreeMap<String, usize> = self
            .nodes
            .iter()
            .map(|node| (node.artifact_id.clone(), 0))
            .collect();
        for edge in &self.edges {
            *incoming.entry(edge.to.clone()).or_default() += 1;
        }
        let mut ready = incoming
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let mut order = Vec::new();
        while let Some(id) = ready.pop() {
            order.push(id.clone());
            for edge in self.edges.iter().filter(|edge| edge.from == id) {
                if let Some(count) = incoming.get_mut(&edge.to) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        ready.push(edge.to.clone());
                    }
                }
            }
        }
        if order.len() < self.nodes.len() {
            for node in &self.nodes {
                if !order.contains(&node.artifact_id) {
                    order.push(node.artifact_id.clone());
                }
            }
        }
        order
    }

    pub fn detect_cycles(&self) -> Vec<String> {
        let mut incoming: BTreeMap<String, usize> = self
            .nodes
            .iter()
            .map(|node| (node.artifact_id.clone(), 0))
            .collect();
        for edge in &self.edges {
            *incoming.entry(edge.to.clone()).or_default() += 1;
        }
        let mut ready = incoming
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let mut visited = BTreeSet::new();
        while let Some(id) = ready.pop() {
            visited.insert(id.clone());
            for edge in self.edges.iter().filter(|edge| edge.from == id) {
                if let Some(count) = incoming.get_mut(&edge.to) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        ready.push(edge.to.clone());
                    }
                }
            }
        }
        if visited.len() < self.nodes.len() {
            self.nodes
                .iter()
                .filter(|node| !visited.contains(&node.artifact_id))
                .map(|node| format!("cycle includes {}", node.artifact_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn pack_dependencies(&self) -> Vec<ArtifactDependency> {
        self.edges
            .iter()
            .map(|edge| ArtifactDependency {
                from_artifact_id: edge.from.clone(),
                to_artifact_id: edge.to.clone(),
                dependency_type: edge.relation.clone(),
            })
            .collect()
    }
}

pub fn build_for_deliverables(deliverables: &[Deliverable]) -> ArtifactDependencyGraph {
    let nodes = deliverables
        .iter()
        .map(|deliverable| ArtifactNode {
            artifact_id: artifact_id_for_deliverable(deliverable),
            title: deliverable.title.clone(),
            artifact_type: artifact_type_for_kind(&deliverable.kind),
        })
        .collect::<Vec<_>>();
    let has = |id: &str| nodes.iter().any(|node| node.artifact_id == id);
    let mut edges = Vec::new();
    if has("presentation") && has("speaker_notes") {
        edges.push(edge(
            "presentation",
            "speaker_notes",
            "speaker notes depend on deck",
        ));
    }
    if has("study_guide") && has("quiz") {
        edges.push(edge("study_guide", "quiz", "quiz depends on study guide"));
    }
    if has("glossary") && has("study_guide") {
        edges.push(edge(
            "glossary",
            "study_guide",
            "glossary supports study guide",
        ));
    }
    for node in &nodes {
        if node.artifact_id != "final_report" && has("final_report") {
            edges.push(edge(
                &node.artifact_id,
                "final_report",
                "final report summarizes artifact",
            ));
        }
        if node.artifact_id != "manifest" && has("manifest") {
            edges.push(edge(
                &node.artifact_id,
                "manifest",
                "manifest includes artifact",
            ));
        }
    }
    ArtifactDependencyGraph { nodes, edges }
}

pub fn artifact_id_for_kind(kind: &DeliverableKind) -> String {
    match kind {
        DeliverableKind::PresentationMarkdown | DeliverableKind::SlideOutline => "presentation",
        DeliverableKind::StudyGuide => "study_guide",
        DeliverableKind::Quiz => "quiz",
        DeliverableKind::Glossary => "glossary",
        DeliverableKind::Checklist => "checklist",
        DeliverableKind::Roadmap => "roadmap",
        DeliverableKind::RiskRegister => "risk_register",
        DeliverableKind::ArchitectureDocument => "architecture",
        DeliverableKind::BudgetTable => "budget",
        DeliverableKind::FAQ => "faq",
        DeliverableKind::UserGuide => "user_guide",
        DeliverableKind::PitchDeck => "pitch_deck",
        DeliverableKind::LandingPageCopy => "landing_page_copy",
        DeliverableKind::DemoScript => "demo_script",
        DeliverableKind::SocialPostSet => "social_posts",
        DeliverableKind::EmailAnnouncement => "email_announcement",
        DeliverableKind::ExecutiveSummary => "executive_summary",
        DeliverableKind::ProductSpec => "product_spec",
        DeliverableKind::TechnicalOverview => "technical_overview",
        DeliverableKind::ArchitectureBrief => "architecture_brief",
        DeliverableKind::MetricsPlan => "metrics_plan",
        DeliverableKind::ReleaseNotes => "release_notes",
        DeliverableKind::GitHubReleaseDraft => "github_release_draft",
        DeliverableKind::LaunchChecklist => "launch_checklist",
        DeliverableKind::Report => "final_report",
        _ => "artifact",
    }
    .to_string()
}

fn artifact_id_for_deliverable(deliverable: &Deliverable) -> String {
    let hint = deliverable
        .path_hint
        .as_deref()
        .unwrap_or(&deliverable.title)
        .to_lowercase();
    if hint.contains("speaker") || hint.contains("notes") {
        return "speaker_notes".to_string();
    }
    if hint.contains("design") {
        return "design_guide".to_string();
    }
    if hint.contains("proposal") {
        return "proposal".to_string();
    }
    artifact_id_for_kind(&deliverable.kind)
}

pub fn artifact_type_for_kind(kind: &DeliverableKind) -> ArtifactType {
    match kind {
        DeliverableKind::PresentationMarkdown | DeliverableKind::SlideOutline => {
            ArtifactType::PresentationMarkdown
        }
        DeliverableKind::StudyGuide => ArtifactType::StudyGuide,
        DeliverableKind::Quiz => ArtifactType::Quiz,
        DeliverableKind::Glossary => ArtifactType::Glossary,
        DeliverableKind::Checklist => ArtifactType::Checklist,
        DeliverableKind::Roadmap => ArtifactType::Roadmap,
        DeliverableKind::RiskRegister => ArtifactType::RiskRegister,
        DeliverableKind::ArchitectureDocument => ArtifactType::ArchitectureDocument,
        DeliverableKind::BudgetTable => ArtifactType::BudgetTable,
        DeliverableKind::FAQ => ArtifactType::Faq,
        DeliverableKind::UserGuide => ArtifactType::UserGuide,
        DeliverableKind::PitchDeck => ArtifactType::PitchDeck,
        DeliverableKind::LandingPageCopy => ArtifactType::LandingPageCopy,
        DeliverableKind::DemoScript => ArtifactType::DemoScript,
        DeliverableKind::SocialPostSet => ArtifactType::SocialPostSet,
        DeliverableKind::EmailAnnouncement => ArtifactType::EmailAnnouncement,
        DeliverableKind::ExecutiveSummary => ArtifactType::ExecutiveSummary,
        DeliverableKind::ProductSpec => ArtifactType::ProductSpec,
        DeliverableKind::TechnicalOverview => ArtifactType::TechnicalOverview,
        DeliverableKind::ArchitectureBrief => ArtifactType::ArchitectureBrief,
        DeliverableKind::MetricsPlan => ArtifactType::MetricsPlan,
        DeliverableKind::ReleaseNotes => ArtifactType::ReleaseNotes,
        DeliverableKind::GitHubReleaseDraft => ArtifactType::GitHubReleaseDraft,
        DeliverableKind::LaunchChecklist => ArtifactType::LaunchChecklist,
        DeliverableKind::Report => ArtifactType::FinalReport,
        _ => ArtifactType::Unknown,
    }
}

fn edge(from: &str, to: &str, relation: &str) -> ArtifactEdge {
    ArtifactEdge {
        from: from.to_string(),
        to: to.to_string(),
        relation: relation.to_string(),
    }
}
