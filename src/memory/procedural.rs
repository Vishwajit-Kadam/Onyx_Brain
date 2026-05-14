//! Procedural memory — stores reusable multi-step procedures (skills).
//!
//! A procedural skill captures a sequence of steps that proved successful for
//! a particular task type. Steps can be validated against expected preconditions,
//! tracked for execution success rate, and scored for reusability.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::storage::{load_json, save_json, DiskStore};

pub use crate::memory::MemoryItem;

/// A single step within a procedural skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureStep {
    pub step_index: usize,
    pub description: String,
    pub tool_action: Option<String>,
    pub precondition: Option<String>,
    pub expected_outcome: Option<String>,
    /// How many times this step succeeded across all executions.
    pub success_count: u64,
    /// How many times this step failed.
    pub failure_count: u64,
}

impl ProcedureStep {
    pub fn new(index: usize, description: impl Into<String>) -> Self {
        Self {
            step_index: index,
            description: description.into(),
            tool_action: None,
            precondition: None,
            expected_outcome: None,
            success_count: 0,
            failure_count: 0,
        }
    }

    /// Success rate as a fraction from 0.0 to 1.0.
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 0.5; // no data — assume neutral
        }
        self.success_count as f32 / total as f32
    }

    pub fn record_success(&mut self) {
        self.success_count += 1;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
    }
}

/// A reusable multi-step skill stored in procedural memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralSkill {
    pub skill_id: String,
    pub title: String,
    pub task_type: String,
    pub steps: Vec<ProcedureStep>,
    pub total_executions: u64,
    pub total_successes: u64,
    pub reusability_score: f32,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
}

impl ProceduralSkill {
    pub fn new(title: impl Into<String>, task_type: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            skill_id: format!("skill_{}", Uuid::new_v4()),
            title: title.into(),
            task_type: task_type.into(),
            steps: Vec::new(),
            total_executions: 0,
            total_successes: 0,
            reusability_score: 0.5,
            created_at: now,
            last_used: now,
        }
    }

    /// Add a new step to the procedure.
    pub fn add_step(&mut self, description: impl Into<String>) -> &mut ProcedureStep {
        let idx = self.steps.len();
        self.steps.push(ProcedureStep::new(idx, description));
        self.steps.last_mut().unwrap()
    }

    /// Record an execution of this skill.
    pub fn record_execution(&mut self, success: bool) {
        self.total_executions += 1;
        if success {
            self.total_successes += 1;
        }
        self.last_used = Utc::now();
        self.reusability_score = self.compute_reusability();
    }

    /// Overall success rate.
    pub fn success_rate(&self) -> f32 {
        if self.total_executions == 0 {
            return 0.5;
        }
        self.total_successes as f32 / self.total_executions as f32
    }

    /// Compute reusability: weighted by success rate, step reliability, and usage frequency.
    fn compute_reusability(&self) -> f32 {
        let success = self.success_rate();
        let step_reliability = if self.steps.is_empty() {
            0.5
        } else {
            self.steps.iter().map(|s| s.success_rate()).sum::<f32>() / self.steps.len() as f32
        };
        let usage_factor = (self.total_executions as f32 / 3.0).min(1.0); // ramp up with usage
        (success * 0.4 + step_reliability * 0.4 + usage_factor * 0.2).clamp(0.0, 1.0)
    }

    /// Validate the skill structure.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.title.trim().is_empty() {
            issues.push("Skill title is empty".into());
        }
        if self.steps.is_empty() {
            issues.push("Skill has no steps".into());
        }
        for step in &self.steps {
            if step.description.trim().is_empty() {
                issues.push(format!("Step {} has empty description", step.step_index));
            }
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Produce a concise summary.
    pub fn summarize(&self) -> String {
        format!(
            "{}: {} steps, success rate {:.0}%, reusability {:.0}%, used {} times",
            self.title,
            self.steps.len(),
            self.success_rate() * 100.0,
            self.reusability_score * 100.0,
            self.total_executions
        )
    }
}

// ── Persistence ─────────────────────────────────────────────────────────────

pub fn save_skill(store: &DiskStore, skill: &ProceduralSkill) -> Result<()> {
    let dir = store.paths.data.join("procedural");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join(format!("{}.json", skill.skill_id)), skill)
}

pub fn load_skills(store: &DiskStore) -> Result<Vec<ProceduralSkill>> {
    let dir = store.paths.data.join("procedural");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut skills = Vec::new();
    for de in fs::read_dir(&dir)? {
        let path = de?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(skill) = load_json::<ProceduralSkill>(&path) {
                skills.push(skill);
            }
        }
    }
    skills.sort_by(|a, b| b.reusability_score.total_cmp(&a.reusability_score));
    Ok(skills)
}

/// Find skills matching a given task type, sorted by reusability.
pub fn find_skills_for_task(store: &DiskStore, task_type: &str) -> Result<Vec<ProceduralSkill>> {
    let skills = load_skills(store)?;
    let normalized = task_type.to_lowercase();
    Ok(skills
        .into_iter()
        .filter(|s| s.task_type.to_lowercase() == normalized)
        .collect())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_skill_has_defaults() {
        let skill = ProceduralSkill::new("Rust project creation", "Code");
        assert_eq!(skill.steps.len(), 0);
        assert_eq!(skill.total_executions, 0);
    }

    #[test]
    fn add_step_builds_sequence() {
        let mut skill = ProceduralSkill::new("Build project", "Code");
        skill.add_step("Create Cargo.toml");
        skill.add_step("Write lib.rs");
        skill.add_step("Run cargo check");
        assert_eq!(skill.steps.len(), 3);
        assert_eq!(skill.steps[2].step_index, 2);
    }

    #[test]
    fn record_execution_updates_metrics() {
        let mut skill = ProceduralSkill::new("Test skill", "Code");
        skill.add_step("step 1");
        skill.record_execution(true);
        skill.record_execution(true);
        skill.record_execution(false);
        assert_eq!(skill.total_executions, 3);
        assert_eq!(skill.total_successes, 2);
        assert!((skill.success_rate() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn validate_rejects_empty_skill() {
        let skill = ProceduralSkill::new("", "Code");
        assert!(skill.validate().is_err());
    }

    #[test]
    fn validate_rejects_no_steps() {
        let skill = ProceduralSkill::new("Valid title", "Code");
        let result = skill.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|i| i.contains("no steps")));
    }

    #[test]
    fn step_success_rate_with_no_data_is_neutral() {
        let step = ProcedureStep::new(0, "test");
        assert!((step.success_rate() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn step_success_rate_tracks_outcomes() {
        let mut step = ProcedureStep::new(0, "cargo check");
        step.record_success();
        step.record_success();
        step.record_failure();
        assert!((step.success_rate() - 0.6667).abs() < 0.01);
    }
}
