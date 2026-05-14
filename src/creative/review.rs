//! Creative review — structured checklist validation and quality scoring
//! for creative production deliverables.
//!
//! Provides a checklist-based review system that can be scored and persisted,
//! rather than just returning a static string.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::storage::{save_json, DiskStore};

/// A single item in the review checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub label: String,
    pub required: bool,
    pub passed: bool,
    pub notes: Option<String>,
}

impl ChecklistItem {
    pub fn new(label: impl Into<String>, required: bool) -> Self {
        Self {
            label: label.into(),
            required,
            passed: false,
            notes: None,
        }
    }

    pub fn pass(&mut self) {
        self.passed = true;
    }

    pub fn fail(&mut self, notes: impl Into<String>) {
        self.passed = false;
        self.notes = Some(notes.into());
    }
}

/// A full review of a creative production.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeReview {
    pub review_id: String,
    pub project_title: String,
    pub items: Vec<ChecklistItem>,
    pub overall_score: f32,
    pub passed: bool,
    pub reviewer_notes: String,
    pub created_at: DateTime<Utc>,
}

impl CreativeReview {
    /// Build a default review for a creative production.
    pub fn new(project_title: impl Into<String>) -> Self {
        Self {
            review_id: format!("review_{}", Uuid::new_v4()),
            project_title: project_title.into(),
            items: default_checklist(),
            overall_score: 0.0,
            passed: false,
            reviewer_notes: String::new(),
            created_at: Utc::now(),
        }
    }

    /// Run the review: compute score and pass/fail status.
    pub fn evaluate(&mut self) {
        if self.items.is_empty() {
            self.overall_score = 0.0;
            self.passed = false;
            return;
        }
        let total = self.items.len() as f32;
        let passed = self.items.iter().filter(|i| i.passed).count() as f32;
        self.overall_score = passed / total;
        // Fail if any required item failed
        self.passed = !self.items.iter().any(|i| i.required && !i.passed);
    }

    /// How many items passed.
    pub fn passed_count(&self) -> usize {
        self.items.iter().filter(|i| i.passed).count()
    }

    /// How many items failed.
    pub fn failed_count(&self) -> usize {
        self.items.iter().filter(|i| !i.passed).count()
    }

    /// Produce a markdown-formatted review report.
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("# Creative Review: {}", self.project_title),
            format!(
                "Score: {:.0}% | {}",
                self.overall_score * 100.0,
                if self.passed {
                    "PASSED ✓"
                } else {
                    "FAILED ✗"
                }
            ),
            String::new(),
            "## Checklist".to_string(),
        ];
        for item in &self.items {
            let mark = if item.passed { "✓" } else { "✗" };
            let required = if item.required { " (required)" } else { "" };
            let notes = item
                .notes
                .as_deref()
                .map(|n| format!(" — {n}"))
                .unwrap_or_default();
            lines.push(format!("- [{}] {}{}{}", mark, item.label, required, notes));
        }
        if !self.reviewer_notes.is_empty() {
            lines.push(String::new());
            lines.push(format!("## Notes\n{}", self.reviewer_notes));
        }
        lines.join("\n")
    }
}

/// Generate the default creative production checklist.
pub fn default_checklist() -> Vec<ChecklistItem> {
    vec![
        ChecklistItem::new("All deliverables exist", true),
        ChecklistItem::new("Timeline is present", true),
        ChecklistItem::new("Every scene has sound notes", false),
        ChecklistItem::new("Every scene has edit notes", false),
        ChecklistItem::new("VFX notes are original", true),
        ChecklistItem::new("No claim that actual video was rendered", true),
        ChecklistItem::new("Storyboard is complete", false),
        ChecklistItem::new("Shot list matches scenes", false),
    ]
}

/// Legacy function — returns the old static checklist string.
pub fn review_checklist() -> String {
    "# Review Checklist\n\n- [ ] All deliverables exist\n- [ ] Timeline is present\n- [ ] Every scene has sound notes\n- [ ] Every scene has edit notes\n- [ ] VFX notes are original\n- [ ] No claim that actual video was rendered\n".to_string()
}

// ── Persistence ─────────────────────────────────────────────────────────────

pub fn save_review(store: &DiskStore, review: &CreativeReview) -> Result<()> {
    let dir = store.paths.data.join("creative_reviews");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join(format!("{}.json", review.review_id)), review)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_review_starts_with_checklist() {
        let review = CreativeReview::new("Test Project");
        assert!(!review.items.is_empty());
        assert_eq!(review.overall_score, 0.0);
    }

    #[test]
    fn evaluate_scores_correctly() {
        let mut review = CreativeReview::new("Scored Project");
        for item in &mut review.items {
            item.pass();
        }
        review.evaluate();
        assert!((review.overall_score - 1.0).abs() < f32::EPSILON);
        assert!(review.passed);
    }

    #[test]
    fn evaluate_fails_on_missing_required() {
        let mut review = CreativeReview::new("Fail Project");
        // Pass everything except the first required item
        for item in review.items.iter_mut().skip(1) {
            item.pass();
        }
        review.evaluate();
        assert!(!review.passed);
    }

    #[test]
    fn markdown_output_is_readable() {
        let mut review = CreativeReview::new("Markdown Test");
        review.items[0].pass();
        review.evaluate();
        let md = review.to_markdown();
        assert!(md.contains("Markdown Test"));
        assert!(md.contains("Checklist"));
    }

    #[test]
    fn legacy_function_returns_string() {
        let checklist = review_checklist();
        assert!(checklist.contains("deliverables"));
    }
}
