//! Executive reflection — structured post-session analysis with scoring and persistence.
//!
//! Reflections capture what went well, what failed, and what to improve next time.
//! They are scored for actionability and accumulated over time to form the brain's
//! "experience" about its own execution patterns.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::storage::{load_json, save_json, DiskStore};

/// A structured reflection produced after a session or major goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveReflection {
    pub reflection_id: String,
    pub session_id: String,
    pub lesson: String,
    pub safety_note: String,
    pub what_worked: Vec<String>,
    pub what_failed: Vec<String>,
    pub improvements: Vec<String>,
    pub actionability_score: f32,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

impl ExecutiveReflection {
    pub fn new(session_id: &str, lesson: impl Into<String>) -> Self {
        Self {
            reflection_id: format!("reflection_{}", Uuid::new_v4()),
            session_id: session_id.to_string(),
            lesson: lesson.into(),
            safety_note: String::new(),
            what_worked: Vec::new(),
            what_failed: Vec::new(),
            improvements: Vec::new(),
            actionability_score: 0.0,
            confidence: 0.5,
            created_at: Utc::now(),
        }
    }

    /// Record something that worked well.
    pub fn add_success(&mut self, pattern: impl Into<String>) {
        self.what_worked.push(pattern.into());
        self.recompute_score();
    }

    /// Record something that failed.
    pub fn add_failure(&mut self, pattern: impl Into<String>) {
        self.what_failed.push(pattern.into());
        self.recompute_score();
    }

    /// Record an improvement for next time.
    pub fn add_improvement(&mut self, improvement: impl Into<String>) {
        self.improvements.push(improvement.into());
        self.recompute_score();
    }

    /// Compute actionability: reflections with specific improvements are more actionable.
    fn recompute_score(&mut self) {
        let improvement_weight = (self.improvements.len() as f32 * 0.3).min(1.0);
        let failure_learning = if !self.what_failed.is_empty() && !self.improvements.is_empty() {
            0.3 // failures + improvements = learning
        } else {
            0.0
        };
        let lesson_quality = if self.lesson.len() > 20 { 0.2 } else { 0.1 };
        self.actionability_score =
            (improvement_weight + failure_learning + lesson_quality).clamp(0.0, 1.0);
    }

    /// Validate reflection integrity.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.lesson.trim().is_empty() {
            issues.push("Reflection lesson is empty".into());
        }
        if self.session_id.trim().is_empty() {
            issues.push("Session ID is empty".into());
        }
        if self.actionability_score < 0.0 || self.actionability_score > 1.0 {
            issues.push("Actionability score out of range".into());
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Summarize this reflection in one line.
    pub fn summarize(&self) -> String {
        format!(
            "Reflection [{}]: {} (+{} -{}), actionability {:.0}%",
            self.session_id,
            truncate_str(&self.lesson, 50),
            self.what_worked.len(),
            self.what_failed.len(),
            self.actionability_score * 100.0
        )
    }
}

// ── Persistence ─────────────────────────────────────────────────────────────

pub fn save_reflection(store: &DiskStore, reflection: &ExecutiveReflection) -> Result<()> {
    let dir = store.paths.executive.join("reflections");
    fs::create_dir_all(&dir)?;
    save_json(
        &dir.join(format!("{}.json", reflection.reflection_id)),
        reflection,
    )
}

pub fn load_reflections(store: &DiskStore, limit: usize) -> Result<Vec<ExecutiveReflection>> {
    let dir = store.paths.executive.join("reflections");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut reflections = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(r) = load_json::<ExecutiveReflection>(&path) {
                reflections.push(r);
            }
        }
    }
    reflections.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    reflections.truncate(limit);
    Ok(reflections)
}

/// Extract the most actionable reflections (highest actionability score).
pub fn most_actionable_reflections(
    store: &DiskStore,
    limit: usize,
) -> Result<Vec<ExecutiveReflection>> {
    let mut reflections = load_reflections(store, 100)?;
    reflections.sort_by(|a, b| b.actionability_score.total_cmp(&a.actionability_score));
    reflections.truncate(limit);
    Ok(reflections)
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reflection_has_defaults() {
        let r = ExecutiveReflection::new("s1", "Test lesson");
        assert_eq!(r.what_worked.len(), 0);
        assert_eq!(r.what_failed.len(), 0);
        assert!(r.validate().is_ok());
    }

    #[test]
    fn adding_patterns_updates_score() {
        let mut r = ExecutiveReflection::new("s1", "Learned about dependency ordering");
        r.add_success("task decomposition worked well");
        r.add_failure("missed edge case in validation");
        r.add_improvement("add validation step before execution");
        assert!(r.actionability_score > 0.0);
    }

    #[test]
    fn validate_rejects_empty_lesson() {
        let r = ExecutiveReflection::new("s1", "");
        assert!(r.validate().is_err());
    }

    #[test]
    fn summarize_is_readable() {
        let mut r = ExecutiveReflection::new("s1", "Better error handling needed");
        r.add_success("tests passed");
        r.add_failure("missed error case");
        let s = r.summarize();
        assert!(s.contains("+1"));
        assert!(s.contains("-1"));
    }
}
