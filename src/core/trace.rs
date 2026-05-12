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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTrace {
    pub task_id: String,
    pub task_input: String,
    pub task_type: TaskType,
    pub activated_neurons: Vec<String>,
    pub activated_synapses: Vec<String>,
    pub selected_experts: Vec<String>,
    pub selected_memories: Vec<String>,
    pub tool_actions: Vec<String>,
    pub success: bool,
    pub result: String,
    pub energy_estimate: u64,
    pub runtime_ms: u128,
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
}

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
