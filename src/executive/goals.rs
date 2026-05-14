//! Executive goal management — lifecycle, priority, deadlines, and persistence.
//!
//! Goals are the executive layer's representation of what the brain is currently
//! trying to achieve. They go through a lifecycle (draft → active → completed/failed/cancelled)
//! and can be persisted across sessions.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::storage::{load_json, save_json, DiskStore};

/// Goal lifecycle states.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Priority levels for goals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// A single executive goal with lifecycle tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveGoal {
    pub goal_id: String,
    pub title: String,
    pub description: String,
    pub priority: GoalPriority,
    pub status: GoalStatus,
    pub parent_goal: Option<String>,
    pub sub_goals: Vec<String>,
    pub progress: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

impl ExecutiveGoal {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        priority: GoalPriority,
    ) -> Self {
        let now = Utc::now();
        Self {
            goal_id: format!("goal_{}", Uuid::new_v4()),
            title: title.into(),
            description: description.into(),
            priority,
            status: GoalStatus::Draft,
            parent_goal: None,
            sub_goals: Vec::new(),
            progress: 0.0,
            created_at: now,
            updated_at: now,
            completed_at: None,
            failure_reason: None,
        }
    }

    /// Transition to active state.
    pub fn activate(&mut self) -> Result<(), String> {
        match self.status {
            GoalStatus::Draft | GoalStatus::Paused => {
                self.status = GoalStatus::Active;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot activate goal in {:?} state", self.status)),
        }
    }

    /// Mark goal as completed.
    pub fn complete(&mut self) -> Result<(), String> {
        if self.status != GoalStatus::Active {
            return Err(format!("Cannot complete goal in {:?} state", self.status));
        }
        self.status = GoalStatus::Completed;
        self.progress = 1.0;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark goal as failed with a reason.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.status = GoalStatus::Failed;
        self.failure_reason = Some(reason.into());
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Pause the goal.
    pub fn pause(&mut self) -> Result<(), String> {
        if self.status != GoalStatus::Active {
            return Err(format!("Cannot pause goal in {:?} state", self.status));
        }
        self.status = GoalStatus::Paused;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update progress (0.0 to 1.0).
    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.updated_at = Utc::now();
    }

    /// Validate goal structure.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.title.trim().is_empty() {
            issues.push("Goal title is empty".into());
        }
        if self.progress < 0.0 || self.progress > 1.0 {
            issues.push(format!("Progress {} out of [0,1] range", self.progress));
        }
        if self.status == GoalStatus::Failed && self.failure_reason.is_none() {
            issues.push("Failed goal should have a failure_reason".into());
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Whether this goal is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            GoalStatus::Completed | GoalStatus::Failed | GoalStatus::Cancelled
        )
    }

    /// Produce a one-line summary.
    pub fn summarize(&self) -> String {
        format!(
            "[{}] {} — {} ({:.0}% done, priority: {:?})",
            self.goal_id,
            self.title,
            self.status,
            self.progress * 100.0,
            self.priority
        )
    }
}

/// Summary for the executive layer to use in status reports.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutiveGoalSummary {
    pub active_goal: Option<String>,
    pub next_best_action: String,
    pub total_goals: usize,
    pub active_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

// ── Persistence ─────────────────────────────────────────────────────────────

pub fn save_goal(store: &DiskStore, goal: &ExecutiveGoal) -> Result<()> {
    let dir = store.paths.executive.join("goals");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join(format!("{}.json", goal.goal_id)), goal)
}

pub fn load_goals(store: &DiskStore) -> Result<Vec<ExecutiveGoal>> {
    let dir = store.paths.executive.join("goals");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut goals = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(goal) = load_json::<ExecutiveGoal>(&path) {
                goals.push(goal);
            }
        }
    }
    goals.sort_by(|a, b| b.priority.cmp(&a.priority));
    Ok(goals)
}

/// Build a summary of all goals for executive status.
pub fn goal_summary(store: &DiskStore) -> Result<ExecutiveGoalSummary> {
    let goals = load_goals(store)?;
    let active: Vec<_> = goals
        .iter()
        .filter(|g| g.status == GoalStatus::Active)
        .collect();
    Ok(ExecutiveGoalSummary {
        active_goal: active.first().map(|g| g.title.clone()),
        next_best_action: if active.is_empty() {
            "No active goals — waiting for user input.".into()
        } else {
            format!("Continue: {}", active[0].title)
        },
        total_goals: goals.len(),
        active_count: active.len(),
        completed_count: goals
            .iter()
            .filter(|g| g.status == GoalStatus::Completed)
            .count(),
        failed_count: goals
            .iter()
            .filter(|g| g.status == GoalStatus::Failed)
            .count(),
    })
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_lifecycle_draft_to_complete() {
        let mut goal =
            ExecutiveGoal::new("Build project", "Create a Rust project", GoalPriority::High);
        assert_eq!(goal.status, GoalStatus::Draft);
        assert!(goal.activate().is_ok());
        assert_eq!(goal.status, GoalStatus::Active);
        goal.update_progress(0.5);
        assert!((goal.progress - 0.5).abs() < f32::EPSILON);
        assert!(goal.complete().is_ok());
        assert_eq!(goal.status, GoalStatus::Completed);
        assert!(goal.is_terminal());
    }

    #[test]
    fn cannot_complete_draft_goal() {
        let mut goal = ExecutiveGoal::new("Test", "desc", GoalPriority::Low);
        assert!(goal.complete().is_err());
    }

    #[test]
    fn fail_sets_reason() {
        let mut goal = ExecutiveGoal::new("Test", "desc", GoalPriority::Medium);
        goal.activate().unwrap();
        goal.fail("safety boundary hit");
        assert_eq!(goal.status, GoalStatus::Failed);
        assert!(goal.failure_reason.as_ref().unwrap().contains("safety"));
    }

    #[test]
    fn validate_rejects_empty_title() {
        let goal = ExecutiveGoal::new("", "desc", GoalPriority::Low);
        assert!(goal.validate().is_err());
    }

    #[test]
    fn pause_and_resume() {
        let mut goal = ExecutiveGoal::new("Test", "desc", GoalPriority::Medium);
        goal.activate().unwrap();
        goal.pause().unwrap();
        assert_eq!(goal.status, GoalStatus::Paused);
        goal.activate().unwrap();
        assert_eq!(goal.status, GoalStatus::Active);
    }
}
