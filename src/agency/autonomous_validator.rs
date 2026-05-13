use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::artifacts::PresentationArtifact;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_id: String,
    pub severity: Severity,
    pub message: String,
    pub affected_file: Option<String>,
    pub suggested_fix: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationResult {
    pub passed: bool,
    pub score: f32,
    pub issues: Vec<ValidationIssue>,
    pub repaired: bool,
    pub repair_attempts: usize,
}

pub fn validate_presentation_artifacts(
    dir: &Path,
    expected_slides: usize,
    presentation: Option<&PresentationArtifact>,
) -> ValidationResult {
    let mut issues = Vec::new();
    let required = [
        ("presentation.md", "presentation deck exists"),
        ("speaker_notes.md", "speaker notes exist"),
        ("design_guide.md", "design guide exists"),
        ("final_report.md", "final report exists"),
        ("artifact_manifest.json", "artifact manifest exists"),
    ];
    for (file, label) in required {
        let path = dir.join(file);
        if !path.exists() {
            issues.push(issue(
                &format!("missing_{file}"),
                Severity::Error,
                &format!("Missing {label}"),
                Some(path.display().to_string()),
                &format!("create {file}"),
                true,
            ));
        } else if fs::read_to_string(&path)
            .map(|content| content.trim().is_empty())
            .unwrap_or(true)
        {
            issues.push(issue(
                &format!("empty_{file}"),
                Severity::Error,
                &format!("Empty {label}"),
                Some(path.display().to_string()),
                &format!("write content to {file}"),
                true,
            ));
        }
    }
    if let Some(presentation) = presentation {
        if presentation.slides.len() != expected_slides {
            issues.push(issue(
                "wrong_slide_count",
                Severity::Error,
                "Slide count does not match request",
                Some(dir.join("presentation.md").display().to_string()),
                "add or remove slides to match requested count",
                true,
            ));
        }
        for slide in &presentation.slides {
            if slide.title.trim().is_empty()
                || slide.bullets.is_empty()
                || slide.speaker_notes.trim().is_empty()
            {
                issues.push(issue(
                    &format!("incomplete_slide_{}", slide.number),
                    Severity::Error,
                    "Slide is missing title, bullets, or speaker notes",
                    Some(dir.join("presentation.md").display().to_string()),
                    "fill missing slide fields",
                    true,
                ));
            }
        }
    }
    let penalty = issues
        .iter()
        .map(|issue| match issue.severity {
            Severity::Critical => 0.5,
            Severity::Error => 0.2,
            Severity::Warning => 0.1,
            Severity::Info => 0.02,
        })
        .sum::<f32>();
    let score = (1.0 - penalty).clamp(0.0, 1.0);
    ValidationResult {
        passed: issues.is_empty(),
        score,
        issues,
        repaired: false,
        repair_attempts: 0,
    }
}

fn issue(
    id: &str,
    severity: Severity,
    message: &str,
    affected_file: Option<String>,
    suggested_fix: &str,
    auto_fixable: bool,
) -> ValidationIssue {
    ValidationIssue {
        issue_id: id.to_string(),
        severity,
        message: message.to_string(),
        affected_file,
        suggested_fix: suggested_fix.to_string(),
        auto_fixable,
    }
}
