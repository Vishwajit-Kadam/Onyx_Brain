use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

use crate::{
    agency::Severity,
    artifacts::{artifact_pack_inspect, ArtifactPackInspection},
    storage::{save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    pub issue_id: String,
    pub severity: Severity,
    pub artifact_path: Option<String>,
    pub message: String,
    pub suggested_fix: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactQualityScore {
    pub completeness: f32,
    pub structure: f32,
    pub clarity: f32,
    pub consistency: f32,
    pub usefulness: f32,
    pub safety: f32,
    pub verification_honesty: f32,
    pub overall: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReview {
    pub review_id: String,
    pub session_id: String,
    pub artifact_id: Option<String>,
    pub overall_score: f32,
    pub completeness_score: f32,
    pub consistency_score: f32,
    pub clarity_score: f32,
    pub structure_score: f32,
    pub safety_score: f32,
    #[serde(default)]
    pub artifact_quality_score: ArtifactQualityScore,
    pub issues: Vec<QualityIssue>,
    pub recommendations: Vec<String>,
    pub report_path: String,
}

pub fn review_artifact_pack(store: &DiskStore, selector: &str) -> Result<QualityReview> {
    let inspection = artifact_pack_inspect(store, selector)?;
    review_pack_inspection(store, inspection)
}

pub fn review_pack_inspection(
    store: &DiskStore,
    inspection: ArtifactPackInspection,
) -> Result<QualityReview> {
    let session_id = session_from_manifest(&inspection.manifest_path);
    let mut issues = Vec::new();
    for row in &inspection.artifacts {
        let path = row.split('|').next().unwrap_or("").trim();
        if row.contains("quiz.md") {
            check_answer_key(path, &mut issues);
        }
        if row.contains("glossary.md") {
            check_glossary(path, &mut issues);
        }
        check_empty_headings(path, &mut issues);
        check_path_leak(path, &mut issues);
    }
    if !inspection
        .artifacts
        .iter()
        .any(|row| row.to_lowercase().contains("final_report"))
    {
        issues.push(issue(
            "missing_final_report",
            Severity::Error,
            None,
            "Final report missing",
            "create final report",
            true,
        ));
    }
    let penalty = issues
        .iter()
        .map(|issue| match issue.severity {
            Severity::Critical => 0.5,
            Severity::Error => 0.2,
            Severity::Warning => 0.1,
            Severity::Info => 0.03,
        })
        .sum::<f32>();
    let score = (1.0 - penalty).clamp(0.0, 1.0);
    let report_path = store
        .paths
        .logs
        .join(format!("quality_review_{}.json", timestamp_slug()));
    let safety_score = if issues.iter().any(|i| i.severity == Severity::Critical) {
        0.5
    } else {
        1.0
    };
    let artifact_quality_score = ArtifactQualityScore {
        completeness: score,
        structure: score,
        clarity: 0.95,
        consistency: 0.95,
        usefulness: 0.9,
        safety: safety_score,
        verification_honesty: verification_honesty(&inspection.artifacts),
        overall: ((score * 2.0) + 0.95 + 0.9 + safety_score) / 5.0,
    };
    let review = QualityReview {
        review_id: format!("review_{}", Uuid::new_v4()),
        session_id,
        artifact_id: None,
        overall_score: artifact_quality_score.overall,
        completeness_score: score,
        consistency_score: 0.95,
        clarity_score: 0.95,
        structure_score: score,
        safety_score,
        artifact_quality_score,
        issues,
        recommendations: vec!["review generated assumptions and limitations".to_string()],
        report_path: report_path.display().to_string(),
    };
    save_json(&report_path, &review)?;
    Ok(review)
}

fn verification_honesty(rows: &[String]) -> f32 {
    let mut saw_marketing_or_research = false;
    let mut saw_verification = false;
    for row in rows {
        let path = row.split('|').next().unwrap_or("").trim();
        let content = fs::read_to_string(path).unwrap_or_default().to_lowercase();
        saw_marketing_or_research |= content.contains("market")
            || content.contains("citation")
            || content.contains("verify")
            || content.contains("research");
        saw_verification |= content.contains("verification notes")
            || content.contains("citation placeholder")
            || content.contains("[citation needed]");
    }
    if !saw_marketing_or_research || saw_verification {
        1.0
    } else {
        0.75
    }
}

fn check_answer_key(path: &str, issues: &mut Vec<QualityIssue>) {
    if let Ok(content) = fs::read_to_string(path) {
        if !content.to_lowercase().contains("answer key") {
            issues.push(issue(
                "missing_answer_key",
                Severity::Error,
                Some(path.to_string()),
                "Quiz is missing an answer key",
                "add answer key",
                true,
            ));
        }
    }
}

fn check_glossary(path: &str, issues: &mut Vec<QualityIssue>) {
    if let Ok(content) = fs::read_to_string(path) {
        if content
            .lines()
            .filter(|line| line.trim_start().starts_with('-'))
            .count()
            < 5
        {
            issues.push(issue(
                "short_glossary",
                Severity::Warning,
                Some(path.to_string()),
                "Glossary has too few terms",
                "add glossary terms",
                true,
            ));
        }
    }
}

fn check_empty_headings(path: &str, issues: &mut Vec<QualityIssue>) {
    if let Ok(content) = fs::read_to_string(path) {
        if content.contains("\n## \n") || content.contains("\n# \n") {
            issues.push(issue(
                "empty_heading",
                Severity::Warning,
                Some(path.to_string()),
                "Document has an empty heading",
                "fill heading text",
                true,
            ));
        }
    }
}

fn check_path_leak(path: &str, issues: &mut Vec<QualityIssue>) {
    if path.ends_with(".json") {
        return;
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    if content.contains("C:\\Users\\") || content.contains("/Users/") {
        issues.push(issue(
            "personal_path_leak",
            Severity::Critical,
            Some(path.to_string()),
            "Personal absolute path leaked into artifact",
            "replace with relative path",
            true,
        ));
    }
}

fn issue(
    id: &str,
    severity: Severity,
    path: Option<String>,
    message: &str,
    fix: &str,
    auto: bool,
) -> QualityIssue {
    QualityIssue {
        issue_id: id.to_string(),
        severity,
        artifact_path: path,
        message: message.to_string(),
        suggested_fix: fix.to_string(),
        auto_fixable: auto,
    }
}

fn session_from_manifest(path: &str) -> String {
    PathBuf::from(path)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}
