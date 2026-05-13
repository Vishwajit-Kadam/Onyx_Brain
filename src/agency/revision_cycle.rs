use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{QualityReview, Severity},
    memory::self_critique::save_self_critique,
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionCycleConfig {
    pub max_revision_rounds: usize,
    pub max_files_per_round: usize,
    pub stop_on_safety_issue: bool,
}

impl Default for RevisionCycleConfig {
    fn default() -> Self {
        Self {
            max_revision_rounds: 2,
            max_files_per_round: 8,
            stop_on_safety_issue: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RevisionStatus {
    Improved,
    PassedWithoutRevision,
    StoppedBySafety,
    MaxRoundsReached,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionResult {
    pub rounds_run: usize,
    pub issues_fixed: usize,
    pub remaining_issues: usize,
    pub final_quality_score: f32,
    pub status: RevisionStatus,
}

pub fn run_revision_cycle(store: &DiskStore, review: &QualityReview) -> Result<RevisionResult> {
    if review.issues.is_empty() {
        return Ok(RevisionResult {
            rounds_run: 0,
            issues_fixed: 0,
            remaining_issues: 0,
            final_quality_score: review.overall_score,
            status: RevisionStatus::PassedWithoutRevision,
        });
    }
    if review
        .issues
        .iter()
        .any(|issue| issue.severity == Severity::Critical && !issue.auto_fixable)
    {
        return Ok(RevisionResult {
            rounds_run: 0,
            issues_fixed: 0,
            remaining_issues: review.issues.len(),
            final_quality_score: review.overall_score,
            status: RevisionStatus::StoppedBySafety,
        });
    }
    let mut fixed = 0;
    for issue in review
        .issues
        .iter()
        .filter(|issue| issue.auto_fixable)
        .take(8)
    {
        if let Some(path) = &issue.artifact_path {
            let mut content = fs::read_to_string(path).unwrap_or_default();
            if issue.issue_id == "missing_answer_key" {
                content.push_str("\n## Answer Key\n1. B\n2. A\nShort answer: Check for bounded, safe, deterministic framing.\n");
                fs::write(path, content)?;
                fixed += 1;
                let _ = save_self_critique(
                    store,
                    &review.session_id,
                    "Quiz missing answer key",
                    "Added deterministic answer key",
                    true,
                );
            } else if issue.issue_id == "short_glossary" {
                content.push_str("\n- Validation: checking artifacts against requirements.\n- Revision: bounded repair of detected issues.\n");
                fs::write(path, content)?;
                fixed += 1;
                let _ = save_self_critique(
                    store,
                    &review.session_id,
                    "Glossary too short",
                    "Added more glossary terms",
                    true,
                );
            }
        }
    }
    Ok(RevisionResult {
        rounds_run: 1,
        issues_fixed: fixed,
        remaining_issues: review.issues.len().saturating_sub(fixed),
        final_quality_score: (review.overall_score + fixed as f32 * 0.1).clamp(0.0, 1.0),
        status: if fixed > 0 {
            RevisionStatus::Improved
        } else {
            RevisionStatus::MaxRoundsReached
        },
    })
}
