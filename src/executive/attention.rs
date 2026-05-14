//! Executive attention — dynamic focus management with distraction detection.
//!
//! Attention tracks what the executive layer is currently focused on,
//! detects when context switches happen, and maintains a focus quality score
//! that degrades with excessive context switches.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Maximum number of context switches before focus degrades significantly.
const MAX_CONTEXT_SWITCHES: usize = 10;

/// Tracks the brain's current attentional focus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionState {
    pub active_goal: Option<String>,
    pub active_task: Option<String>,
    pub active_context: Vec<String>,
    pub focus_score: f32,
    pub distraction_flags: Vec<String>,
    pub context_switch_count: usize,
    pub last_switch_at: Option<DateTime<Utc>>,
    pub sustained_focus_minutes: f32,
}

impl Default for AttentionState {
    fn default() -> Self {
        Self {
            active_goal: None,
            active_task: None,
            active_context: vec!["current workspace".to_string(), "safety policy".to_string()],
            focus_score: 1.0,
            distraction_flags: Vec::new(),
            context_switch_count: 0,
            last_switch_at: None,
            sustained_focus_minutes: 0.0,
        }
    }
}

impl AttentionState {
    /// Set focus to a specific goal and task.
    pub fn focus_on(&mut self, goal: Option<String>, task: Option<String>) {
        let changed_goal = self.active_goal != goal;
        let changed_task = self.active_task != task;
        if changed_goal || changed_task {
            self.context_switch_count += 1;
            self.last_switch_at = Some(Utc::now());
            self.sustained_focus_minutes = 0.0;
        }
        self.active_goal = goal;
        self.active_task = task;
        self.recompute_focus_score();
    }

    /// Add a contextual note to the attention state.
    pub fn add_context(&mut self, ctx: impl Into<String>) {
        let c = ctx.into();
        if !self.active_context.contains(&c) {
            self.active_context.push(c);
        }
    }

    /// Flag a distraction.
    pub fn flag_distraction(&mut self, reason: impl Into<String>) {
        self.distraction_flags.push(reason.into());
        self.recompute_focus_score();
    }

    /// Clear distractions (e.g., after resolving them).
    pub fn clear_distractions(&mut self) {
        self.distraction_flags.clear();
        self.recompute_focus_score();
    }

    /// Recompute focus score based on context switches and distractions.
    fn recompute_focus_score(&mut self) {
        let switch_penalty =
            (self.context_switch_count as f32 / MAX_CONTEXT_SWITCHES as f32).min(1.0) * 0.4;
        let distraction_penalty = (self.distraction_flags.len() as f32 * 0.1).min(0.3);
        self.focus_score = (1.0 - switch_penalty - distraction_penalty).clamp(0.0, 1.0);
    }

    /// Whether the attention state indicates good focus.
    pub fn is_focused(&self) -> bool {
        self.focus_score > 0.6 && self.distraction_flags.is_empty()
    }

    /// Reset context switch counter (e.g., at start of new session).
    pub fn reset_switches(&mut self) {
        self.context_switch_count = 0;
        self.distraction_flags.clear();
        self.focus_score = 1.0;
        self.sustained_focus_minutes = 0.0;
    }

    /// Produce a summary.
    pub fn summarize(&self) -> String {
        let goal = self.active_goal.as_deref().unwrap_or("none");
        let task = self.active_task.as_deref().unwrap_or("none");
        format!(
            "Attention: goal='{}', task='{}', focus={:.0}%, switches={}, distractions={}",
            goal,
            task,
            self.focus_score * 100.0,
            self.context_switch_count,
            self.distraction_flags.len()
        )
    }
}

/// Convenience constructor matching the original API.
pub fn attention_state(active_goal: Option<String>, active_task: Option<String>) -> AttentionState {
    let mut state = AttentionState::default();
    state.active_goal = active_goal;
    state.active_task = active_task;
    state
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_focused() {
        let state = AttentionState::default();
        assert!(state.is_focused());
        assert!((state.focus_score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn context_switches_degrade_focus() {
        let mut state = AttentionState::default();
        for i in 0..8 {
            state.focus_on(Some(format!("goal_{i}")), None);
        }
        assert!(state.focus_score < 0.8);
        assert_eq!(state.context_switch_count, 8);
    }

    #[test]
    fn distractions_degrade_focus() {
        let mut state = AttentionState::default();
        state.flag_distraction("unrelated user query");
        state.flag_distraction("timeout warning");
        assert!(state.focus_score < 1.0);
        assert!(!state.is_focused());
    }

    #[test]
    fn clear_distractions_restores_focus() {
        let mut state = AttentionState::default();
        state.flag_distraction("noise");
        assert!(!state.is_focused());
        state.clear_distractions();
        assert!(state.is_focused());
    }

    #[test]
    fn reset_restores_full_focus() {
        let mut state = AttentionState::default();
        state.focus_on(Some("g1".into()), None);
        state.focus_on(Some("g2".into()), None);
        state.flag_distraction("noise");
        state.reset_switches();
        assert_eq!(state.context_switch_count, 0);
        assert!((state.focus_score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn add_context_avoids_duplicates() {
        let mut state = AttentionState::default();
        state.add_context("safety policy");
        state.add_context("safety policy");
        assert_eq!(
            state
                .active_context
                .iter()
                .filter(|c| *c == "safety policy")
                .count(),
            1
        );
    }
}
