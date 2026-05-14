//! Execution trace tracking and self-evaluation.
//!
//! Stores the trajectory of cognitive events (activated neurons, tools used,
//! skills reused, and decisions made) during a cognitive cycle. The trace
//! forms the episodic basis for learning and habit formation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    agency::{FastPathDecision, PlanCacheMatch, RecoveryPlan, ReliabilityScore},
    core::TaskType,
    energy::{AdaptiveBudgetDecision, EnergyReport, RuntimeBreakdown},
    learning::{
        AutoOptimizeHint, HabitMatch, LearningReport, LiveHabitUpdate, SkillApplicationResult,
        SkillMatch,
    },
    tools::CargoValidationPolicy,
};

/// A sequential record of a single task's cognitive execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTrace {
    pub task_id: String,
    pub task_input: String,
    pub task_type: TaskType,

    // Cognitive activity logs
    pub activated_neurons: Vec<String>,
    pub activated_synapses: Vec<String>,
    pub selected_experts: Vec<String>,
    pub selected_memories: Vec<String>,
    pub tool_actions: Vec<String>,

    // Outcomes
    pub success: bool,
    pub result: String,
    pub energy_estimate: u64,
    pub runtime_ms: u128,

    // Reports & Learning outputs
    pub energy_report: EnergyReport,
    pub learning_updates: LearningReport,

    #[serde(default)]
    pub reused_skills: Vec<SkillMatch>,
    #[serde(default)]
    pub skill_application_results: Vec<SkillApplicationResult>,
    #[serde(default)]
    pub habits_used: Vec<HabitMatch>,
    #[serde(default)]
    pub plan_cache_match: Option<PlanCacheMatch>,
    #[serde(default)]
    pub adaptive_budget: Option<AdaptiveBudgetDecision>,
    #[serde(default)]
    pub live_habit_update: Option<LiveHabitUpdate>,
    #[serde(default)]
    pub fast_path_decision: Option<FastPathDecision>,
    #[serde(default)]
    pub cargo_validation_policy: Option<CargoValidationPolicy>,
    #[serde(default)]
    pub runtime_breakdown: Option<RuntimeBreakdown>,
    #[serde(default)]
    pub optimization_hint: Option<AutoOptimizeHint>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub journal_entries: Vec<String>,
    #[serde(default)]
    pub snapshot_ids: Vec<String>,
    #[serde(default)]
    pub transaction_ids: Vec<String>,
    #[serde(default)]
    pub recovery_plan: Option<RecoveryPlan>,
    #[serde(default)]
    pub reliability_score: Option<ReliabilityScore>,

    /// Wall-clock time tracking for accurate latency analysis
    #[serde(default)]
    pub timestamps: TraceTimestamps,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceTimestamps {
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// A boolean checklist self-evaluation conducted immediately after execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReview {
    pub task_completed: bool,
    pub expected_files_created: bool,
    pub cargo_check_passed_if_attempted: bool,
    pub stayed_inside_activation_budget: bool,
    pub tools_stayed_sandboxed: bool,
    pub energy_recorded: bool,
    pub success: bool,
    pub notes: Vec<String>,
}

impl SelfReview {
    /// Validates the review logic. A task cannot be marked successful if it bypassed the sandbox
    /// or if it was supposed to create files but didn't.
    pub fn is_structurally_valid(&self) -> bool {
        if !self.tools_stayed_sandboxed && self.success {
            return false; // Critical safety violation cannot be considered a success
        }
        if !self.expected_files_created && self.success {
            return false; // Missed core deliverable
        }
        true
    }
}

/// A continuous numerical evaluation of execution quality used for reinforcement learning.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelfEvaluation {
    pub correctness_score: f32,
    pub test_coverage_score: f32,
    pub completeness_score: f32,
    pub energy_efficiency_score: f32,
    #[serde(default)]
    pub skill_reuse_score: f32,
    #[serde(default)]
    pub memory_hygiene_score: f32,
    #[serde(default)]
    pub habit_reuse_score: f32,
    #[serde(default)]
    pub plan_cache_score: f32,
    #[serde(default)]
    pub route_efficiency_score: f32,
    #[serde(default)]
    pub irrelevant_skill_penalty: f32,
    pub overall_score: f32,
    pub notes: Vec<String>,
}

impl SelfEvaluation {
    /// Compute the overall evaluation score given the individual component metrics.
    /// Incorporates penalties for inefficient operations or hallucinated skills.
    pub fn compute_overall_score(&mut self) {
        let base_score = (self.correctness_score * 0.4)
            + (self.completeness_score * 0.3)
            + (self.test_coverage_score * 0.2)
            + (self.energy_efficiency_score * 0.1);

        let bonus = (self.skill_reuse_score * 0.05)
            + (self.habit_reuse_score * 0.05)
            + (self.plan_cache_score * 0.05);

        let raw_total = base_score + bonus - self.irrelevant_skill_penalty;

        self.overall_score = raw_total.clamp(0.0, 1.0);
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_review_structural_validation_catches_sandbox_violations() {
        let review = SelfReview {
            task_completed: true,
            expected_files_created: true,
            cargo_check_passed_if_attempted: true,
            stayed_inside_activation_budget: true,
            tools_stayed_sandboxed: false, // Violation
            energy_recorded: true,
            success: true, // Incorrectly marked as success
            notes: vec![],
        };
        assert!(!review.is_structurally_valid());
    }

    #[test]
    fn self_review_structural_validation_catches_missing_files() {
        let review = SelfReview {
            task_completed: true,
            expected_files_created: false, // Violation
            cargo_check_passed_if_attempted: true,
            stayed_inside_activation_budget: true,
            tools_stayed_sandboxed: true,
            energy_recorded: true,
            success: true, // Incorrectly marked as success
            notes: vec![],
        };
        assert!(!review.is_structurally_valid());
    }

    #[test]
    fn compute_overall_score_clamps_to_range() {
        let mut eval = SelfEvaluation {
            correctness_score: 1.0,
            test_coverage_score: 1.0,
            completeness_score: 1.0,
            energy_efficiency_score: 1.0,
            skill_reuse_score: 1.0,
            habit_reuse_score: 1.0,
            plan_cache_score: 1.0,
            irrelevant_skill_penalty: 0.0,
            ..Default::default()
        };

        // Base is 1.0. Bonus is 0.15. Total 1.15. Should clamp to 1.0.
        eval.compute_overall_score();
        assert!((eval.overall_score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_overall_score_applies_penalty() {
        let mut eval = SelfEvaluation {
            correctness_score: 0.8,
            test_coverage_score: 0.5,
            completeness_score: 0.8,
            energy_efficiency_score: 0.5,
            skill_reuse_score: 0.0,
            habit_reuse_score: 0.0,
            plan_cache_score: 0.0,
            irrelevant_skill_penalty: 0.3,
            ..Default::default()
        };

        eval.compute_overall_score();
        // base = 0.32 + 0.24 + 0.10 + 0.05 = 0.71
        // penalty = 0.3
        // total = 0.41
        assert!(eval.overall_score > 0.4 && eval.overall_score < 0.42);
    }
}
