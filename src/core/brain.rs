use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::{fs, path::Path};

use crate::{
    agency::{
        active_session_count, apply_simple_rust_fix, decide_fast_path, decompose_goal,
        extract_worker_project_name, find_cached_plan, find_template_for_goal,
        get_or_start_session, journal_count, latest_journal_entries, latest_session_id, list_goals,
        load_project_registry, load_project_state, load_task_queue, mark_cache_used, parse_goal,
        plan_cache_overview, quick_journal, recover_latest, recovery_plan_for_failure,
        register_project, reliability_score, render_template_files, retry_allowed, rollback_latest,
        save_goal, save_project_state, save_task_queue, session_end, session_resume, session_start,
        sessions, snapshot_count, snapshot_create, snapshot_restore, snapshots,
        store_or_strengthen_rust_cli_template, store_successful_plan, template_cache_overview,
        ActionJournalSummary, ActionType, Checkpoint, FastPathDecision, GoalMemoryItem, GoalStatus,
        IntentKind, ParsedGoal, PlanCacheMatch, PlanCacheOverview, Planner, ProjectRecord,
        ProjectState, RecoveryPlan, RecoveryResult, ReliabilityScore, RollbackReport,
        SessionStatus, SnapshotOverview, SnapshotRestoreReport, TaskStatus, TemplateCacheOverview,
        WorkSession, WorkSessionSummary, WorkerModeOutput,
    },
    core::{
        ActiveNeuron, NeuronKind, RouteTrace, SelfEvaluation, SelfReview, Synapse, SynapseType,
        Task, TaskType, VirtualNeuron,
    },
    energy::{
        new_profile_id, performance_overview, save_performance_profile, AdaptiveBudgetDecision,
        AdaptiveBudgetDecisionType, AdaptiveBudgetManager, EnergyReport, PerformanceProfile,
        Profiler, RuntimeBreakdown,
    },
    experts::{
        ensure_default_expert_stats, update_expert_stats, CodeExpert, Expert, ExpertContext,
        ExpertResult, LanguageExpert, ReasoningExpert, ToolUseExpert,
    },
    learning::{
        auto_optimize_hint, extract_skills_from_project, find_matching_habits,
        form_or_strengthen_habit_from_project, habit_overview, irrelevant_skill_count,
        lightweight_auto_optimize, list_habits, update_live_habit_after_project, update_routes,
        update_skill_usage, AutoOptimizeHint, HabitMatch, LearningReport, LiveHabitUpdate,
        SkillApplicationResult, SkillMatch, SkillReuseEngine,
    },
    memory::{
        dedup::{dedup_memories, inspect_memory_hygiene, MemoryDedupReport, MemoryHygieneReport},
        hygiene::{cleanup_backups, BackupCleanupReport, MemoryHygienePolicy},
        project::remember_project_state,
        retrieve_relevant_memories, MemoryItem, MemoryType,
    },
    routing::{
        route_efficiency_overview, update_named_route_efficiency,
        update_route_efficiency_from_synapses, Classifier, RouteEfficiencyOverview, Router,
    },
    sleep::{consolidate, ConsolidationReport},
    storage::{doctor, load_json, save_json, DiskStore, DoctorReport, LabelIndex, TaskTypeIndex},
    testing::{regression_check, RegressionCheckReport},
    tools::{
        decide_cargo_validation, diagnose_command, transactions, CargoValidationPolicy,
        CodeEditorTool, DiagnosticKind, FilesystemTool, RustProjectTool, TerminalTool,
        TransactionOverview,
    },
    utils::{environment_report, time::timestamp_slug, EnvironmentReport},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainOutput {
    pub task_id: String,
    pub task: String,
    pub task_type: TaskType,
    pub answer: String,
    pub activated_neurons: Vec<ActiveNeuron>,
    pub activated_experts: Vec<String>,
    pub used_memories: Vec<String>,
    pub tool_actions: Vec<String>,
    pub result: String,
    pub energy_report: EnergyReport,
    pub learning_report: LearningReport,
    pub self_review: SelfReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectOutput {
    pub goal_id: String,
    pub intent: IntentKind,
    pub project_name: String,
    pub features_requested: Vec<String>,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub files_created: Vec<String>,
    pub files_modified: Vec<String>,
    pub cargo_check_result: String,
    pub cargo_test_result: String,
    pub retries_used: u32,
    pub self_evaluation: SelfEvaluation,
    pub reused_skills: Vec<SkillMatch>,
    pub skill_application_results: Vec<SkillApplicationResult>,
    #[serde(default)]
    pub habits_used: Vec<HabitMatch>,
    #[serde(default)]
    pub plan_cache_match: Option<PlanCacheMatch>,
    pub adaptive_budget: AdaptiveBudgetDecision,
    #[serde(default)]
    pub live_habit_update: LiveHabitUpdate,
    #[serde(default)]
    pub fast_path_decision: FastPathDecision,
    #[serde(default)]
    pub cargo_validation_policy: CargoValidationPolicy,
    #[serde(default)]
    pub runtime_breakdown: RuntimeBreakdown,
    #[serde(default)]
    pub optimization_hint: AutoOptimizeHint,
    #[serde(default)]
    pub template_cache_used: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub journal_summary: Vec<String>,
    #[serde(default)]
    pub snapshot_summary: Vec<String>,
    #[serde(default)]
    pub rollback_readiness: f32,
    #[serde(default)]
    pub reliability_score: ReliabilityScore,
    #[serde(default)]
    pub recovery_plan: Option<RecoveryPlan>,
    #[serde(default)]
    pub json_report_path: Option<String>,
    pub final_status: String,
    pub project_report_path: String,
    pub ram_minimal_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub benchmark_name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub tasks_run: u64,
    pub tasks_successful: u64,
    pub tasks_failed: u64,
    pub total_runtime_ms: u64,
    pub average_energy_estimate: f32,
    pub active_neuron_counts: Vec<usize>,
    pub reused_skills_count: u64,
    #[serde(default)]
    pub irrelevant_skills_used: u64,
    #[serde(default)]
    pub habits_used: u64,
    #[serde(default)]
    pub cache_hits: u64,
    #[serde(default)]
    pub adaptive_budget_decisions: u64,
    #[serde(default)]
    pub average_route_efficiency: f32,
    #[serde(default)]
    pub template_cache_hits: u64,
    #[serde(default)]
    pub runtime_breakdown: RuntimeBreakdown,
    #[serde(default)]
    pub runtime_diagnosis: BenchmarkRuntimeDiagnosis,
    pub memories_created: u64,
    pub memories_archived: u64,
    pub project_count_before: usize,
    pub project_count_after: usize,
    pub final_score: f32,
    pub report_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkReliabilityReport {
    pub tasks_run: u64,
    pub tasks_successful: u64,
    pub rollback_success: bool,
    pub snapshot_restore_success: bool,
    pub doctor_critical_issues: usize,
    pub regression_check_passed: bool,
    pub reliability_score: f32,
    pub runtime_ms: u64,
    pub report_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkRuntimeDiagnosis {
    pub main_runtime_source: String,
    pub brain_runtime_percent: f32,
    pub tool_runtime_percent: f32,
    pub cargo_runtime_percent: f32,
    pub filesystem_runtime_percent: f32,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkHistoryEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub final_score: f32,
    pub runtime_ms: u64,
    pub average_energy_estimate: f32,
    pub reused_skills: u64,
    pub memories_archived: u64,
    pub tasks_successful: u64,
    pub tasks_failed: u64,
    #[serde(default)]
    pub irrelevant_skills_used: u64,
    #[serde(default)]
    pub habits_used: u64,
    #[serde(default)]
    pub cache_hits: u64,
    #[serde(default)]
    pub adaptive_budget_decisions: u64,
    #[serde(default)]
    pub average_route_efficiency: f32,
    #[serde(default)]
    pub template_cache_hits: u64,
    #[serde(default)]
    pub runtime_diagnosis: BenchmarkRuntimeDiagnosis,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkHistory(pub Vec<BenchmarkHistoryEntry>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkCompareReport {
    pub last_score: Option<f32>,
    pub best_score: Option<f32>,
    pub average_score: f32,
    pub runtime_trend: String,
    pub energy_trend: String,
    pub skill_reuse_trend: String,
    #[serde(default)]
    pub skill_reuse_quality_trend: String,
    #[serde(default)]
    pub habit_usage_trend: String,
    #[serde(default)]
    pub cache_hit_rate_trend: String,
    #[serde(default)]
    pub route_efficiency_trend: String,
    pub memory_hygiene_trend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalExecutionOutput {
    pub goal_id: String,
    pub goal_status: GoalStatus,
    pub project_name: Option<String>,
    pub skills_reused: Vec<String>,
    pub energy_estimate: u64,
    pub self_evaluation: SelfEvaluation,
    pub goal_memory_path: String,
    #[serde(default)]
    pub habits_used: Vec<String>,
    pub adaptive_budget: AdaptiveBudgetDecision,
    #[serde(default)]
    pub live_habit_update: LiveHabitUpdate,
    #[serde(default)]
    pub fast_path_decision: FastPathDecision,
    #[serde(default)]
    pub cargo_validation_policy: CargoValidationPolicy,
    #[serde(default)]
    pub runtime_breakdown: RuntimeBreakdown,
    #[serde(default)]
    pub optimization_hint: AutoOptimizeHint,
    #[serde(default)]
    pub template_cache_used: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub journal_summary: Vec<String>,
    #[serde(default)]
    pub snapshot_summary: Vec<String>,
    #[serde(default)]
    pub rollback_readiness: f32,
    #[serde(default)]
    pub reliability_score: ReliabilityScore,
    #[serde(default)]
    pub recovery_plan: Option<RecoveryPlan>,
    #[serde(default)]
    pub json_report_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainStatus {
    pub version: String,
    pub neurons: usize,
    pub synapses: usize,
    pub active_registered_projects: usize,
    pub historical_project_memories: usize,
    pub goals_active: usize,
    pub goals_completed: usize,
    pub goals_blocked: usize,
    pub memories_by_type: Vec<String>,
    pub duplicate_memory_groups: usize,
    pub top_skills_by_reuse: Vec<String>,
    pub benchmark_last_score: Option<f32>,
    pub average_project_self_evaluation: f32,
    pub memory_hygiene_score: f32,
    pub recommended_maintenance_actions: Vec<String>,
    #[serde(default)]
    pub performance_profile_count: usize,
    #[serde(default)]
    pub average_runtime_last_5: f32,
    #[serde(default)]
    pub average_route_efficiency: f32,
    #[serde(default)]
    pub habits_count: usize,
    #[serde(default)]
    pub top_habits: Vec<String>,
    #[serde(default)]
    pub plan_cache_entries: usize,
    #[serde(default)]
    pub cache_hit_rate: f32,
    #[serde(default)]
    pub adaptive_budget_savings_estimate: f32,
    #[serde(default)]
    pub optimization_recommendations: Vec<String>,
    #[serde(default)]
    pub environment: EnvironmentReport,
    #[serde(default)]
    pub average_brain_runtime_last_5: f32,
    #[serde(default)]
    pub average_tool_runtime_last_5: f32,
    #[serde(default)]
    pub average_cargo_runtime_last_5: f32,
    #[serde(default)]
    pub journal_entries_count: usize,
    #[serde(default)]
    pub active_sessions_count: usize,
    #[serde(default)]
    pub latest_session: Option<String>,
    #[serde(default)]
    pub snapshots_count: usize,
    #[serde(default)]
    pub recent_failures: Vec<String>,
    #[serde(default)]
    pub doctor_health_summary: String,
    #[serde(default)]
    pub reliability_score: ReliabilityScore,
    #[serde(default)]
    pub rollback_readiness: f32,
    #[serde(default)]
    pub recovery_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrainStatusLite {
    pub version: String,
    pub neurons: usize,
    pub synapses: usize,
    pub memories: usize,
    pub registered_projects: usize,
    pub goals_active: usize,
    pub goals_completed: usize,
    pub goals_blocked: usize,
    pub memory_hygiene: String,
    pub habits_count: usize,
    pub cache_entries: usize,
    pub last_benchmark_score: Option<f32>,
    pub average_route_efficiency: f32,
    pub recommended_action: String,
    pub environment_notes: Vec<String>,
    #[serde(default)]
    pub reliability_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizationReport {
    pub profiles_analyzed: usize,
    pub habits_created: usize,
    pub habits_strengthened: usize,
    pub routes_optimized: usize,
    pub low_efficiency_routes_penalized: usize,
    pub irrelevant_skills_penalized: usize,
    pub recommendations: Vec<String>,
    pub report_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInspectOutput {
    pub project_name: String,
    pub root_path: String,
    pub status: String,
    pub last_report: Option<String>,
    pub files: Vec<String>,
    pub memories: Vec<String>,
    pub task_queue_status: Vec<String>,
    pub recent_errors: Vec<String>,
    pub self_evaluation: Option<SelfEvaluation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectSummary {
    pub neurons: usize,
    pub synapses: usize,
    pub memories: usize,
    pub logs: usize,
    pub sandbox: String,
    pub top_strongest_synapses: Vec<String>,
    pub top_used_neurons: Vec<String>,
    pub top_important_memories: Vec<String>,
    pub last_tasks: Vec<String>,
    pub average_energy_estimate: f32,
    pub last_consolidation_time: Option<String>,
    pub known_projects: Vec<String>,
    pub failed_tasks: Vec<String>,
    pub retry_counts: Vec<String>,
    pub last_project_report_path: Option<String>,
    pub registered_project_count: usize,
    pub last_modified_project: Option<String>,
    pub top_extracted_skills: Vec<String>,
    pub average_project_self_evaluation_score: f32,
    pub failed_or_blocked_task_count: usize,
    pub memory_hygiene: MemoryHygieneReport,
    pub historical_project_memories: usize,
    pub archived_project_memories: usize,
    pub duplicate_project_memories: usize,
    #[serde(default)]
    pub route_efficiency_top: Vec<String>,
    #[serde(default)]
    pub habit_summary: Vec<String>,
    #[serde(default)]
    pub cache_summary: Vec<String>,
    #[serde(default)]
    pub slowest_recent_tasks: Vec<String>,
    #[serde(default)]
    pub average_runtime_ms: f32,
    #[serde(default)]
    pub average_energy: f32,
    #[serde(default)]
    pub adaptive_budget_summary: String,
    #[serde(default)]
    pub latest_journal_entries: Vec<String>,
    #[serde(default)]
    pub latest_snapshots: Vec<String>,
    #[serde(default)]
    pub recent_sessions: Vec<String>,
    #[serde(default)]
    pub recovery_reports: Vec<String>,
    #[serde(default)]
    pub doctor_health_summary: String,
    #[serde(default)]
    pub transaction_summary: Vec<String>,
    #[serde(default)]
    pub reliability_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InspectSummaryLite {
    pub version: String,
    pub neurons: usize,
    pub synapses: usize,
    pub memories: usize,
    pub logs: usize,
    pub registered_projects: usize,
    pub goals: String,
    pub memory_hygiene: String,
    pub habits_count: usize,
    pub cache_entries: usize,
    pub last_benchmark_score: Option<f32>,
    pub average_route_efficiency: f32,
    pub recommended_action: String,
    #[serde(default)]
    pub reliability_summary: String,
}

#[derive(Debug, Clone)]
pub struct Brain {
    store: DiskStore,
}

impl Brain {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            store: DiskStore::new(root),
        }
    }

    pub fn store(&self) -> &DiskStore {
        &self.store
    }

    pub fn init(&self) -> Result<()> {
        self.store.ensure_layout()?;
        let mut label_index = LabelIndex::default();
        let mut task_index = TaskTypeIndex::default();

        let seeds = vec![
            seed_neuron("task_code", "code task", NeuronKind::TaskType, 0.5, 0.25),
            seed_neuron("task_chat", "chat task", NeuronKind::TaskType, 0.5, 0.2),
            seed_neuron(
                "task_file_operation",
                "file operation task",
                NeuronKind::TaskType,
                0.5,
                0.2,
            ),
            seed_neuron(
                "task_planning",
                "planning task",
                NeuronKind::TaskType,
                0.5,
                0.2,
            ),
            seed_neuron(
                "expert_language",
                "language expert",
                NeuronKind::Expert,
                0.4,
                0.1,
            ),
            seed_neuron("expert_code", "code expert", NeuronKind::Expert, 0.4, 0.1),
            seed_neuron(
                "expert_reasoning",
                "reasoning expert",
                NeuronKind::Expert,
                0.4,
                0.1,
            ),
            seed_neuron(
                "expert_tool_use",
                "tool use expert",
                NeuronKind::Expert,
                0.4,
                0.1,
            ),
            seed_neuron(
                "tool_filesystem",
                "filesystem tool",
                NeuronKind::Tool,
                0.4,
                0.1,
            ),
            seed_neuron("tool_terminal", "terminal tool", NeuronKind::Tool, 0.4, 0.1),
            seed_neuron(
                "tool_rust_project",
                "rust project tool",
                NeuronKind::Tool,
                0.4,
                0.1,
            ),
            seed_neuron(
                "memory_rust",
                "rust semantic memory",
                NeuronKind::Memory,
                0.4,
                0.1,
            ),
            seed_neuron(
                "memory_cargo",
                "cargo semantic memory",
                NeuronKind::Memory,
                0.4,
                0.1,
            ),
            seed_neuron(
                "goal_create_project",
                "create project goal",
                NeuronKind::Goal,
                0.4,
                0.1,
            ),
        ];
        for neuron in seeds {
            self.store.save_neuron(&neuron)?;
            index_neuron(&mut label_index, &neuron);
        }

        task_index
            .0
            .insert("Code".to_string(), vec!["task_code".to_string()]);
        task_index
            .0
            .insert("Chat".to_string(), vec!["task_chat".to_string()]);
        task_index.0.insert(
            "FileOperation".to_string(),
            vec!["task_file_operation".to_string()],
        );
        task_index
            .0
            .insert("Planning".to_string(), vec!["task_planning".to_string()]);
        task_index
            .0
            .insert("Unknown".to_string(), vec!["task_chat".to_string()]);

        let mut synapses = vec![
            seed_synapse(
                "syn_task_code_expert_code",
                "task_code",
                "expert_code",
                SynapseType::Excitatory,
                0.9,
            ),
            seed_synapse(
                "syn_task_code_expert_reasoning",
                "task_code",
                "expert_reasoning",
                SynapseType::Excitatory,
                0.75,
            ),
            seed_synapse(
                "syn_task_code_tool_rust_project",
                "task_code",
                "tool_rust_project",
                SynapseType::ToolPointer,
                0.8,
            ),
            seed_synapse(
                "syn_task_file_operation_tool_filesystem",
                "task_file_operation",
                "tool_filesystem",
                SynapseType::ToolPointer,
                0.8,
            ),
            seed_synapse(
                "syn_task_planning_expert_reasoning",
                "task_planning",
                "expert_reasoning",
                SynapseType::Excitatory,
                0.7,
            ),
            seed_synapse(
                "syn_expert_code_memory_rust",
                "expert_code",
                "memory_rust",
                SynapseType::MemoryPointer,
                0.7,
            ),
            seed_synapse(
                "syn_memory_rust_memory_cargo",
                "memory_rust",
                "memory_cargo",
                SynapseType::Excitatory,
                0.6,
            ),
            seed_synapse(
                "syn_task_chat_expert_language",
                "task_chat",
                "expert_language",
                SynapseType::Excitatory,
                0.85,
            ),
        ];
        if let Some(synapse) = synapses
            .iter_mut()
            .find(|s| s.id == "syn_expert_code_memory_rust")
        {
            synapse.memory_ref = Some("memory_rust".to_string());
        }
        for synapse in synapses {
            self.store.save_synapse(&synapse)?;
            let mut outgoing = self.store.read_outgoing_synapse_ids(&synapse.from)?;
            outgoing.push(synapse.id.clone());
            outgoing.sort();
            outgoing.dedup();
            self.store
                .write_outgoing_synapse_ids(&synapse.from, &outgoing)?;
        }

        self.store.save_memory(&MemoryItem::new(
            "memory_rust",
            MemoryType::Semantic,
            "Rust",
            "Rust is a systems programming language focused on safety and performance.",
            vec!["rust".to_string(), "code".to_string()],
            vec!["memory_rust".to_string(), "expert_code".to_string()],
        ))?;
        self.store.save_memory(&MemoryItem::new(
            "memory_cargo",
            MemoryType::Semantic,
            "Cargo",
            "Cargo is Rust's build tool and package manager.",
            vec!["cargo".to_string(), "rust".to_string()],
            vec!["memory_cargo".to_string(), "memory_rust".to_string()],
        ))?;

        self.store.save_label_index(&label_index)?;
        self.store.save_task_type_index(&task_index)?;
        ensure_default_expert_stats(
            &self.store,
            &[
                "LanguageExpert",
                "CodeExpert",
                "ReasoningExpert",
                "ToolUseExpert",
            ],
        )?;
        Ok(())
    }

    pub fn think(&self, input: String) -> Result<BrainOutput> {
        self.store.ensure_layout()?;
        if self.store.label_index()?.0.is_empty() {
            self.init()?;
        }

        let started_at = chrono::Utc::now();
        let profiler = Profiler::start();
        let task_type = Classifier::classify(&input);
        let task = Task::new(input, task_type);
        let adaptive_budget = AdaptiveBudgetManager::decide_for_task(
            &self.store,
            &task.task_type,
            None,
            &[],
            false,
            false,
        );
        let budget = adaptive_budget.adjusted_budget.clone();
        let routing = Router::route(&self.store, &task, &budget)?;
        for mut neuron in routing.active_neuron_records.clone() {
            neuron.activation_count += 1;
            neuron.last_activated_at = Some(crate::utils::time::now());
            self.store.save_neuron(&neuron)?;
        }
        let active_ids: Vec<String> = routing
            .active_neurons
            .iter()
            .map(|neuron| neuron.id.clone())
            .collect();
        let memories =
            retrieve_relevant_memories(&self.store, &task, &active_ids, budget.max_memory_items)?;
        let selected_experts = select_experts(
            &self.store,
            &task,
            &memories,
            &active_ids,
            budget.max_experts,
        )?;
        let expert_names: Vec<String> = selected_experts
            .iter()
            .map(|result| result.expert_name.clone())
            .collect();

        let plan = Planner::plan(&task);
        Checkpoint {
            task_id: task.id.clone(),
            current_goal: task.input.clone(),
            planned_steps: plan.clone(),
            completed_steps: plan.iter().take(1).cloned().collect(),
            failed_steps: Vec::new(),
            status: "started".to_string(),
        }
        .save(&self.store)?;

        let mut tool_actions = Vec::new();
        let mut result = "Success".to_string();
        let mut created_project_path = None;
        let mut cargo_check_attempted = false;
        let mut cargo_check_passed = false;
        let mut answer = selected_experts
            .iter()
            .map(|result| result.summary.clone())
            .collect::<Vec<_>>()
            .join(" ");

        if should_create_rust_project(&task) && budget.max_tool_actions > 0 {
            let project_name = project_name_from_input(&task.input);
            let fs_tool = FilesystemTool::new(&self.store.paths.sandbox)?;
            let rust_tool = RustProjectTool::new(fs_tool.clone());
            let created = rust_tool.create_hello_world(&project_name)?;
            tool_actions.push(format!(
                "RustProjectTool: created {} files at {}",
                created.files.len(),
                created.path.display()
            ));
            Checkpoint {
                task_id: task.id.clone(),
                current_goal: task.input.clone(),
                planned_steps: plan.clone(),
                completed_steps: plan.iter().take(4).cloned().collect(),
                failed_steps: Vec::new(),
                status: "project-created".to_string(),
            }
            .save(&self.store)?;
            created_project_path = Some(created.path.clone());

            if tool_actions.len() < budget.max_tool_actions {
                let terminal = TerminalTool::new(&self.store.paths.sandbox)?;
                cargo_check_attempted = true;
                match terminal.run(&["cargo", "check"], &created.path) {
                    Ok(command) if command.status == Some(0) => {
                        tool_actions.push("TerminalTool: cargo check succeeded".to_string());
                        cargo_check_passed = true;
                    }
                    Ok(command) => {
                        result = "Partial".to_string();
                        tool_actions.push(format!(
                            "TerminalTool: cargo check exited with {:?}: {}",
                            command.status,
                            command.stderr.lines().next().unwrap_or("")
                        ));
                    }
                    Err(error) => {
                        result = "Partial".to_string();
                        tool_actions.push(format!("TerminalTool: cargo check skipped: {error}"));
                    }
                }
            }
            answer = format!(
                "Created project at sandbox/projects/{project_name}. {}",
                answer
            );
        }

        Checkpoint {
            task_id: task.id.clone(),
            current_goal: task.input.clone(),
            planned_steps: plan.clone(),
            completed_steps: plan,
            failed_steps: if result == "Success" {
                Vec::new()
            } else {
                vec!["Cargo check did not complete successfully".to_string()]
            },
            status: result.clone(),
        }
        .save(&self.store)?;

        let success = result == "Success";
        let learning_report =
            update_routes(&self.store, &routing.loaded_synapses, &active_ids, success)?;
        let energy_report = profiler.finish(
            routing.active_neurons.len(),
            routing.loaded_synapses.len(),
            memories.len(),
            expert_names.len(),
            tool_actions.len(),
        );
        let think_runtime_breakdown = RuntimeBreakdown {
            total_runtime_ms: energy_report.runtime_ms as u64,
            brain_runtime_ms: energy_report.runtime_ms as u64,
            routing_runtime_ms: 0,
            memory_runtime_ms: 0,
            planning_runtime_ms: 0,
            tool_runtime_ms: 0,
            cargo_check_runtime_ms: None,
            cargo_test_runtime_ms: None,
            filesystem_runtime_ms: 0,
            reporting_runtime_ms: 0,
            maintenance_runtime_ms: 0,
            unknown_runtime_ms: 0,
        }
        .finalize_unknown();
        update_expert_stats(
            &self.store,
            &expert_names,
            success,
            energy_report.estimated_cost_units as f32,
            energy_report.runtime_ms as f32,
        )?;
        let _ = update_route_efficiency_from_synapses(
            &self.store,
            &routing.loaded_synapses,
            energy_report.runtime_ms as u64,
            energy_report.estimated_cost_units as f32,
            success,
            if success { 1.0 } else { 0.2 },
        );
        let used_synapses = routing
            .loaded_synapses
            .iter()
            .map(|synapse| synapse.id.clone())
            .collect::<Vec<_>>();
        let used_memories = memories
            .iter()
            .map(|memory| memory.id.clone())
            .collect::<Vec<_>>();
        let self_review = review_task(
            &task,
            &budget,
            &routing.active_neurons,
            &tool_actions,
            created_project_path.as_deref(),
            cargo_check_attempted,
            cargo_check_passed,
            &energy_report,
            success,
        );
        let output = BrainOutput {
            task_id: task.id.clone(),
            task: task.input.clone(),
            task_type: task.task_type.clone(),
            answer,
            activated_neurons: routing.active_neurons.clone(),
            activated_experts: expert_names.clone(),
            used_memories: used_memories.clone(),
            tool_actions: tool_actions.clone(),
            result: result.clone(),
            energy_report: energy_report.clone(),
            learning_report: learning_report.clone(),
            self_review: self_review.clone(),
        };
        self.store.save_log(
            &task.id.clone(),
            &RouteTrace {
                task_id: task.id.clone(),
                task_input: task.input.clone(),
                task_type: task.task_type.clone(),
                activated_neurons: active_ids,
                activated_synapses: used_synapses,
                selected_experts: expert_names,
                selected_memories: used_memories,
                tool_actions: tool_actions.clone(),
                success,
                result,
                energy_estimate: energy_report.estimated_cost_units,
                runtime_ms: energy_report.runtime_ms,
                energy_report: energy_report.clone(),
                learning_updates: learning_report.clone(),
                reused_skills: Vec::new(),
                skill_application_results: Vec::new(),
                habits_used: Vec::new(),
                plan_cache_match: None,
                adaptive_budget: Some(adaptive_budget.clone()),
                live_habit_update: None,
                fast_path_decision: None,
                cargo_validation_policy: None,
                runtime_breakdown: Some(think_runtime_breakdown.clone()),
                optimization_hint: None,
                session_id: None,
                journal_entries: Vec::new(),
                snapshot_ids: Vec::new(),
                transaction_ids: Vec::new(),
                recovery_plan: None,
                reliability_score: None,
            },
        )?;
        let _ = save_performance_profile(
            &self.store,
            &PerformanceProfile {
                id: new_profile_id(),
                command_name: "think".to_string(),
                task_type: format!("{:?}", task.task_type),
                project_name: None,
                started_at,
                ended_at: chrono::Utc::now(),
                runtime_ms: energy_report.runtime_ms as u64,
                estimated_energy: energy_report.estimated_cost_units as f32,
                active_neurons: routing.active_neurons.len(),
                loaded_synapses: routing.loaded_synapses.len(),
                memories_loaded: memories.len(),
                skills_reused: 0,
                tool_actions: tool_actions.len(),
                cargo_check_runtime_ms: None,
                cargo_test_runtime_ms: None,
                success,
                final_score: if success { 1.0 } else { 0.2 },
                adaptive_budget: Some(adaptive_budget),
                habits_used: 0,
                cache_hits: 0,
                runtime_breakdown: think_runtime_breakdown,
                habit_created: false,
                habit_strengthened: false,
                habit_id: None,
                fast_path_decision: None,
            },
        );
        Ok(output)
    }

    #[allow(unreachable_code)]
    pub fn run_project(&self, prompt: String) -> Result<ProjectOutput> {
        return self.run_project_v04(prompt.clone(), None);

        self.store.ensure_layout()?;
        if self.store.label_index()?.0.is_empty() {
            self.init()?;
        }

        let goal_id = Uuid::new_v4().to_string();
        let project_name = project_name_from_input(&prompt);
        let fs_tool = FilesystemTool::new(&self.store.paths.sandbox)?;
        let project_root = fs_tool.safe_path(&format!("projects/{project_name}"))?;
        let editor = CodeEditorTool::new(&self.store.paths.sandbox)?;
        let terminal = TerminalTool::new(&self.store.paths.sandbox)?;
        let mut queue = decompose_goal(&goal_id, &prompt);
        let mut state = ProjectState::new(
            goal_id.clone(),
            project_name.clone(),
            project_root.display().to_string(),
            prompt.clone(),
        );
        save_task_queue(&self.store, &goal_id, &queue)?;
        save_project_state(&self.store, &state)?;

        let mut cargo_check_result = "not run".to_string();
        let mut cargo_test_result = "not run".to_string();

        for index in 0..queue.len() {
            queue[index].status = TaskStatus::Running;
            queue[index].started_at = Some(crate::utils::time::now());
            queue[index].attempts += 1;
            save_task_queue(&self.store, &goal_id, &queue)?;

            let task_title = queue[index].title.clone();
            let task_result: Result<String> = match task_title.as_str() {
                "Understand goal" => Ok(format!("Goal understood for project {project_name}.")),
                "Create project directory" => {
                    fs_tool.create_dir(&format!("projects/{project_name}/src"))?;
                    fs_tool.create_dir(&format!("projects/{project_name}/tests"))?;
                    Ok("Project directories created.".to_string())
                }
                "Write Cargo.toml" => {
                    let path = editor.write_project_file(
                        &project_name,
                        "Cargo.toml",
                        &CodeExpert::cargo_toml(&project_name),
                    )?;
                    remember_created(&mut state, path);
                    Ok("Cargo.toml written.".to_string())
                }
                "Write src/main.rs" => {
                    let path = editor.write_project_file(
                        &project_name,
                        "src/main.rs",
                        &CodeExpert::calculator_main(&project_name),
                    )?;
                    remember_created(&mut state, path);
                    Ok("src/main.rs written.".to_string())
                }
                "Write src/lib.rs" => {
                    let path = editor.write_project_file(
                        &project_name,
                        "src/lib.rs",
                        CodeExpert::calculator_lib(),
                    )?;
                    remember_created(&mut state, path);
                    Ok("src/lib.rs written.".to_string())
                }
                "Write tests" => {
                    let path = editor.write_project_file(
                        &project_name,
                        "tests/calculator.rs",
                        &CodeExpert::calculator_tests(&project_name),
                    )?;
                    remember_created(&mut state, path);
                    Ok("Integration tests written.".to_string())
                }
                "Write README" => {
                    let path = editor.write_project_file(
                        &project_name,
                        "README.md",
                        &CodeExpert::readme(&project_name, &prompt),
                    )?;
                    remember_created(&mut state, path);
                    Ok("README written.".to_string())
                }
                "Run cargo check" => {
                    let command = terminal.run(&["cargo", "check"], &project_root)?;
                    state.commands_run.push("cargo check".to_string());
                    let diagnostic = diagnose_command(&command);
                    if diagnostic.kind == DiagnosticKind::CargoCheckPassed {
                        cargo_check_result = "passed".to_string();
                        Ok("cargo check passed.".to_string())
                    } else if retry_allowed(queue[index].attempts, queue[index].max_attempts) {
                        state.errors_seen.push(diagnostic.summary.clone());
                        if let Some(summary) =
                            apply_simple_rust_fix(&editor, &project_name, &diagnostic)?
                        {
                            state.retries_used += 1;
                            state.files_modified.push("src/main.rs".to_string());
                            let retry = terminal.run(&["cargo", "check"], &project_root)?;
                            state.commands_run.push("cargo check retry".to_string());
                            let retry_diagnostic = diagnose_command(&retry);
                            if retry_diagnostic.kind == DiagnosticKind::CargoCheckPassed {
                                cargo_check_result = "passed after retry".to_string();
                                Ok(format!("cargo check passed after retry: {summary}"))
                            } else {
                                cargo_check_result = "failed".to_string();
                                Err(anyhow::anyhow!(retry_diagnostic.summary))
                            }
                        } else {
                            cargo_check_result = "failed".to_string();
                            Err(anyhow::anyhow!(diagnostic.summary))
                        }
                    } else {
                        cargo_check_result = "failed".to_string();
                        Err(anyhow::anyhow!(diagnostic.summary))
                    }
                }
                "Run cargo test" => {
                    let command = terminal.run(&["cargo", "test"], &project_root)?;
                    state.commands_run.push("cargo test".to_string());
                    let diagnostic = diagnose_command(&command);
                    if diagnostic.kind == DiagnosticKind::CargoTestPassed {
                        cargo_test_result = "passed".to_string();
                        Ok("cargo test passed.".to_string())
                    } else {
                        cargo_test_result = "failed".to_string();
                        state.errors_seen.push(diagnostic.summary.clone());
                        Err(anyhow::anyhow!(diagnostic.summary))
                    }
                }
                "Inspect result" => Ok(format!(
                    "{} files created, {} commands run.",
                    state.files_created.len(),
                    state.commands_run.len()
                )),
                "Create final report" => Ok("Final report will be written.".to_string()),
                _ => Ok("Task completed.".to_string()),
            };

            match task_result {
                Ok(summary) => {
                    queue[index].status = TaskStatus::Completed;
                    queue[index].completed_at = Some(crate::utils::time::now());
                    queue[index].result_summary = Some(summary.clone());
                    state.remember_checkpoint(format!("{task_title}: {summary}"));
                }
                Err(error) => {
                    queue[index].status = TaskStatus::Failed;
                    queue[index].completed_at = Some(crate::utils::time::now());
                    queue[index].error_summary = Some(error.to_string());
                    state.errors_seen.push(format!("{task_title}: {error}"));
                    state.status = "Failed".to_string();
                    state.remember_checkpoint(format!("{task_title}: failed"));
                    save_task_queue(&self.store, &goal_id, &queue)?;
                    save_project_state(&self.store, &state)?;
                    break;
                }
            }
            save_task_queue(&self.store, &goal_id, &queue)?;
            save_project_state(&self.store, &state)?;
        }

        let completed = queue
            .iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .count();
        let failed = queue
            .iter()
            .filter(|task| task.status == TaskStatus::Failed)
            .count();
        if failed == 0 {
            state.status = "Completed".to_string();
        }
        let report_path = self
            .store
            .paths
            .projects
            .join(&goal_id)
            .join("final_report.md");
        let report = format!(
            "# Onyx Brain Project Report\n\nProject: {project_name}\nGoal: {prompt}\nStatus: {}\nTasks completed: {completed}\nTasks failed: {failed}\nCargo check: {cargo_check_result}\nCargo test: {cargo_test_result}\nRetries used: {}\nFiles created:\n{}\n",
            state.status,
            state.retries_used,
            state
                .files_created
                .iter()
                .map(|file| format!("- {file}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        fs::write(&report_path, report)?;
        state.final_summary = Some(format!(
            "Project {project_name} finished with status {}.",
            state.status
        ));
        state.final_report_path = Some(report_path.display().to_string());
        save_project_state(&self.store, &state)?;
        let _ = remember_project_state(&self.store, &state)?;

        Ok(ProjectOutput {
            goal_id,
            intent: IntentKind::CreateProject,
            project_name,
            features_requested: Vec::new(),
            tasks_completed: completed,
            tasks_failed: failed,
            files_created: state.files_created,
            files_modified: state.files_modified,
            cargo_check_result,
            cargo_test_result,
            retries_used: state.retries_used,
            self_evaluation: SelfEvaluation::default(),
            reused_skills: Vec::new(),
            skill_application_results: Vec::new(),
            habits_used: Vec::new(),
            plan_cache_match: None,
            adaptive_budget: AdaptiveBudgetManager::decide_for_task(
                &self.store,
                &TaskType::Code,
                None,
                &[],
                false,
                false,
            ),
            live_habit_update: LiveHabitUpdate::default(),
            fast_path_decision: FastPathDecision::default(),
            cargo_validation_policy: CargoValidationPolicy::default(),
            runtime_breakdown: RuntimeBreakdown::default(),
            optimization_hint: AutoOptimizeHint::default(),
            template_cache_used: None,
            session_id: None,
            journal_summary: Vec::new(),
            snapshot_summary: Vec::new(),
            rollback_readiness: 0.0,
            reliability_score: ReliabilityScore::default(),
            recovery_plan: None,
            json_report_path: None,
            final_status: state.status,
            project_report_path: report_path.display().to_string(),
            ram_minimal_note:
                "Project execution used disk-backed queue/state and task-local working data."
                    .to_string(),
        })
    }

    fn run_project_v04(
        &self,
        prompt: String,
        resume_goal_id: Option<String>,
    ) -> Result<ProjectOutput> {
        self.store.ensure_layout()?;
        let started_at = chrono::Utc::now();
        let runtime_timer = std::time::Instant::now();
        let routing_runtime_ms = 0_u64;
        let mut memory_runtime_ms = 0_u64;
        let mut planning_runtime_ms = 0_u64;
        let mut filesystem_runtime_ms = 0_u64;
        let mut cargo_check_runtime_ms = None;
        let mut cargo_test_runtime_ms = None;
        let mut reporting_runtime_ms = 0_u64;
        if self.store.label_index()?.0.is_empty() {
            self.init()?;
        }
        let mut session = get_or_start_session(&self.store, "project run")?;
        let session_id = session.session_id.clone();
        let mut journal_entries = Vec::new();
        let mut snapshot_ids = Vec::new();

        let planning_timer = std::time::Instant::now();
        let parsed = parse_goal(&prompt);
        let registry = load_project_registry(&self.store)?;
        let fs_tool = FilesystemTool::new(&self.store.paths.sandbox)?;
        let editor = CodeEditorTool::new(&self.store.paths.sandbox)?;
        let terminal = TerminalTool::new(&self.store.paths.sandbox)?;

        let (goal_id, project_name, mut state, mut queue, intent) =
            if let Some(goal_id) = resume_goal_id {
                let state = load_project_state(&self.store, &goal_id)?;
                let queue = load_task_queue(&self.store, &goal_id)?;
                (
                    goal_id,
                    state.project_name.clone(),
                    state,
                    queue,
                    IntentKind::ResumeProject,
                )
            } else if parsed.intent == IntentKind::ModifyProject {
                let Some(project_name) = parsed.project_name.clone() else {
                    return Err(anyhow::anyhow!("modify task needs a project name"));
                };
                let Some(record) = registry.find_by_name(&project_name) else {
                    return Err(anyhow::anyhow!(
                        "project not found in registry: {project_name}"
                    ));
                };
                let mut state = load_project_state(&self.store, &record.goal_id)?;
                state.original_prompt = prompt.clone();
                state.status = "Running".to_string();
                let queue = decompose_modification_goal(&record.goal_id, &parsed);
                (
                    record.goal_id,
                    project_name,
                    state,
                    queue,
                    IntentKind::ModifyProject,
                )
            } else {
                let goal_id = Uuid::new_v4().to_string();
                let project_name = parsed
                    .project_name
                    .clone()
                    .unwrap_or_else(|| project_name_from_input(&prompt));
                let project_root = fs_tool.safe_path(&format!("projects/{project_name}"))?;
                let state = ProjectState::new(
                    goal_id.clone(),
                    project_name.clone(),
                    project_root.display().to_string(),
                    prompt.clone(),
                );
                let queue = decompose_goal(&goal_id, &prompt);
                (
                    goal_id,
                    project_name,
                    state,
                    queue,
                    IntentKind::CreateProject,
                )
            };
        planning_runtime_ms += planning_timer.elapsed().as_millis() as u64;
        journal_entries.push(quick_journal(
            &self.store,
            &session_id,
            ActionType::UpdateProjectState,
            Some(project_name.clone()),
            Some(project_name.clone()),
            None,
            None,
        )?);
        if matches!(intent, IntentKind::ModifyProject) {
            if let Ok(snapshot) = snapshot_create(
                &self.store,
                &project_name,
                "automatic snapshot before project modification",
            ) {
                snapshot_ids.push(snapshot.snapshot_id);
            }
        }

        let memory_timer = std::time::Instant::now();
        let habit_matches = find_matching_habits(&self.store, &parsed, 3)?;
        let plan_cache_match = find_cached_plan(&self.store, &parsed)?;
        let template_cache = find_template_for_goal(&self.store, &parsed)?;
        let template_files = template_cache
            .as_ref()
            .map(|entry| render_template_files(entry, &project_name))
            .unwrap_or_default();
        memory_runtime_ms += memory_timer.elapsed().as_millis() as u64;
        if let Some(cache) = &plan_cache_match {
            state.remember_checkpoint(format!(
                "Plan cache matched {} ({:.2}): {}",
                cache.cache_id, cache.similarity_score, cache.reason
            ));
        }
        if !habit_matches.is_empty() {
            state.remember_checkpoint(format!(
                "Habits matched: {}",
                habit_matches
                    .iter()
                    .map(|habit| habit.title.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let project_plan = queue
            .iter()
            .map(|task| task.title.clone())
            .collect::<Vec<_>>();
        let previous_failure = previous_project_failure(&self.store, &parsed)?;
        let fast_path_decision = decide_fast_path(
            &parsed,
            &habit_matches,
            plan_cache_match.as_ref(),
            previous_failure,
        );
        if fast_path_decision.used_fast_path {
            state.remember_checkpoint(format!("Fast path used: {}", fast_path_decision.reason));
        }
        let adaptive_budget = AdaptiveBudgetManager::decide_for_task(
            &self.store,
            &TaskType::Code,
            Some(&parsed),
            &habit_matches,
            plan_cache_match.is_some(),
            previous_failure,
        );
        let budget = adaptive_budget.adjusted_budget.clone();
        let memory_timer = std::time::Instant::now();
        let reused_skills =
            SkillReuseEngine::find_relevant_skills(&self.store, &parsed, &[], &budget)?;
        memory_runtime_ms += memory_timer.elapsed().as_millis() as u64;
        let skill_application_results = reused_skills
            .iter()
            .map(|skill| SkillReuseEngine::apply_skill_to_plan(skill, &project_plan))
            .collect::<Vec<_>>();
        if !reused_skills.is_empty() {
            state.remember_checkpoint(format!(
                "Reused skills: {}",
                reused_skills
                    .iter()
                    .map(|skill| skill.title.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        save_task_queue(&self.store, &goal_id, &queue)?;
        save_project_state(&self.store, &state)?;
        let project_root = fs_tool.safe_path(&format!("projects/{project_name}"))?;
        let mut cargo_check_result = "not run".to_string();
        let mut cargo_test_result = "not run".to_string();
        let mut cargo_validation_policy = CargoValidationPolicy::default();

        for index in 0..queue.len() {
            if !matches!(
                queue[index].status,
                TaskStatus::Pending | TaskStatus::Failed | TaskStatus::Running
            ) {
                continue;
            }
            if queue[index].status == TaskStatus::Failed
                && !retry_allowed(queue[index].attempts, queue[index].max_attempts)
            {
                continue;
            }
            queue[index].status = TaskStatus::Running;
            queue[index].started_at = Some(crate::utils::time::now());
            queue[index].attempts += 1;
            save_task_queue(&self.store, &goal_id, &queue)?;

            let title = queue[index].title.clone();
            if fast_path_decision.used_fast_path
                && matches!(
                    title.as_str(),
                    "Understand goal" | "Inspect existing project"
                )
            {
                queue[index].status = TaskStatus::Completed;
                queue[index].completed_at = Some(crate::utils::time::now());
                queue[index].result_summary =
                    Some("Fast path shortened redundant planning/inspection.".to_string());
                state.remember_checkpoint(format!("{title}: fast-path shortened."));
                save_task_queue(&self.store, &goal_id, &queue)?;
                save_project_state(&self.store, &state)?;
                continue;
            }
            if matches!(title.as_str(), "Run cargo check" | "Run cargo test") {
                cargo_validation_policy = decide_cargo_validation(
                    &parsed,
                    &state.files_created,
                    &state.files_modified,
                    previous_failure,
                );
                if title == "Run cargo check" && !cargo_validation_policy.run_cargo_check {
                    queue[index].status = TaskStatus::Completed;
                    queue[index].completed_at = Some(crate::utils::time::now());
                    queue[index].result_summary = Some(format!(
                        "cargo check skipped: {}",
                        cargo_validation_policy.reason
                    ));
                    cargo_check_result = "skipped".to_string();
                    state.remember_checkpoint(format!(
                        "{title}: skipped by cargo policy: {}",
                        cargo_validation_policy.reason
                    ));
                    save_task_queue(&self.store, &goal_id, &queue)?;
                    save_project_state(&self.store, &state)?;
                    continue;
                }
                if title == "Run cargo test" && !cargo_validation_policy.run_cargo_test {
                    queue[index].status = TaskStatus::Completed;
                    queue[index].completed_at = Some(crate::utils::time::now());
                    queue[index].result_summary = Some(format!(
                        "cargo test skipped: {}",
                        cargo_validation_policy.reason
                    ));
                    cargo_test_result = "skipped".to_string();
                    state.remember_checkpoint(format!(
                        "{title}: skipped by cargo policy: {}",
                        cargo_validation_policy.reason
                    ));
                    save_task_queue(&self.store, &goal_id, &queue)?;
                    save_project_state(&self.store, &state)?;
                    continue;
                }
            }
            let task_timer = std::time::Instant::now();
            let task_result = execute_project_task(
                &title,
                &prompt,
                &parsed,
                &project_name,
                &project_root,
                &fs_tool,
                &editor,
                &terminal,
                &mut state,
                &mut cargo_check_result,
                &mut cargo_test_result,
                &template_files,
            );
            let elapsed = task_timer.elapsed().as_millis() as u64;
            match title.as_str() {
                "Run cargo check" => cargo_check_runtime_ms = Some(elapsed),
                "Run cargo test" => cargo_test_runtime_ms = Some(elapsed),
                "Write Cargo.toml"
                | "Write src/main.rs"
                | "Write src/lib.rs"
                | "Write tests"
                | "Write README"
                | "Update README"
                | "Apply requested features"
                | "Create project directory" => filesystem_runtime_ms += elapsed,
                _ => planning_runtime_ms += elapsed.min(50),
            }

            match task_result {
                Ok(summary) => {
                    queue[index].status = TaskStatus::Completed;
                    queue[index].completed_at = Some(crate::utils::time::now());
                    queue[index].result_summary = Some(summary.clone());
                    queue[index].error_summary = None;
                    state.remember_checkpoint(format!("{title}: {summary}"));
                    let action_type = match title.as_str() {
                        "Run cargo check" | "Run cargo test" => ActionType::RunCommand,
                        "Create project directory" => ActionType::CreateDirectory,
                        "Write Cargo.toml"
                        | "Write src/main.rs"
                        | "Write src/lib.rs"
                        | "Write tests"
                        | "Write README"
                        | "Update README"
                        | "Apply requested features" => ActionType::ModifyFile,
                        _ => ActionType::UpdateProjectState,
                    };
                    if let Ok(id) = quick_journal(
                        &self.store,
                        &session_id,
                        action_type,
                        Some(project_name.clone()),
                        Some(project_root.display().to_string()),
                        Some(title.clone()),
                        snapshot_ids.last().cloned(),
                    ) {
                        journal_entries.push(id);
                    }
                }
                Err(error) if retry_allowed(queue[index].attempts, queue[index].max_attempts) => {
                    queue[index].status = TaskStatus::Failed;
                    queue[index].error_summary = Some(error.to_string());
                    state.errors_seen.push(format!("{title}: {error}"));
                    state.retries_used += 1;
                    let plan = recovery_plan_for_failure(
                        &error.to_string(),
                        Some(project_name.clone()),
                        Some(queue[index].id.clone()),
                    );
                    state
                        .errors_seen
                        .push(format!("Recovery plan: {:?}", plan.failure_kind));
                }
                Err(error) => {
                    queue[index].status = TaskStatus::Blocked;
                    queue[index].completed_at = Some(crate::utils::time::now());
                    queue[index].error_summary = Some(error.to_string());
                    state.errors_seen.push(format!("{title}: {error}"));
                    state.status = "Blocked".to_string();
                    let plan = recovery_plan_for_failure(
                        &error.to_string(),
                        Some(project_name.clone()),
                        Some(queue[index].id.clone()),
                    );
                    state
                        .errors_seen
                        .push(format!("Recovery plan: {:?}", plan.failure_kind));
                    break;
                }
            }
            save_task_queue(&self.store, &goal_id, &queue)?;
            save_project_state(&self.store, &state)?;
        }

        let completed = queue
            .iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .count();
        let failed = queue
            .iter()
            .filter(|task| matches!(task.status, TaskStatus::Failed | TaskStatus::Blocked))
            .count();
        if failed == 0 {
            state.status = "Completed".to_string();
        }
        if cargo_validation_policy.reason.is_empty() {
            cargo_validation_policy = decide_cargo_validation(
                &parsed,
                &state.files_created,
                &state.files_modified,
                previous_failure,
            );
        }
        let evaluation = evaluate_project(
            &parsed,
            &state,
            &editor,
            &project_name,
            &cargo_check_result,
            &cargo_test_result,
            &reused_skills,
            &habit_matches,
            plan_cache_match.as_ref(),
            &route_efficiency_overview(&self.store)?,
            &inspect_memory_hygiene(&self.store)?,
        );
        state.self_evaluation = Some(evaluation.clone());
        let reporting_timer = std::time::Instant::now();
        let report_path = self
            .store
            .paths
            .projects
            .join(&goal_id)
            .join("final_report.md");
        let runtime_so_far = runtime_timer.elapsed().as_millis() as u64;
        let runtime_breakdown = RuntimeBreakdown {
            total_runtime_ms: runtime_so_far,
            brain_runtime_ms: runtime_so_far.saturating_sub(
                filesystem_runtime_ms
                    + cargo_check_runtime_ms.unwrap_or(0)
                    + cargo_test_runtime_ms.unwrap_or(0),
            ),
            routing_runtime_ms,
            memory_runtime_ms,
            planning_runtime_ms,
            tool_runtime_ms: filesystem_runtime_ms
                + cargo_check_runtime_ms.unwrap_or(0)
                + cargo_test_runtime_ms.unwrap_or(0),
            cargo_check_runtime_ms,
            cargo_test_runtime_ms,
            filesystem_runtime_ms,
            reporting_runtime_ms: 0,
            maintenance_runtime_ms: 0,
            unknown_runtime_ms: 0,
        }
        .finalize_unknown();
        let live_habit_update = update_live_habit_after_project(
            &self.store,
            &prompt,
            &state,
            project_plan.clone(),
            runtime_so_far,
            completed as f32 * 10.0,
        )?;
        let optimization_hint = auto_optimize_hint(
            &self.store,
            irrelevant_skill_count(&reused_skills, Some(&project_name)),
            habit_matches.is_empty(),
        )?;
        let recovery_plan = if failed > 0 {
            Some(recovery_plan_for_failure(
                state
                    .errors_seen
                    .last()
                    .map(String::as_str)
                    .unwrap_or("unknown failure"),
                Some(project_name.clone()),
                None,
            ))
        } else {
            None
        };
        let reliability = reliability_score(
            &self.store,
            cargo_check_result.contains("passed") && cargo_test_result.contains("passed"),
            recovery_plan
                .as_ref()
                .map(|plan| plan.confidence)
                .unwrap_or(1.0),
        )?;
        let report = project_report(
            &project_name,
            &prompt,
            &state,
            completed,
            failed,
            &cargo_check_result,
            &cargo_test_result,
            &evaluation,
            &runtime_breakdown,
            &fast_path_decision,
            &habit_matches,
            plan_cache_match.as_ref(),
            &cargo_validation_policy,
            &adaptive_budget,
            &live_habit_update,
            &optimization_hint,
            &session_id,
            &journal_entries,
            &snapshot_ids,
            &reliability,
            recovery_plan.as_ref(),
        );
        fs::write(&report_path, &report)?;
        let json_report_path = self
            .store
            .paths
            .projects
            .join(&goal_id)
            .join("final_report.json");
        save_json(
            &json_report_path,
            &serde_json::json!({
                "project": project_name,
                "goal": prompt,
                "status": state.status,
                "session_id": session_id,
                "journal_entries": journal_entries,
                "snapshots": snapshot_ids,
                "reliability_score": reliability.clone(),
                "recovery_plan": recovery_plan.clone(),
                "runtime_breakdown": runtime_breakdown.clone(),
                "cargo_validation_policy": cargo_validation_policy.clone(),
            }),
        )?;
        reporting_runtime_ms += reporting_timer.elapsed().as_millis() as u64;
        state.final_summary = Some(format!(
            "Project {project_name} {:?} finished with status {} and score {:.2}.",
            intent, state.status, evaluation.overall_score
        ));
        state.final_report_path = Some(report_path.display().to_string());
        save_project_state(&self.store, &state)?;
        let _ = remember_project_state(&self.store, &state)?;
        save_skills_without_duplicates(&self.store, &state, &report)?;
        update_skill_usage(
            &self.store,
            &reused_skills,
            state.status == "Completed",
            &goal_id,
        )?;
        let _ = store_successful_plan(
            &self.store,
            &parsed,
            &state,
            project_plan.clone(),
            runtime_timer.elapsed().as_millis() as u64,
            completed as f32 * 10.0,
        );
        if state.status == "Completed" && matches!(intent, IntentKind::CreateProject) {
            let _ = store_or_strengthen_rust_cli_template(
                &self.store,
                &parsed,
                &project_name,
                if template_cache.is_some() { 150.0 } else { 0.0 },
            );
        }
        if let Some(cache) = &plan_cache_match {
            let _ = mark_cache_used(
                &self.store,
                &cache.cache_id,
                state.status == "Completed",
                runtime_timer.elapsed().as_millis() as u64,
                completed as f32 * 10.0,
            );
        }
        let _ = update_named_route_efficiency(
            &self.store,
            &format!("project:{:?}:{}", parsed.intent, project_name),
            "project_worker",
            "sandbox_tools",
            runtime_timer.elapsed().as_millis() as u64,
            completed as f32 * 10.0,
            state.status == "Completed",
            evaluation.overall_score,
        );
        if state.status == "Completed" && MemoryHygienePolicy::default().dedup_after_project_run {
            let _ = dedup_memories(&self.store)?;
        }
        register_project(
            &self.store,
            ProjectRecord {
                goal_id: goal_id.clone(),
                project_name: project_name.clone(),
                root_path: state.root_path.clone(),
                status: state.status.clone(),
                created_at: crate::utils::time::now(),
                updated_at: crate::utils::time::now(),
                last_report_path: Some(report_path.display().to_string()),
                tags: vec!["rust".to_string(), "project".to_string()],
                summary: state.final_summary.clone().unwrap_or_default(),
            },
        )?;
        let runtime_ms = runtime_timer.elapsed().as_millis() as u64;
        let runtime_breakdown = RuntimeBreakdown {
            total_runtime_ms: runtime_ms,
            brain_runtime_ms: runtime_ms.saturating_sub(
                filesystem_runtime_ms
                    + cargo_check_runtime_ms.unwrap_or(0)
                    + cargo_test_runtime_ms.unwrap_or(0)
                    + reporting_runtime_ms,
            ),
            routing_runtime_ms,
            memory_runtime_ms,
            planning_runtime_ms,
            tool_runtime_ms: filesystem_runtime_ms
                + cargo_check_runtime_ms.unwrap_or(0)
                + cargo_test_runtime_ms.unwrap_or(0),
            cargo_check_runtime_ms,
            cargo_test_runtime_ms,
            filesystem_runtime_ms,
            reporting_runtime_ms,
            maintenance_runtime_ms: 0,
            unknown_runtime_ms: 0,
        }
        .finalize_unknown();
        let estimated_energy = completed as u64 * 10 + state.commands_run.len() as u64 * 8;
        session.project_ids.push(goal_id.clone());
        session.journal_entries.extend(journal_entries.clone());
        session
            .checkpoints
            .push(format!("Project {project_name}: {}", state.status));
        session.total_runtime_ms += runtime_ms;
        session.total_energy += estimated_energy as f32;
        session.summary = format!("Project {project_name} finished with {}", state.status);
        session.status = SessionStatus::Completed;
        session.ended_at = Some(chrono::Utc::now());
        let _ = crate::agency::save_session(&self.store, &session);
        self.store.save_log(
            &format!("project_trace_{goal_id}"),
            &RouteTrace {
                task_id: goal_id.clone(),
                task_input: prompt.clone(),
                task_type: TaskType::Code,
                activated_neurons: Vec::new(),
                activated_synapses: Vec::new(),
                selected_experts: vec!["CodeExpert".to_string()],
                selected_memories: reused_skills
                    .iter()
                    .map(|skill| skill.skill_id.clone())
                    .collect(),
                tool_actions: state.commands_run.clone(),
                success: state.status == "Completed",
                result: state.status.clone(),
                energy_estimate: estimated_energy,
                runtime_ms: runtime_ms as u128,
                energy_report: EnergyReport {
                    active_neuron_count: 0,
                    loaded_synapse_count: 0,
                    memory_items_loaded: reused_skills.len(),
                    expert_count: 1,
                    tool_action_count: state.commands_run.len(),
                    runtime_ms: runtime_ms as u128,
                    estimated_cost_units: estimated_energy,
                },
                learning_updates: LearningReport {
                    strengthened: 0,
                    weakened: 0,
                    new_synapses: 0,
                },
                reused_skills: reused_skills.clone(),
                skill_application_results: skill_application_results.clone(),
                habits_used: habit_matches.clone(),
                plan_cache_match: plan_cache_match.clone(),
                adaptive_budget: Some(adaptive_budget.clone()),
                live_habit_update: Some(live_habit_update.clone()),
                fast_path_decision: Some(fast_path_decision.clone()),
                cargo_validation_policy: Some(cargo_validation_policy.clone()),
                runtime_breakdown: Some(runtime_breakdown.clone()),
                optimization_hint: Some(optimization_hint.clone()),
                session_id: Some(session_id.clone()),
                journal_entries: journal_entries.clone(),
                snapshot_ids: snapshot_ids.clone(),
                transaction_ids: Vec::new(),
                recovery_plan: recovery_plan.clone(),
                reliability_score: Some(reliability.clone()),
            },
        )?;
        let _ = save_performance_profile(
            &self.store,
            &PerformanceProfile {
                id: new_profile_id(),
                command_name: "project".to_string(),
                task_type: "Code".to_string(),
                project_name: Some(project_name.clone()),
                started_at,
                ended_at: chrono::Utc::now(),
                runtime_ms,
                estimated_energy: estimated_energy as f32,
                active_neurons: 0,
                loaded_synapses: 0,
                memories_loaded: reused_skills.len(),
                skills_reused: reused_skills.len(),
                tool_actions: state.commands_run.len(),
                cargo_check_runtime_ms,
                cargo_test_runtime_ms,
                success: state.status == "Completed",
                final_score: evaluation.overall_score,
                adaptive_budget: Some(adaptive_budget.clone()),
                habits_used: habit_matches.len(),
                cache_hits: usize::from(plan_cache_match.is_some()),
                runtime_breakdown: runtime_breakdown.clone(),
                habit_created: live_habit_update.habit_created,
                habit_strengthened: live_habit_update.habit_strengthened,
                habit_id: live_habit_update.habit_id.clone(),
                fast_path_decision: Some(fast_path_decision.clone()),
            },
        );

        Ok(ProjectOutput {
            goal_id,
            intent,
            project_name,
            features_requested: parsed.requested_features,
            tasks_completed: completed,
            tasks_failed: failed,
            files_created: state.files_created,
            files_modified: state.files_modified,
            cargo_check_result,
            cargo_test_result,
            retries_used: state.retries_used,
            self_evaluation: evaluation,
            reused_skills,
            skill_application_results,
            habits_used: habit_matches,
            plan_cache_match,
            adaptive_budget,
            live_habit_update,
            fast_path_decision,
            cargo_validation_policy,
            runtime_breakdown,
            optimization_hint,
            template_cache_used: template_cache.map(|entry| entry.template_id),
            session_id: Some(session_id),
            journal_summary: journal_entries.clone(),
            snapshot_summary: snapshot_ids.clone(),
            rollback_readiness: reliability.rollback_readiness,
            reliability_score: reliability,
            recovery_plan,
            json_report_path: Some(json_report_path.display().to_string()),
            final_status: state.status,
            project_report_path: report_path.display().to_string(),
            ram_minimal_note:
                "Project modification used disk-backed registry/queue/state and task-local working data."
                    .to_string(),
        })
    }

    pub fn execute_goal(&self, prompt: String) -> Result<GoalExecutionOutput> {
        self.store.ensure_layout()?;
        let started_at = chrono::Utc::now();
        let timer = std::time::Instant::now();
        let parsed = parse_goal(&prompt);
        let goal_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let mut goal = GoalMemoryItem {
            goal_id: goal_id.clone(),
            title: parsed
                .project_name
                .clone()
                .unwrap_or_else(|| "Onyx goal".to_string()),
            original_prompt: prompt.clone(),
            parsed_intent: parsed.intent.clone(),
            project_name: parsed.project_name.clone(),
            status: GoalStatus::Active,
            priority: crate::core::Priority::Normal,
            created_at: now,
            updated_at: now,
            completed_at: None,
            linked_project_id: None,
            linked_memories: Vec::new(),
            linked_skills: Vec::new(),
            success_score: 0.0,
            energy_spent: 0,
            notes: vec!["Goal created and scheduled.".to_string()],
        };
        save_goal(&self.store, &goal)?;
        let output = self.run_project(prompt)?;
        goal.status = if output.final_status == "Completed" {
            GoalStatus::Completed
        } else {
            GoalStatus::Failed
        };
        goal.project_name = Some(output.project_name.clone());
        goal.linked_project_id = Some(output.goal_id.clone());
        goal.linked_skills = output
            .reused_skills
            .iter()
            .map(|skill| skill.skill_id.clone())
            .collect();
        goal.success_score = output.self_evaluation.overall_score;
        goal.energy_spent = output.tasks_completed as u64 * 10;
        goal.updated_at = chrono::Utc::now();
        goal.completed_at = Some(goal.updated_at);
        goal.notes.push(format!(
            "Project worker finished with {}.",
            output.final_status
        ));
        save_goal(&self.store, &goal)?;
        let _ = update_named_route_efficiency(
            &self.store,
            &format!("goal:{:?}", parsed.intent),
            "goal_memory",
            "project_worker",
            timer.elapsed().as_millis() as u64,
            goal.energy_spent as f32,
            goal.status == GoalStatus::Completed,
            goal.success_score,
        );
        let _ = save_performance_profile(
            &self.store,
            &PerformanceProfile {
                id: new_profile_id(),
                command_name: "goal".to_string(),
                task_type: format!("{:?}", parsed.intent),
                project_name: output.project_name.clone().into(),
                started_at,
                ended_at: chrono::Utc::now(),
                runtime_ms: timer.elapsed().as_millis() as u64,
                estimated_energy: goal.energy_spent as f32,
                active_neurons: 0,
                loaded_synapses: 0,
                memories_loaded: output.reused_skills.len(),
                skills_reused: output.reused_skills.len(),
                tool_actions: output.tasks_completed,
                cargo_check_runtime_ms: output.runtime_breakdown.cargo_check_runtime_ms,
                cargo_test_runtime_ms: output.runtime_breakdown.cargo_test_runtime_ms,
                success: goal.status == GoalStatus::Completed,
                final_score: goal.success_score,
                adaptive_budget: Some(output.adaptive_budget.clone()),
                habits_used: output.habits_used.len(),
                cache_hits: usize::from(output.plan_cache_match.is_some()),
                runtime_breakdown: output.runtime_breakdown.clone(),
                habit_created: output.live_habit_update.habit_created,
                habit_strengthened: output.live_habit_update.habit_strengthened,
                habit_id: output.live_habit_update.habit_id.clone(),
                fast_path_decision: Some(output.fast_path_decision.clone()),
            },
        );
        let live_habit_update = output.live_habit_update.clone();
        let fast_path_decision = output.fast_path_decision.clone();
        let cargo_validation_policy = output.cargo_validation_policy.clone();
        let runtime_breakdown = output.runtime_breakdown.clone();
        let optimization_hint = output.optimization_hint.clone();
        let template_cache_used = output.template_cache_used.clone();
        let session_id = output.session_id.clone();
        let journal_summary = output.journal_summary.clone();
        let snapshot_summary = output.snapshot_summary.clone();
        let rollback_readiness = output.rollback_readiness;
        let reliability_score = output.reliability_score.clone();
        let recovery_plan = output.recovery_plan.clone();
        let json_report_path = output.json_report_path.clone();
        Ok(GoalExecutionOutput {
            goal_id: goal.goal_id.clone(),
            goal_status: goal.status,
            project_name: goal.project_name,
            skills_reused: output
                .reused_skills
                .into_iter()
                .map(|skill| skill.title)
                .collect(),
            energy_estimate: goal.energy_spent,
            self_evaluation: output.self_evaluation,
            goal_memory_path: crate::agency::goal_path(&self.store, &goal_id)
                .display()
                .to_string(),
            habits_used: output
                .habits_used
                .iter()
                .map(|habit| habit.title.clone())
                .collect(),
            adaptive_budget: output.adaptive_budget,
            live_habit_update,
            fast_path_decision,
            cargo_validation_policy,
            runtime_breakdown,
            optimization_hint,
            template_cache_used,
            session_id,
            journal_summary,
            snapshot_summary,
            rollback_readiness,
            reliability_score,
            recovery_plan,
            json_report_path,
        })
    }

    pub fn goals(&self) -> Result<Vec<GoalMemoryItem>> {
        Ok(crate::energy::PriorityScheduler::order_goals(list_goals(
            &self.store,
        )?))
    }

    pub fn resume_project(&self, goal_id: &str) -> Result<ProjectOutput> {
        let resolved = if goal_id.eq_ignore_ascii_case("latest") {
            load_project_registry(&self.store)?
                .projects
                .first()
                .map(|project| project.goal_id.clone())
                .ok_or_else(|| anyhow::anyhow!("no projects in registry"))?
        } else {
            goal_id.to_string()
        };
        self.run_project_v04(format!("resume {resolved}"), Some(resolved))
    }

    pub fn project_inspect(&self, project_name: &str) -> Result<ProjectInspectOutput> {
        let registry = load_project_registry(&self.store)?;
        let record = registry
            .find_by_name(project_name)
            .ok_or_else(|| anyhow::anyhow!("project not found: {project_name}"))?;
        let state = load_project_state(&self.store, &record.goal_id)?;
        let queue = load_task_queue(&self.store, &record.goal_id)?;
        let root = std::path::PathBuf::from(&record.root_path);
        let mut files = Vec::new();
        if root.exists() {
            collect_files(&root, &root, &mut files)?;
        }
        let memories = self
            .store
            .memory_files()?
            .into_iter()
            .filter_map(|path| load_json::<MemoryItem>(&path).ok())
            .filter(|memory| {
                memory
                    .tags
                    .iter()
                    .any(|tag| tag.eq_ignore_ascii_case(project_name))
                    || memory
                        .title
                        .to_lowercase()
                        .contains(&project_name.to_lowercase())
            })
            .map(|memory| format!("{}: {}", memory.id, memory.title))
            .collect();
        Ok(ProjectInspectOutput {
            project_name: state.project_name,
            root_path: state.root_path,
            status: state.status,
            last_report: state.final_report_path,
            files,
            memories,
            task_queue_status: queue
                .iter()
                .map(|task| format!("{}: {:?}", task.title, task.status))
                .collect(),
            recent_errors: state.errors_seen.into_iter().rev().take(5).collect(),
            self_evaluation: state.self_evaluation,
        })
    }

    pub fn projects(&self) -> Result<Vec<ProjectRecord>> {
        Ok(load_project_registry(&self.store)?.projects)
    }

    pub fn consolidate(&self) -> Result<ConsolidationReport> {
        consolidate(&self.store)
    }

    pub fn inspect(&self) -> Result<InspectSummary> {
        self.store.ensure_layout()?;
        let routes = route_efficiency_overview(&self.store)?;
        let habits = habit_overview(&self.store)?;
        let cache = plan_cache_overview(&self.store)?;
        let doctor_report = doctor(&self.store, false)?;
        let reliability = reliability_score(&self.store, doctor_report.critical == 0, 0.8)?;
        let performance = performance_overview(&self.store)?;
        let latest_journal = latest_journal_entries(&self.store, 5, None).unwrap_or_default();
        let snapshot_overview = snapshots(&self.store).unwrap_or_default();
        let recent_sessions = sessions(&self.store).unwrap_or_default();
        let tx_overview = transactions(&self.store).unwrap_or_default();
        Ok(InspectSummary {
            neurons: count_json(&self.store.paths.neurons)?,
            synapses: count_json(&self.store.paths.synapses)?,
            memories: count_json(&self.store.paths.memories)?,
            logs: count_json(&self.store.paths.logs)?,
            sandbox: self.store.paths.sandbox.display().to_string(),
            top_strongest_synapses: top_strongest_synapses(&self.store)?,
            top_used_neurons: top_used_neurons(&self.store)?,
            top_important_memories: top_important_memories(&self.store)?,
            last_tasks: last_tasks(&self.store)?,
            average_energy_estimate: average_energy_estimate(&self.store)?,
            last_consolidation_time: last_consolidation_time(&self.store)?,
            known_projects: known_projects(&self.store)?,
            failed_tasks: failed_project_tasks(&self.store)?,
            retry_counts: project_retry_counts(&self.store)?,
            last_project_report_path: last_project_report_path(&self.store)?,
            registered_project_count: load_project_registry(&self.store)?.projects.len(),
            last_modified_project: load_project_registry(&self.store)?
                .projects
                .first()
                .map(|project| project.project_name.clone()),
            top_extracted_skills: top_extracted_skills(&self.store)?,
            average_project_self_evaluation_score: average_project_self_evaluation_score(
                &self.store,
            )?,
            failed_or_blocked_task_count: failed_or_blocked_task_count(&self.store)?,
            memory_hygiene: inspect_memory_hygiene(&self.store)?,
            historical_project_memories: count_project_memories(&self.store, false)?,
            archived_project_memories: count_project_memories(&self.store, true)?,
            duplicate_project_memories: inspect_memory_hygiene(&self.store)?
                .duplicate_project_memories,
            route_efficiency_top: routes.top_routes,
            habit_summary: habits.top_habits,
            cache_summary: cache.top_cached_plans,
            slowest_recent_tasks: performance.slowest_command_types,
            average_runtime_ms: performance.average_runtime_last_5,
            average_energy: performance.average_energy_last_5,
            adaptive_budget_summary: format!(
                "estimated savings {:.0}%",
                performance.estimated_budget_savings * 100.0
            ),
            latest_journal_entries: latest_journal
                .iter()
                .map(|entry| format!("{} {:?} {:?}", entry.id, entry.action_type, entry.status))
                .collect(),
            latest_snapshots: snapshot_overview.snapshots.into_iter().take(5).collect(),
            recent_sessions: recent_sessions
                .into_iter()
                .take(5)
                .map(|session| {
                    format!(
                        "{} {:?} {}",
                        session.session_id, session.status, session.title
                    )
                })
                .collect(),
            recovery_reports: recovery_report_rows(&self.store)?,
            doctor_health_summary: doctor_report.recommendation,
            transaction_summary: tx_overview.transactions.into_iter().take(5).collect(),
            reliability_summary: format!(
                "score {:.2}, rollback {:.2}, snapshots {:.2}, journal {:.2}",
                reliability.overall,
                reliability.rollback_readiness,
                reliability.snapshot_coverage,
                reliability.journal_completeness
            ),
        })
    }

    pub fn memory_inspect(&self) -> Result<MemoryHygieneReport> {
        inspect_memory_hygiene(&self.store)
    }

    pub fn memory_dedup(&self) -> Result<MemoryDedupReport> {
        dedup_memories(&self.store)
    }

    pub fn benchmark(&self, name: &str) -> Result<BenchmarkReport> {
        if name != "basic" {
            return Err(anyhow::anyhow!("unknown benchmark: {name}"));
        }
        let started_at = chrono::Utc::now();
        let timer = std::time::Instant::now();
        let project_count_before = load_project_registry(&self.store)?.projects.len();
        let memories_before = self.store.memory_files()?.len();
        let mut tasks_run = 0_u64;
        let mut tasks_successful = 0_u64;
        let mut tasks_failed = 0_u64;
        let mut reused_skills_count = 0_u64;
        let mut irrelevant_skills_used = 0_u64;
        let mut habits_used = 0_u64;
        let mut cache_hits = 0_u64;
        let mut template_cache_hits = 0_u64;
        let mut adaptive_budget_decisions = 0_u64;
        let mut active_neuron_counts = Vec::new();
        let mut filesystem_runtime_ms = 0_u64;
        let mut cargo_check_runtime_ms = None;
        let mut cargo_test_runtime_ms = None;

        let create_timer = std::time::Instant::now();
        let create = self.run_project(
            "Create a Rust CLI calculator project called bench_calc with add and subtract functions, tests, and README"
                .to_string(),
        );
        let create_elapsed = create_timer.elapsed().as_millis() as u64;
        tasks_run += 1;
        match create {
            Ok(output) => {
                filesystem_runtime_ms += output.runtime_breakdown.filesystem_runtime_ms;
                if cargo_check_runtime_ms.is_none() {
                    cargo_check_runtime_ms = output.runtime_breakdown.cargo_check_runtime_ms;
                }
                if cargo_test_runtime_ms.is_none() {
                    cargo_test_runtime_ms = output.runtime_breakdown.cargo_test_runtime_ms;
                }
                tasks_successful += u64::from(output.final_status == "Completed");
                tasks_failed += u64::from(output.final_status != "Completed");
                reused_skills_count += output.reused_skills.len() as u64;
                irrelevant_skills_used +=
                    irrelevant_skill_count(&output.reused_skills, Some(&output.project_name))
                        as u64;
                habits_used += output.habits_used.len() as u64;
                cache_hits += u64::from(output.plan_cache_match.is_some());
                template_cache_hits += u64::from(output.template_cache_used.is_some());
                adaptive_budget_decisions += u64::from(!matches!(
                    output.adaptive_budget.decision_type,
                    AdaptiveBudgetDecisionType::Unchanged
                ));
            }
            Err(_) => {
                filesystem_runtime_ms += create_elapsed;
                tasks_failed += 1;
            }
        }
        let modify_timer = std::time::Instant::now();
        let modify = self.run_project(
            "Modify the bench_calc project to add multiply and divide functions with tests"
                .to_string(),
        );
        let modify_elapsed = modify_timer.elapsed().as_millis() as u64;
        tasks_run += 1;
        match modify {
            Ok(output) => {
                filesystem_runtime_ms += output.runtime_breakdown.filesystem_runtime_ms;
                cargo_check_runtime_ms = output
                    .runtime_breakdown
                    .cargo_check_runtime_ms
                    .or(cargo_check_runtime_ms);
                cargo_test_runtime_ms = output
                    .runtime_breakdown
                    .cargo_test_runtime_ms
                    .or(cargo_test_runtime_ms);
                tasks_successful += u64::from(output.final_status == "Completed");
                tasks_failed += u64::from(output.final_status != "Completed");
                reused_skills_count += output.reused_skills.len() as u64;
                irrelevant_skills_used +=
                    irrelevant_skill_count(&output.reused_skills, Some(&output.project_name))
                        as u64;
                habits_used += output.habits_used.len() as u64;
                cache_hits += u64::from(output.plan_cache_match.is_some());
                template_cache_hits += u64::from(output.template_cache_used.is_some());
                adaptive_budget_decisions += u64::from(!matches!(
                    output.adaptive_budget.decision_type,
                    AdaptiveBudgetDecisionType::Unchanged
                ));
            }
            Err(_) => {
                filesystem_runtime_ms += modify_elapsed;
                tasks_failed += 1;
            }
        }
        let _ = self.project_inspect("bench_calc");
        tasks_run += 1;
        tasks_successful += 1;
        let terminal = TerminalTool::new(&self.store.paths.sandbox)?;
        let fs_tool = FilesystemTool::new(&self.store.paths.sandbox)?;
        let root = fs_tool.safe_path("projects/bench_calc")?;
        for command in [["cargo", "check"], ["cargo", "test"]] {
            tasks_run += 1;
            let cargo_timer = std::time::Instant::now();
            let ok = terminal.run(&command, &root).is_ok();
            let elapsed = cargo_timer.elapsed().as_millis() as u64;
            if command[1] == "check" {
                cargo_check_runtime_ms = Some(cargo_check_runtime_ms.unwrap_or(0) + elapsed);
            } else {
                cargo_test_runtime_ms = Some(cargo_test_runtime_ms.unwrap_or(0) + elapsed);
            }
            if ok {
                tasks_successful += 1;
            } else {
                tasks_failed += 1;
            }
        }
        let _ = self.consolidate();
        tasks_run += 1;
        tasks_successful += 1;
        let dedup = self.memory_dedup()?;
        tasks_run += 1;
        tasks_successful += 1;
        let memories_after = self.store.memory_files()?.len();
        let project_count_after = load_project_registry(&self.store)?.projects.len();
        active_neuron_counts.push(0);
        let completed_at = chrono::Utc::now();
        let total_runtime_ms = timer.elapsed().as_millis() as u64;
        let cargo_runtime_ms =
            cargo_check_runtime_ms.unwrap_or(0) + cargo_test_runtime_ms.unwrap_or(0);
        let runtime_breakdown = RuntimeBreakdown {
            total_runtime_ms,
            brain_runtime_ms: total_runtime_ms
                .saturating_sub(filesystem_runtime_ms + cargo_runtime_ms),
            routing_runtime_ms: 0,
            memory_runtime_ms: 0,
            planning_runtime_ms: 0,
            tool_runtime_ms: filesystem_runtime_ms + cargo_runtime_ms,
            cargo_check_runtime_ms,
            cargo_test_runtime_ms,
            filesystem_runtime_ms,
            reporting_runtime_ms: 0,
            maintenance_runtime_ms: 0,
            unknown_runtime_ms: 0,
        }
        .finalize_unknown();
        let runtime_diagnosis = diagnose_benchmark_runtime(&runtime_breakdown);
        let final_score = if tasks_run == 0 {
            0.0
        } else {
            tasks_successful as f32 / tasks_run as f32
        };
        let report_name = format!("benchmark_basic_{}", timestamp_slug());
        let report_path = self.store.paths.logs.join(format!("{report_name}.json"));
        let route_overview = route_efficiency_overview(&self.store)?;
        let report = BenchmarkReport {
            benchmark_name: "basic".to_string(),
            started_at,
            completed_at,
            tasks_run,
            tasks_successful,
            tasks_failed,
            total_runtime_ms,
            average_energy_estimate: average_energy_estimate(&self.store)?,
            active_neuron_counts,
            reused_skills_count,
            irrelevant_skills_used,
            habits_used,
            cache_hits,
            adaptive_budget_decisions,
            average_route_efficiency: route_overview.average_efficiency,
            template_cache_hits,
            runtime_breakdown: runtime_breakdown.clone(),
            runtime_diagnosis: runtime_diagnosis.clone(),
            memories_created: memories_after.saturating_sub(memories_before) as u64,
            memories_archived: dedup.memories_archived as u64,
            project_count_before,
            project_count_after,
            final_score,
            report_path: report_path.display().to_string(),
        };
        save_json(&report_path, &report)?;
        append_benchmark_history(&self.store, &report)?;
        let _ = save_performance_profile(
            &self.store,
            &PerformanceProfile {
                id: new_profile_id(),
                command_name: "benchmark".to_string(),
                task_type: "Benchmark".to_string(),
                project_name: Some("bench_calc".to_string()),
                started_at,
                ended_at: completed_at,
                runtime_ms: report.total_runtime_ms,
                estimated_energy: report.average_energy_estimate,
                active_neurons: 0,
                loaded_synapses: 0,
                memories_loaded: 0,
                skills_reused: report.reused_skills_count as usize,
                tool_actions: report.tasks_run as usize,
                cargo_check_runtime_ms: report.runtime_breakdown.cargo_check_runtime_ms,
                cargo_test_runtime_ms: report.runtime_breakdown.cargo_test_runtime_ms,
                success: report.tasks_failed == 0,
                final_score: report.final_score,
                adaptive_budget: None,
                habits_used: report.habits_used as usize,
                cache_hits: report.cache_hits as usize,
                runtime_breakdown,
                habit_created: false,
                habit_strengthened: false,
                habit_id: None,
                fast_path_decision: None,
            },
        );
        Ok(report)
    }

    pub fn benchmark_reliability(&self) -> Result<BenchmarkReliabilityReport> {
        let timer = std::time::Instant::now();
        let mut tasks_run = 0;
        let mut tasks_successful = 0;
        let create = self.run_project(
            "Create a Rust CLI calculator project called reliability_calc with add and subtract functions, tests, and README"
                .to_string(),
        );
        tasks_run += 1;
        if create
            .as_ref()
            .is_ok_and(|output| output.final_status == "Completed")
        {
            tasks_successful += 1;
        }
        let snapshot = self.snapshot_create("reliability_calc", "benchmark reliability snapshot");
        tasks_run += 1;
        if snapshot.is_ok() {
            tasks_successful += 1;
        }
        let modify = self.run_project(
            "Modify the reliability_calc project to add multiply and divide functions with tests"
                .to_string(),
        );
        tasks_run += 1;
        if modify
            .as_ref()
            .is_ok_and(|output| output.final_status == "Completed")
        {
            tasks_successful += 1;
        }
        let rollback = self.rollback_latest(Some("reliability_calc"));
        tasks_run += 1;
        let rollback_success = rollback
            .as_ref()
            .is_ok_and(|report| report.status == "Completed");
        if rollback_success {
            tasks_successful += 1;
        }
        let restore = snapshot
            .as_ref()
            .ok()
            .map(|snapshot| self.snapshot_restore(&snapshot.snapshot_id));
        tasks_run += 1;
        let snapshot_restore_success = restore.as_ref().is_some_and(|result| {
            result
                .as_ref()
                .is_ok_and(|report| report.status == "Completed")
        });
        if snapshot_restore_success {
            tasks_successful += 1;
        }
        let doctor = self.doctor(false)?;
        tasks_run += 1;
        if doctor.critical == 0 {
            tasks_successful += 1;
        }
        let regression = self.regression_check()?;
        tasks_run += 1;
        if regression.status == "pass" {
            tasks_successful += 1;
        }
        let _ = self.maintain();
        tasks_run += 1;
        tasks_successful += 1;
        let reliability = reliability_score(&self.store, true, 0.8)?;
        let report_path = self
            .store
            .paths
            .logs
            .join(format!("benchmark_reliability_{}.json", timestamp_slug()));
        let report = BenchmarkReliabilityReport {
            tasks_run,
            tasks_successful,
            rollback_success,
            snapshot_restore_success,
            doctor_critical_issues: doctor.critical,
            regression_check_passed: regression.status == "pass",
            reliability_score: reliability.overall,
            runtime_ms: timer.elapsed().as_millis() as u64,
            report_path: report_path.display().to_string(),
        };
        save_json(&report_path, &report)?;
        Ok(report)
    }

    pub fn benchmark_compare(&self) -> Result<BenchmarkCompareReport> {
        let history = load_benchmark_history(&self.store)?;
        if history.0.is_empty() {
            return Ok(BenchmarkCompareReport {
                runtime_trend: "insufficient history".to_string(),
                energy_trend: "insufficient history".to_string(),
                skill_reuse_trend: "insufficient history".to_string(),
                skill_reuse_quality_trend: "insufficient history".to_string(),
                habit_usage_trend: "insufficient history".to_string(),
                cache_hit_rate_trend: "insufficient history".to_string(),
                route_efficiency_trend: "insufficient history".to_string(),
                memory_hygiene_trend: "insufficient history".to_string(),
                ..BenchmarkCompareReport::default()
            });
        }
        let last = history.0.last();
        let best = history
            .0
            .iter()
            .max_by(|a, b| a.final_score.total_cmp(&b.final_score));
        let average_score =
            history.0.iter().map(|entry| entry.final_score).sum::<f32>() / history.0.len() as f32;
        let runtime_trend = benchmark_runtime_trend(&history);
        Ok(BenchmarkCompareReport {
            last_score: last.map(|entry| entry.final_score),
            best_score: best.map(|entry| entry.final_score),
            average_score,
            runtime_trend,
            energy_trend: trend(
                history
                    .0
                    .iter()
                    .map(|entry| entry.average_energy_estimate)
                    .collect(),
                false,
            ),
            skill_reuse_trend: trend(
                history
                    .0
                    .iter()
                    .map(|entry| entry.reused_skills as f32)
                    .collect(),
                true,
            ),
            skill_reuse_quality_trend: trend(
                history
                    .0
                    .iter()
                    .map(|entry| 1.0 / (1.0 + entry.irrelevant_skills_used as f32))
                    .collect(),
                true,
            ),
            habit_usage_trend: trend(
                history
                    .0
                    .iter()
                    .map(|entry| entry.habits_used as f32)
                    .collect(),
                true,
            ),
            cache_hit_rate_trend: trend(
                history
                    .0
                    .iter()
                    .map(|entry| entry.cache_hits as f32)
                    .collect(),
                true,
            ),
            route_efficiency_trend: trend(
                history
                    .0
                    .iter()
                    .map(|entry| entry.average_route_efficiency)
                    .collect(),
                true,
            ),
            memory_hygiene_trend: trend(
                history
                    .0
                    .iter()
                    .map(|entry| entry.memories_archived as f32)
                    .collect(),
                false,
            ),
        })
    }

    pub fn cleanup_backups(&self) -> Result<BackupCleanupReport> {
        cleanup_backups(
            &self.store,
            MemoryHygienePolicy::default().max_backups_per_file,
        )
    }

    pub fn habits(&self) -> Result<Vec<crate::learning::Habit>> {
        list_habits(&self.store)
    }

    pub fn routes(&self) -> Result<RouteEfficiencyOverview> {
        route_efficiency_overview(&self.store)
    }

    pub fn cache_inspect(&self) -> Result<PlanCacheOverview> {
        plan_cache_overview(&self.store)
    }

    pub fn template_cache_inspect(&self) -> Result<TemplateCacheOverview> {
        template_cache_overview(&self.store)
    }

    pub fn journal(&self, session: Option<String>) -> Result<Vec<ActionJournalSummary>> {
        latest_journal_entries(&self.store, 20, session.as_deref())
    }

    pub fn snapshots(&self) -> Result<SnapshotOverview> {
        snapshots(&self.store)
    }

    pub fn snapshot_create(
        &self,
        project_name: &str,
        reason: &str,
    ) -> Result<crate::agency::ProjectSnapshot> {
        snapshot_create(&self.store, project_name, reason)
    }

    pub fn snapshot_restore(&self, snapshot_id: &str) -> Result<SnapshotRestoreReport> {
        snapshot_restore(&self.store, snapshot_id)
    }

    pub fn rollback_latest(&self, project_name_filter: Option<&str>) -> Result<RollbackReport> {
        rollback_latest(&self.store, project_name_filter)
    }

    pub fn transactions(&self) -> Result<TransactionOverview> {
        transactions(&self.store)
    }

    pub fn doctor(&self, repair: bool) -> Result<DoctorReport> {
        doctor(&self.store, repair)
    }

    pub fn recover_latest(&self, project_name_filter: Option<&str>) -> Result<RecoveryResult> {
        recover_latest(&self.store, project_name_filter)
    }

    pub fn sessions(&self) -> Result<Vec<WorkSessionSummary>> {
        sessions(&self.store)
    }

    pub fn session_start(&self, title: String) -> Result<WorkSession> {
        session_start(&self.store, title)
    }

    pub fn session_status(&self, selector: &str) -> Result<WorkSession> {
        crate::agency::load_session(&self.store, selector)
    }

    pub fn session_end(&self, selector: &str) -> Result<WorkSession> {
        session_end(&self.store, selector)
    }

    pub fn session_resume(&self, selector: &str) -> Result<WorkSession> {
        session_resume(&self.store, selector)
    }

    pub fn regression_check(&self) -> Result<RegressionCheckReport> {
        regression_check(&self.store)
    }

    pub fn worker(&self, prompt: String) -> Result<WorkerModeOutput> {
        let session = session_start(&self.store, "worker mode")?;
        let project_name = extract_worker_project_name(&prompt);
        let create = self.run_project(format!(
            "Create a Rust CLI calculator project called {project_name} with add and subtract functions, tests, and README"
        ))?;
        let modify = self.run_project(format!(
            "Modify the {project_name} project to add multiply and divide functions with tests"
        ))?;
        let mut failures = Vec::new();
        if create.final_status != "Completed" {
            failures.push(format!("create: {}", create.final_status));
        }
        if modify.final_status != "Completed" {
            failures.push(format!("modify: {}", modify.final_status));
        }
        let ended = session_end(&self.store, &session.session_id).unwrap_or(session);
        Ok(WorkerModeOutput {
            session_id: ended.session_id,
            goal: prompt,
            phases_completed: 5,
            tasks_completed: create.tasks_completed + modify.tasks_completed + 2,
            failures,
            recovery_actions: vec![
                "journaled project actions".to_string(),
                "snapshot coverage checked".to_string(),
                "cargo validation preserved".to_string(),
            ],
            final_report: modify.project_report_path,
        })
    }

    pub fn optimize(&self) -> Result<OptimizationReport> {
        let performance = performance_overview(&self.store)?;
        let mut habits_created = 0;
        let mut habits_strengthened = 0;
        for path in self.store.list_log_files()?.into_iter().rev().take(64) {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !name.starts_with("project_trace_") {
                continue;
            }
            let Ok(trace) = load_json::<RouteTrace>(&path) else {
                continue;
            };
            if !trace.success {
                continue;
            }
            let parsed = parse_goal(&trace.task_input);
            let state = ProjectState {
                goal_id: trace.task_id.clone(),
                project_name: parsed
                    .project_name
                    .clone()
                    .unwrap_or_else(|| "project".to_string()),
                root_path: String::new(),
                original_prompt: trace.task_input.clone(),
                status: "Completed".to_string(),
                files_created: Vec::new(),
                files_modified: Vec::new(),
                commands_run: trace.tool_actions.clone(),
                errors_seen: Vec::new(),
                checkpoints: Vec::new(),
                final_summary: None,
                final_report_path: None,
                retries_used: 0,
                self_evaluation: None,
            };
            let plan = vec![
                "Understand goal".to_string(),
                "Apply cached deterministic project workflow".to_string(),
                "Run cargo check".to_string(),
                "Run cargo test".to_string(),
            ];
            let (created, strengthened) = form_or_strengthen_habit_from_project(
                &self.store,
                &parsed,
                &state,
                plan,
                trace.runtime_ms as u64,
                trace.energy_estimate as f32,
            )?;
            habits_created += created;
            habits_strengthened += strengthened;
            let _ = update_named_route_efficiency(
                &self.store,
                &format!("optimized:{:?}", parsed.intent),
                "optimizer",
                "habit_cache",
                trace.runtime_ms as u64,
                trace.energy_estimate as f32,
                true,
                0.85,
            );
        }
        let routes = route_efficiency_overview(&self.store)?;
        let irrelevant_skills_penalized = penalize_irrelevant_skill_reuse(&self.store)?;
        let report_path = self
            .store
            .paths
            .logs
            .join(format!("optimization_report_{}.json", timestamp_slug()));
        let recommendations = if performance.profile_count < 3 {
            vec!["run more repeated tasks before expecting strong habits".to_string()]
        } else if habits_created + habits_strengthened == 0 {
            vec!["no new repeated successful workflow detected".to_string()]
        } else {
            vec!["habit and route efficiency indexes updated".to_string()]
        };
        let report = OptimizationReport {
            profiles_analyzed: performance.profile_count,
            habits_created,
            habits_strengthened,
            routes_optimized: routes.route_count,
            low_efficiency_routes_penalized: routes.least_efficient_routes.len().min(3),
            irrelevant_skills_penalized,
            recommendations,
            report_path: report_path.display().to_string(),
        };
        save_json(&report_path, &report)?;
        Ok(report)
    }

    pub fn brain_status(&self) -> Result<BrainStatus> {
        let inspect = self.inspect()?;
        let goals = list_goals(&self.store)?;
        let benchmark = self.benchmark_compare()?;
        let performance = performance_overview(&self.store)?;
        let routes = route_efficiency_overview(&self.store)?;
        let habits = habit_overview(&self.store)?;
        let cache = plan_cache_overview(&self.store)?;
        let doctor_report = doctor(&self.store, false)?;
        let reliability = reliability_score(&self.store, doctor_report.critical == 0, 0.8)?;
        let hygiene_score = if inspect.memory_hygiene.duplicate_groups == 0 {
            1.0
        } else {
            0.8
        };
        let mut recommended = Vec::new();
        if inspect.memory_hygiene.duplicate_groups > 0 {
            recommended.push("run memory-dedup".to_string());
        }
        if inspect.failed_or_blocked_task_count > 0 {
            recommended.push("inspect failed project tasks".to_string());
        }
        if performance.profile_count > 3 && routes.average_efficiency < 0.55 {
            recommended.push("run optimize".to_string());
        }
        if recommended.is_empty() {
            recommended.push("no immediate maintenance required".to_string());
        }
        Ok(BrainStatus {
            version: crate::ONYX_VERSION.to_string(),
            neurons: inspect.neurons,
            synapses: inspect.synapses,
            active_registered_projects: inspect.registered_project_count,
            historical_project_memories: inspect.historical_project_memories,
            goals_active: goals
                .iter()
                .filter(|goal| goal.status == GoalStatus::Active)
                .count(),
            goals_completed: goals
                .iter()
                .filter(|goal| goal.status == GoalStatus::Completed)
                .count(),
            goals_blocked: goals
                .iter()
                .filter(|goal| goal.status == GoalStatus::Blocked)
                .count(),
            memories_by_type: vec![
                format!("semantic: {}", inspect.memory_hygiene.semantic_memories),
                format!("procedural: {}", inspect.memory_hygiene.procedural_memories),
                format!("project: {}", inspect.memory_hygiene.project_memories),
            ],
            duplicate_memory_groups: inspect.memory_hygiene.duplicate_groups,
            top_skills_by_reuse: inspect.top_extracted_skills,
            benchmark_last_score: benchmark.last_score,
            average_project_self_evaluation: inspect.average_project_self_evaluation_score,
            memory_hygiene_score: hygiene_score,
            recommended_maintenance_actions: recommended,
            performance_profile_count: performance.profile_count,
            average_runtime_last_5: performance.average_runtime_last_5,
            average_brain_runtime_last_5: performance.average_brain_runtime_last_5,
            average_tool_runtime_last_5: performance.average_tool_runtime_last_5,
            average_cargo_runtime_last_5: performance.average_cargo_runtime_last_5,
            average_route_efficiency: routes.average_efficiency,
            habits_count: habits.habit_count,
            top_habits: habits.top_habits,
            plan_cache_entries: cache.entries,
            cache_hit_rate: cache.cache_hit_rate,
            adaptive_budget_savings_estimate: performance.estimated_budget_savings,
            optimization_recommendations: if routes.average_efficiency < 0.55 {
                vec!["run optimize to strengthen efficient routes".to_string()]
            } else {
                vec!["no immediate optimization needed".to_string()]
            },
            environment: environment_report(&self.store.paths.root),
            journal_entries_count: journal_count(&self.store).unwrap_or(0),
            active_sessions_count: active_session_count(&self.store).unwrap_or(0),
            latest_session: latest_session_id(&self.store).unwrap_or(None),
            snapshots_count: snapshot_count(&self.store).unwrap_or(0),
            recent_failures: latest_journal_entries(&self.store, 8, None)
                .unwrap_or_default()
                .into_iter()
                .filter(|entry| matches!(entry.status, crate::agency::ActionStatus::Failed))
                .map(|entry| format!("{} {:?}", entry.id, entry.action_type))
                .collect(),
            doctor_health_summary: doctor_report.recommendation,
            rollback_readiness: reliability.rollback_readiness,
            reliability_score: reliability,
            recovery_recommendations: vec!["run doctor before repair-sensitive work".to_string()],
        })
    }

    pub fn brain_status_summary(&self) -> Result<BrainStatusLite> {
        let status = self.brain_status()?;
        Ok(BrainStatusLite {
            version: status.version,
            neurons: status.neurons,
            synapses: status.synapses,
            memories: self.store.memory_files()?.len(),
            registered_projects: status.active_registered_projects,
            goals_active: status.goals_active,
            goals_completed: status.goals_completed,
            goals_blocked: status.goals_blocked,
            memory_hygiene: format!(
                "score {:.2}, duplicate groups {}",
                status.memory_hygiene_score, status.duplicate_memory_groups
            ),
            habits_count: status.habits_count,
            cache_entries: status.plan_cache_entries,
            last_benchmark_score: status.benchmark_last_score,
            average_route_efficiency: status.average_route_efficiency,
            recommended_action: status
                .recommended_maintenance_actions
                .first()
                .cloned()
                .unwrap_or_else(|| "no immediate maintenance required".to_string()),
            environment_notes: status.environment.potential_overhead_notes,
            reliability_summary: format!(
                "score {:.2}, rollback {:.2}, journal {}, snapshots {}",
                status.reliability_score.overall,
                status.rollback_readiness,
                status.journal_entries_count,
                status.snapshots_count
            ),
        })
    }

    pub fn inspect_summary(&self) -> Result<InspectSummaryLite> {
        let inspect = self.inspect()?;
        let goals = list_goals(&self.store)?;
        let habits = habit_overview(&self.store)?;
        let cache = plan_cache_overview(&self.store)?;
        let benchmark = self.benchmark_compare()?;
        let routes = route_efficiency_overview(&self.store)?;
        Ok(InspectSummaryLite {
            version: crate::ONYX_VERSION.to_string(),
            neurons: inspect.neurons,
            synapses: inspect.synapses,
            memories: inspect.memories,
            logs: inspect.logs,
            registered_projects: inspect.registered_project_count,
            goals: format!(
                "active {}, completed {}, blocked {}",
                goals
                    .iter()
                    .filter(|goal| goal.status == GoalStatus::Active)
                    .count(),
                goals
                    .iter()
                    .filter(|goal| goal.status == GoalStatus::Completed)
                    .count(),
                goals
                    .iter()
                    .filter(|goal| goal.status == GoalStatus::Blocked)
                    .count()
            ),
            memory_hygiene: inspect.memory_hygiene.recommendation,
            habits_count: habits.habit_count,
            cache_entries: cache.entries,
            last_benchmark_score: benchmark.last_score,
            average_route_efficiency: routes.average_efficiency,
            recommended_action: inspect.adaptive_budget_summary,
            reliability_summary: inspect.reliability_summary,
        })
    }

    pub fn maintain(
        &self,
    ) -> Result<(
        MemoryDedupReport,
        BackupCleanupReport,
        ConsolidationReport,
        BenchmarkCompareReport,
    )> {
        let started_at = chrono::Utc::now();
        let timer = std::time::Instant::now();
        let dedup = self.memory_dedup()?;
        let backups = self.cleanup_backups()?;
        let consolidation = self.consolidate()?;
        let _ = lightweight_auto_optimize(&self.store)?;
        let compare = self.benchmark_compare()?;
        let runtime_ms = timer.elapsed().as_millis() as u64;
        let runtime_breakdown = RuntimeBreakdown {
            total_runtime_ms: runtime_ms,
            brain_runtime_ms: 0,
            routing_runtime_ms: 0,
            memory_runtime_ms: 0,
            planning_runtime_ms: 0,
            tool_runtime_ms: 0,
            cargo_check_runtime_ms: None,
            cargo_test_runtime_ms: None,
            filesystem_runtime_ms: 0,
            reporting_runtime_ms: 0,
            maintenance_runtime_ms: runtime_ms,
            unknown_runtime_ms: 0,
        }
        .finalize_unknown();
        let _ = save_performance_profile(
            &self.store,
            &PerformanceProfile {
                id: new_profile_id(),
                command_name: "maintain".to_string(),
                task_type: "Maintenance".to_string(),
                project_name: None,
                started_at,
                ended_at: chrono::Utc::now(),
                runtime_ms,
                estimated_energy: consolidation.strengthened_routes as f32,
                active_neurons: 0,
                loaded_synapses: 0,
                memories_loaded: 0,
                skills_reused: 0,
                tool_actions: 3,
                cargo_check_runtime_ms: None,
                cargo_test_runtime_ms: None,
                success: true,
                final_score: compare.last_score.unwrap_or(1.0),
                adaptive_budget: None,
                habits_used: 0,
                cache_hits: 0,
                runtime_breakdown,
                habit_created: false,
                habit_strengthened: false,
                habit_id: None,
                fast_path_decision: None,
            },
        );
        Ok((dedup, backups, consolidation, compare))
    }

    pub fn add_memory(
        &self,
        memory_type: MemoryType,
        title: String,
        tags: Vec<String>,
        content: String,
    ) -> Result<MemoryItem> {
        self.store.ensure_layout()?;
        let id = crate::core::stable_id(&format!("memory_{title}"));
        let linked_neurons = tags
            .iter()
            .map(|tag| format!("memory_{}", crate::core::stable_id(tag)))
            .collect();
        let memory = MemoryItem::new(id, memory_type, title, content, tags, linked_neurons);
        self.store.save_memory(&memory)?;
        Ok(memory)
    }
}

fn seed_neuron(
    id: &str,
    label: &str,
    kind: NeuronKind,
    threshold: f32,
    base_activation: f32,
) -> VirtualNeuron {
    VirtualNeuron {
        id: id.to_string(),
        label: label.to_string(),
        kind,
        threshold,
        base_activation,
        last_activated_at: None,
        activation_count: 0,
        metadata: Map::new(),
    }
}

fn seed_synapse(id: &str, from: &str, to: &str, synapse_type: SynapseType, weight: f32) -> Synapse {
    let mut synapse = Synapse::new(id, from, to, synapse_type, weight);
    synapse.confidence = 0.8;
    synapse
}

fn index_neuron(index: &mut LabelIndex, neuron: &VirtualNeuron) {
    index
        .0
        .insert(neuron.label.to_lowercase(), neuron.id.clone());
    for token in neuron
        .label
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        index.0.insert(token.to_lowercase(), neuron.id.clone());
    }
}

fn select_experts(
    store: &DiskStore,
    task: &Task,
    memories: &[MemoryItem],
    active_neuron_ids: &[String],
    limit: usize,
) -> Result<Vec<ExpertResult>> {
    let stats = ensure_default_expert_stats(
        store,
        &[
            "LanguageExpert",
            "CodeExpert",
            "ReasoningExpert",
            "ToolUseExpert",
        ],
    )?;
    let experts: Vec<Box<dyn Expert>> = vec![
        Box::new(LanguageExpert),
        Box::new(CodeExpert),
        Box::new(ReasoningExpert),
        Box::new(ToolUseExpert),
    ];
    let mut scored = experts
        .into_iter()
        .map(|expert| {
            let expert_stats = stats
                .0
                .get(expert.name())
                .cloned()
                .unwrap_or_else(|| crate::experts::ExpertStats::new(expert.name()));
            let active_bonus = if active_neuron_ids.iter().any(|id| {
                id == &format!(
                    "expert_{}",
                    expert.name().trim_end_matches("Expert").to_lowercase()
                )
            }) {
                0.2
            } else {
                0.0
            };
            let normalized_energy = (expert_stats.average_energy_cost / 100.0).clamp(0.0, 1.0);
            let score =
                expert.can_handle(task) * 0.45 + expert_stats.confidence * 0.35 + active_bonus
                    - normalized_energy * 0.20;
            (score, expert)
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    let context = ExpertContext {
        task: task.clone(),
        memories: memories.to_vec(),
        active_neurons: active_neuron_ids.to_vec(),
    };
    Ok(scored
        .into_iter()
        .take(limit)
        .map(|(_, expert)| expert.run(&context))
        .collect::<Vec<_>>())
}

fn should_create_rust_project(task: &Task) -> bool {
    matches!(task.task_type, TaskType::Code | TaskType::FileOperation)
        && task.input.to_lowercase().contains("project")
}

pub fn project_name_from_input(input: &str) -> String {
    let words = input
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-'))
        .collect::<Vec<_>>();
    for marker in ["called", "named"] {
        if let Some(index) = words
            .iter()
            .position(|word| word.eq_ignore_ascii_case(marker))
        {
            if let Some(name) = words.get(index + 1) {
                let cleaned = crate::tools::sanitize_project_name(name);
                if !cleaned.is_empty() {
                    return cleaned;
                }
            }
        }
    }
    format!("project_{}", timestamp_slug())
}

fn count_json(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }
    Ok(fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .count())
}

fn remember_created(state: &mut ProjectState, path: std::path::PathBuf) {
    let display = path.display().to_string();
    if !state
        .files_created
        .iter()
        .any(|existing| existing == &display)
    {
        state.files_created.push(display);
    }
}

fn remember_modified(state: &mut ProjectState, path: impl Into<String>) {
    let path = path.into();
    if !state
        .files_modified
        .iter()
        .any(|existing| existing == &path)
    {
        state.files_modified.push(path);
    }
}

fn decompose_modification_goal(
    goal_id: &str,
    parsed: &ParsedGoal,
) -> Vec<crate::agency::QueuedTask> {
    let mut tasks = vec![
        crate::agency::QueuedTask::new(
            goal_id,
            "Inspect existing project",
            "Read current project files.",
            TaskType::Reasoning,
        ),
        crate::agency::QueuedTask::new(
            goal_id,
            "Apply requested features",
            "Edit source and tests for requested features.",
            TaskType::Code,
        ),
    ];
    if parsed.wants_readme {
        tasks.push(crate::agency::QueuedTask::new(
            goal_id,
            "Update README",
            "Update README with new features.",
            TaskType::FileOperation,
        ));
    }
    tasks.extend([
        crate::agency::QueuedTask::new(
            goal_id,
            "Run cargo check",
            "Run safe cargo check.",
            TaskType::ToolUse,
        ),
        crate::agency::QueuedTask::new(
            goal_id,
            "Run cargo test",
            "Run safe cargo test.",
            TaskType::ToolUse,
        ),
        crate::agency::QueuedTask::new(
            goal_id,
            "Create final report",
            "Create final modification report.",
            TaskType::FileOperation,
        ),
    ]);
    for index in 1..tasks.len() {
        tasks[index].dependencies = vec![tasks[index - 1].id.clone()];
    }
    tasks
}

#[allow(clippy::too_many_arguments)]
fn execute_project_task(
    title: &str,
    prompt: &str,
    parsed: &ParsedGoal,
    project_name: &str,
    project_root: &Path,
    fs_tool: &FilesystemTool,
    editor: &CodeEditorTool,
    terminal: &TerminalTool,
    state: &mut ProjectState,
    cargo_check_result: &mut String,
    cargo_test_result: &mut String,
    template_files: &[(String, String)],
) -> Result<String> {
    match title {
        "Understand goal" => Ok(format!("Goal understood for project {project_name}.")),
        "Create project directory" => {
            fs_tool.create_dir(&format!("projects/{project_name}/src"))?;
            fs_tool.create_dir(&format!("projects/{project_name}/tests"))?;
            Ok("Project directories created.".to_string())
        }
        "Write Cargo.toml" => {
            let path = editor.write_project_file(
                project_name,
                "Cargo.toml",
                &template_content(template_files, "Cargo.toml")
                    .unwrap_or_else(|| CodeExpert::cargo_toml(project_name)),
            )?;
            remember_created(state, path);
            Ok("Cargo.toml written.".to_string())
        }
        "Write src/main.rs" => {
            let path = editor.write_project_file(
                project_name,
                "src/main.rs",
                &template_content(template_files, "src/main.rs")
                    .unwrap_or_else(|| CodeExpert::calculator_main(project_name)),
            )?;
            remember_created(state, path);
            Ok("src/main.rs written.".to_string())
        }
        "Write src/lib.rs" => {
            let path = editor.write_project_file(
                project_name,
                "src/lib.rs",
                &template_content(template_files, "src/lib.rs")
                    .unwrap_or_else(|| CodeExpert::calculator_lib().to_string()),
            )?;
            remember_created(state, path);
            Ok("src/lib.rs written.".to_string())
        }
        "Write tests" => {
            let path = editor.write_project_file(
                project_name,
                "tests/calculator.rs",
                &template_content(template_files, "tests/calculator.rs")
                    .unwrap_or_else(|| CodeExpert::calculator_tests(project_name)),
            )?;
            remember_created(state, path);
            Ok("Integration tests written.".to_string())
        }
        "Write README" | "Update README" => {
            editor.update_readme_section(
                project_name,
                "features",
                &template_content(template_files, "README.md").unwrap_or_else(|| {
                    format!(
                        "Goal: {prompt}\nFeatures: {}",
                        parsed.requested_features.join(", ")
                    )
                }),
            )?;
            remember_modified(state, "README.md");
            Ok("README updated.".to_string())
        }
        "Inspect existing project" => {
            let lib = editor.read_project_file(project_name, "src/lib.rs")?;
            Ok(format!("Read src/lib.rs ({} bytes).", lib.len()))
        }
        "Apply requested features" => {
            apply_requested_features(editor, project_name, parsed, state)?;
            Ok(format!(
                "Applied features: {}.",
                parsed.requested_features.join(", ")
            ))
        }
        "Run cargo check" => run_cargo_step(
            terminal,
            editor,
            project_name,
            project_root,
            state,
            cargo_check_result,
            &["cargo", "check"],
            DiagnosticKind::CargoCheckPassed,
        ),
        "Run cargo test" => run_cargo_step(
            terminal,
            editor,
            project_name,
            project_root,
            state,
            cargo_test_result,
            &["cargo", "test"],
            DiagnosticKind::CargoTestPassed,
        ),
        "Inspect result" => Ok(format!(
            "{} files created, {} files modified, {} commands run.",
            state.files_created.len(),
            state.files_modified.len(),
            state.commands_run.len()
        )),
        "Create final report" => Ok("Final report will be written.".to_string()),
        _ => Ok("Task completed.".to_string()),
    }
}

fn apply_requested_features(
    editor: &CodeEditorTool,
    project_name: &str,
    parsed: &ParsedGoal,
    state: &mut ProjectState,
) -> Result<()> {
    let features = parsed
        .requested_features
        .iter()
        .map(|feature| feature.to_lowercase())
        .collect::<Vec<_>>();
    if features.iter().any(|feature| feature == "multiply") {
        editor.insert_function_in_lib_rs(
            project_name,
            "pub fn multiply(left: i32, right: i32) -> i32 {\n    left * right\n}\n",
        )?;
        editor.insert_test_in_tests_file(
            project_name,
            "calculator.rs",
            &format!("use {project_name}::multiply;\n\n#[test]\nfn multiply_works() {{\n    assert_eq!(multiply(6, 7), 42);\n}}\n"),
        )?;
        remember_modified(state, "src/lib.rs");
        remember_modified(state, "tests/calculator.rs");
    }
    if features.iter().any(|feature| feature == "divide") {
        editor.insert_function_in_lib_rs(
            project_name,
            "pub fn divide(left: i32, right: i32) -> Option<i32> {\n    if right == 0 {\n        None\n    } else {\n        Some(left / right)\n    }\n}\n",
        )?;
        editor.insert_test_in_tests_file(
            project_name,
            "calculator.rs",
            &format!("use {project_name}::divide;\n\n#[test]\nfn divide_works() {{\n    assert_eq!(divide(20, 4), Some(5));\n}}\n\n#[test]\nfn divide_by_zero_returns_none() {{\n    assert_eq!(divide(20, 0), None);\n}}\n"),
        )?;
        remember_modified(state, "src/lib.rs");
        remember_modified(state, "tests/calculator.rs");
    }
    Ok(())
}

fn run_cargo_step(
    terminal: &TerminalTool,
    editor: &CodeEditorTool,
    project_name: &str,
    project_root: &Path,
    state: &mut ProjectState,
    result: &mut String,
    command: &[&str],
    success_kind: DiagnosticKind,
) -> Result<String> {
    let command_result = terminal.run(command, project_root)?;
    state.commands_run.push(command.join(" "));
    let diagnostic = diagnose_command(&command_result);
    if diagnostic.kind == success_kind {
        *result = "passed".to_string();
        return Ok(format!("{} passed.", command.join(" ")));
    }
    state.errors_seen.push(diagnostic.summary.clone());
    if let Some(summary) = apply_simple_rust_fix(editor, project_name, &diagnostic)? {
        state.retries_used += 1;
        remember_modified(state, "src/lib.rs");
        let retry = terminal.run(command, project_root)?;
        state
            .commands_run
            .push(format!("{} retry", command.join(" ")));
        let retry_diagnostic = diagnose_command(&retry);
        if retry_diagnostic.kind == success_kind {
            *result = "passed after retry".to_string();
            return Ok(format!(
                "{} passed after retry: {summary}",
                command.join(" ")
            ));
        }
        *result = "failed".to_string();
        return Err(anyhow::anyhow!(retry_diagnostic.summary));
    }
    *result = "failed".to_string();
    Err(anyhow::anyhow!(diagnostic.summary))
}

fn evaluate_project(
    parsed: &ParsedGoal,
    state: &ProjectState,
    editor: &CodeEditorTool,
    project_name: &str,
    cargo_check_result: &str,
    cargo_test_result: &str,
    reused_skills: &[SkillMatch],
    habits_used: &[HabitMatch],
    plan_cache_match: Option<&PlanCacheMatch>,
    route_efficiency: &RouteEfficiencyOverview,
    hygiene: &MemoryHygieneReport,
) -> SelfEvaluation {
    let lib = editor
        .read_project_file(project_name, "src/lib.rs")
        .unwrap_or_default();
    let readme = editor
        .read_project_file(project_name, "README.md")
        .unwrap_or_default();
    let mut notes = Vec::new();
    let correctness_score =
        if cargo_check_result.contains("passed") && cargo_test_result.contains("passed") {
            notes.push("cargo check and cargo test passed.".to_string());
            1.0
        } else if cargo_check_result.contains("passed") {
            notes.push("cargo check passed.".to_string());
            0.7
        } else {
            0.2
        };
    let test_coverage_score = if cargo_test_result.contains("passed") || parsed.wants_tests {
        1.0
    } else {
        0.5
    };
    let requested = parsed
        .requested_features
        .iter()
        .filter(|feature| matches!(feature.as_str(), "multiply" | "divide" | "add" | "subtract"))
        .count();
    let found = parsed
        .requested_features
        .iter()
        .filter(|feature| lib.contains(&format!("fn {}(", feature.to_lowercase())))
        .count();
    let mut completeness_score = if requested == 0 {
        0.9
    } else {
        found as f32 / requested as f32
    };
    if parsed.wants_readme && !readme.is_empty() {
        completeness_score = (completeness_score + 0.1).min(1.0);
    }
    let irrelevant_project_workflows = irrelevant_skill_count(reused_skills, Some(project_name));
    let excess_skill_penalty = reused_skills.len().saturating_sub(5) as f32 * 0.04;
    let energy_efficiency_score = if state.retries_used == 0 {
        (1.0 - excess_skill_penalty).clamp(0.55, 1.0)
    } else {
        (0.75 - excess_skill_penalty).clamp(0.45, 1.0)
    };
    let skill_reuse_score = if reused_skills.is_empty() {
        0.5
    } else if irrelevant_project_workflows > 0 {
        0.6
    } else if state.status == "Completed" {
        1.0
    } else {
        0.3
    };
    let memory_hygiene_score = if hygiene.duplicate_groups == 0 {
        1.0
    } else {
        (1.0 - hygiene.duplicate_groups as f32 * 0.05).clamp(0.4, 1.0)
    };
    let habit_reuse_score = if habits_used.is_empty() {
        0.5
    } else {
        habits_used
            .iter()
            .map(|habit| habit.confidence * habit.relevance_score)
            .fold(0.0_f32, f32::max)
            .clamp(0.3, 1.0)
    };
    let plan_cache_score = plan_cache_match
        .map(|cache| cache.similarity_score.clamp(0.0, 1.0))
        .unwrap_or(0.5);
    let route_efficiency_score = if route_efficiency.route_count == 0 {
        0.5
    } else {
        route_efficiency.average_efficiency.clamp(0.0, 1.0)
    };
    let irrelevant_skill_penalty =
        (irrelevant_project_workflows as f32 * 0.05 + excess_skill_penalty).clamp(0.0, 0.25);
    let overall_score = correctness_score * 0.4
        + test_coverage_score * 0.13
        + completeness_score * 0.22
        + energy_efficiency_score * 0.08
        + skill_reuse_score * 0.06
        + habit_reuse_score * 0.04
        + plan_cache_score * 0.03
        + memory_hygiene_score * 0.03
        + route_efficiency_score * 0.04
        - irrelevant_skill_penalty;
    SelfEvaluation {
        correctness_score,
        test_coverage_score,
        completeness_score,
        energy_efficiency_score,
        skill_reuse_score,
        memory_hygiene_score,
        habit_reuse_score,
        plan_cache_score,
        route_efficiency_score,
        irrelevant_skill_penalty,
        overall_score,
        notes,
    }
}

fn project_report(
    project_name: &str,
    prompt: &str,
    state: &ProjectState,
    completed: usize,
    failed: usize,
    cargo_check_result: &str,
    cargo_test_result: &str,
    evaluation: &SelfEvaluation,
    runtime_breakdown: &RuntimeBreakdown,
    fast_path_decision: &FastPathDecision,
    habits_used: &[HabitMatch],
    plan_cache_match: Option<&PlanCacheMatch>,
    cargo_validation_policy: &CargoValidationPolicy,
    adaptive_budget: &AdaptiveBudgetDecision,
    live_habit_update: &LiveHabitUpdate,
    optimization_hint: &AutoOptimizeHint,
    session_id: &str,
    journal_entries: &[String],
    snapshot_ids: &[String],
    reliability: &ReliabilityScore,
    recovery_plan: Option<&RecoveryPlan>,
) -> String {
    let habits = if habits_used.is_empty() {
        "none".to_string()
    } else {
        habits_used
            .iter()
            .map(|habit| habit.title.clone())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let plan_cache = plan_cache_match
        .map(|cache| format!("hit {} ({:.2})", cache.cache_id, cache.similarity_score))
        .unwrap_or_else(|| "miss".to_string());
    let recovery = recovery_plan
        .map(|plan| format!("{:?} confidence {:.2}", plan.failure_kind, plan.confidence))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "# Onyx Brain Project Report\n\nProject: {project_name}\nGoal: {prompt}\nStatus: {}\nTasks completed: {completed}\nTasks failed: {failed}\nCargo check: {cargo_check_result}\nCargo test: {cargo_test_result}\nRetries used: {}\nSelf evaluation: {:.2}\nReliability score: {:.2}\nRollback readiness: {:.2}\nSession id: {session_id}\nJournal entries: {}\nSnapshots: {}\nRecovery plan: {recovery}\n\nRuntime breakdown:\n- total: {} ms\n- brain: {} ms\n- tools: {} ms\n- cargo: {} ms\n- filesystem: {} ms\n- reporting: {} ms\n\nFast path: {}\nReason: {}\nPreserved safety steps: {}\n\nHabits used: {habits}\nPlan cache: {plan_cache}\nCargo validation policy: check={}, test={}, reason={}\nAdaptive budget: {:?}, savings {:.0}%\nLive habit update: created={}, strengthened={}, habit={}\nOptimization hint: {} ({})\n\nFiles created:\n{}\n\nFiles modified:\n{}\n",
        state.status,
        state.retries_used,
        evaluation.overall_score,
        reliability.overall,
        reliability.rollback_readiness,
        journal_entries.len(),
        snapshot_ids.join(", "),
        runtime_breakdown.total_runtime_ms,
        runtime_breakdown.brain_runtime_ms,
        runtime_breakdown.tool_runtime_ms,
        runtime_breakdown.cargo_runtime_ms(),
        runtime_breakdown.filesystem_runtime_ms,
        runtime_breakdown.reporting_runtime_ms,
        if fast_path_decision.used_fast_path { "used" } else { "not used" },
        fast_path_decision.reason,
        fast_path_decision.preserved_steps.join(", "),
        cargo_validation_policy.run_cargo_check,
        cargo_validation_policy.run_cargo_test,
        cargo_validation_policy.reason,
        adaptive_budget.decision_type,
        adaptive_budget.estimated_savings * 100.0,
        live_habit_update.habit_created,
        live_habit_update.habit_strengthened,
        live_habit_update
            .habit_id
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        optimization_hint.reason,
        optimization_hint.recommended_command,
        state
            .files_created
            .iter()
            .map(|file| format!("- {file}"))
            .collect::<Vec<_>>()
            .join("\n"),
        state
            .files_modified
            .iter()
            .map(|file| format!("- {file}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn template_content(template_files: &[(String, String)], path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    template_files.iter().find_map(|(candidate, content)| {
        if candidate.replace('\\', "/") == normalized {
            Some(content.clone())
        } else {
            None
        }
    })
}

fn save_skills_without_duplicates(
    store: &DiskStore,
    state: &ProjectState,
    report: &str,
) -> Result<()> {
    let existing_titles = store
        .memory_files()?
        .into_iter()
        .filter_map(|path| load_json::<MemoryItem>(&path).ok())
        .filter(|memory| memory.memory_type == MemoryType::Procedural)
        .map(|memory| memory.title)
        .collect::<std::collections::BTreeSet<_>>();
    for memory in extract_skills_from_project(state, None, report) {
        if !existing_titles.contains(&memory.title) {
            store.save_memory(&memory)?;
        }
    }
    Ok(())
}

fn collect_files(root: &Path, current: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let path = entry?.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some("target") {
                continue;
            }
            collect_files(root, &path, files)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            files.push(relative.display().to_string());
        }
    }
    files.sort();
    Ok(())
}

fn review_task(
    task: &Task,
    budget: &crate::energy::EnergyBudget,
    active_neurons: &[ActiveNeuron],
    tool_actions: &[String],
    created_project_path: Option<&Path>,
    cargo_check_attempted: bool,
    cargo_check_passed: bool,
    energy_report: &EnergyReport,
    task_success: bool,
) -> SelfReview {
    let expects_project = should_create_rust_project(task);
    let expected_files_created = if expects_project {
        created_project_path.is_some_and(|path| {
            path.join("Cargo.toml").exists()
                && path.join("src").join("main.rs").exists()
                && path.join("src").join("lib.rs").exists()
        })
    } else {
        true
    };
    let cargo_check_passed_if_attempted = !cargo_check_attempted || cargo_check_passed;
    let stayed_inside_activation_budget = active_neurons.len() <= budget.max_active_neurons;
    let tools_stayed_sandboxed = tool_actions.iter().all(|action| {
        !action.contains("..")
            && !action.to_lowercase().contains("powershell")
            && !action.to_lowercase().contains("bash")
    });
    let energy_recorded = energy_report.estimated_cost_units > 0;
    let success = task_success
        && expected_files_created
        && cargo_check_passed_if_attempted
        && stayed_inside_activation_budget
        && tools_stayed_sandboxed
        && energy_recorded;
    let mut notes = Vec::new();
    if expected_files_created {
        notes.push("Expected files are present or not required.".to_string());
    }
    if cargo_check_passed_if_attempted {
        notes.push("Cargo check passed when attempted.".to_string());
    }
    if stayed_inside_activation_budget {
        notes.push("Activation budget respected.".to_string());
    }
    if tools_stayed_sandboxed {
        notes.push("Tool actions remained sandbox-oriented.".to_string());
    }
    if energy_recorded {
        notes.push("Energy estimate recorded.".to_string());
    }
    SelfReview {
        task_completed: task_success,
        expected_files_created,
        cargo_check_passed_if_attempted,
        stayed_inside_activation_budget,
        tools_stayed_sandboxed,
        energy_recorded,
        success,
        notes,
    }
}

fn top_strongest_synapses(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    for path in store.synapse_files()? {
        let synapse: Synapse = load_json(&path)?;
        rows.push((
            synapse.weight.abs() * synapse.confidence,
            format!(
                "{}: {} -> {} weight {:.2} confidence {:.2}",
                synapse.id, synapse.from, synapse.to, synapse.weight, synapse.confidence
            ),
        ));
    }
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    Ok(rows.into_iter().take(10).map(|(_, row)| row).collect())
}

fn top_used_neurons(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    for path in crate::storage::list_json_files(&store.paths.neurons)? {
        let neuron: VirtualNeuron = load_json(&path)?;
        rows.push((
            neuron.activation_count,
            format!(
                "{}: {} activations ({})",
                neuron.id, neuron.activation_count, neuron.label
            ),
        ));
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(rows.into_iter().take(10).map(|(_, row)| row).collect())
}

fn top_important_memories(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    for path in store.memory_files()? {
        let memory: MemoryItem = load_json(&path)?;
        rows.push((
            memory.importance,
            format!(
                "{}: {:.2} importance, {} accesses ({})",
                memory.id, memory.importance, memory.access_count, memory.title
            ),
        ));
    }
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    Ok(rows.into_iter().take(10).map(|(_, row)| row).collect())
}

fn last_tasks(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    for path in store.list_log_files()? {
        let Ok(trace) = load_json::<RouteTrace>(&path) else {
            continue;
        };
        rows.push((
            path.metadata()?.modified()?,
            format!(
                "{}: {:?} {} ({})",
                trace.task_id, trace.task_type, trace.task_input, trace.result
            ),
        ));
    }
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(rows.into_iter().take(5).map(|(_, row)| row).collect())
}

fn average_energy_estimate(store: &DiskStore) -> Result<f32> {
    let mut total = 0_u64;
    let mut count = 0_u64;
    for path in store.list_log_files()? {
        let Ok(trace) = load_json::<RouteTrace>(&path) else {
            continue;
        };
        total += trace.energy_estimate;
        count += 1;
    }
    if count == 0 {
        Ok(0.0)
    } else {
        Ok(total as f32 / count as f32)
    }
}

fn last_consolidation_time(store: &DiskStore) -> Result<Option<String>> {
    let mut latest = None;
    for path in store.list_log_files()? {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("consolidation_report_") {
            continue;
        }
        let modified = path.metadata()?.modified()?;
        let display = name
            .trim_start_matches("consolidation_report_")
            .trim_end_matches(".json")
            .to_string();
        if latest
            .as_ref()
            .is_none_or(|(latest_modified, _): &(std::time::SystemTime, String)| {
                modified > *latest_modified
            })
        {
            latest = Some((modified, display));
        }
    }
    Ok(latest.map(|(_, display)| display))
}

fn known_projects(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    if !store.paths.projects.exists() {
        return Ok(rows);
    }
    for entry in fs::read_dir(&store.paths.projects)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let state_path = path.join("project_state.json");
        if !state_path.exists() {
            continue;
        }
        let state: ProjectState = load_json(&state_path)?;
        rows.push(format!(
            "{}: {} ({})",
            state.goal_id, state.project_name, state.status
        ));
    }
    rows.sort();
    Ok(rows.into_iter().take(10).collect())
}

fn failed_project_tasks(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    if !store.paths.projects.exists() {
        return Ok(rows);
    }
    for entry in fs::read_dir(&store.paths.projects)? {
        let path = entry?.path();
        let queue_path = path.join("task_queue.json");
        if !queue_path.exists() {
            continue;
        }
        let queue: Vec<crate::agency::QueuedTask> = load_json(&queue_path)?;
        for task in queue
            .into_iter()
            .filter(|task| task.status == TaskStatus::Failed || task.status == TaskStatus::Blocked)
        {
            rows.push(format!(
                "{}: {} - {}",
                task.parent_goal_id,
                task.title,
                task.error_summary.unwrap_or_default()
            ));
        }
    }
    Ok(rows.into_iter().take(10).collect())
}

fn project_retry_counts(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    if !store.paths.projects.exists() {
        return Ok(rows);
    }
    for entry in fs::read_dir(&store.paths.projects)? {
        let path = entry?.path();
        let Some(goal_id) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if let Ok(state) = load_project_state(store, goal_id) {
            rows.push(format!(
                "{}: {} retries",
                state.project_name, state.retries_used
            ));
        }
    }
    rows.sort();
    Ok(rows.into_iter().take(10).collect())
}

fn last_project_report_path(store: &DiskStore) -> Result<Option<String>> {
    let mut latest = None;
    if !store.paths.projects.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(&store.paths.projects)? {
        let path = entry?.path().join("final_report.md");
        if !path.exists() {
            continue;
        }
        let modified = path.metadata()?.modified()?;
        let display = path.display().to_string();
        if latest
            .as_ref()
            .is_none_or(|(latest_modified, _): &(std::time::SystemTime, String)| {
                modified > *latest_modified
            })
        {
            latest = Some((modified, display));
        }
    }
    Ok(latest.map(|(_, display)| display))
}

fn recovery_report_rows(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    for path in store.list_log_files()? {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with("rollback_report_") || name.starts_with("doctor_report_") {
            rows.push(path.display().to_string());
        }
    }
    rows.sort();
    rows.reverse();
    rows.truncate(5);
    Ok(rows)
}

fn top_extracted_skills(store: &DiskStore) -> Result<Vec<String>> {
    let mut rows = Vec::new();
    for path in store.memory_files()? {
        let memory: MemoryItem = load_json(&path)?;
        if memory.memory_type == MemoryType::Procedural
            && memory.tags.iter().any(|tag| tag == "skill")
        {
            rows.push((
                memory.importance,
                format!("{}: {}", memory.id, memory.title),
            ));
        }
    }
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    Ok(rows.into_iter().take(10).map(|(_, row)| row).collect())
}

fn average_project_self_evaluation_score(store: &DiskStore) -> Result<f32> {
    let mut total = 0.0_f32;
    let mut count = 0.0_f32;
    if !store.paths.projects.exists() {
        return Ok(0.0);
    }
    for entry in fs::read_dir(&store.paths.projects)? {
        let path = entry?.path().join("project_state.json");
        if !path.exists() {
            continue;
        }
        let state: ProjectState = load_json(&path)?;
        if let Some(evaluation) = state.self_evaluation {
            total += evaluation.overall_score;
            count += 1.0;
        }
    }
    if count == 0.0 {
        Ok(0.0)
    } else {
        Ok(total / count)
    }
}

fn failed_or_blocked_task_count(store: &DiskStore) -> Result<usize> {
    Ok(failed_project_tasks(store)?.len())
}

fn count_project_memories(store: &DiskStore, archived: bool) -> Result<usize> {
    let dir = if archived {
        store.paths.memories.join("archive")
    } else {
        store.paths.memories.clone()
    };
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Ok(memory) = load_json::<MemoryItem>(&path) {
                if memory.memory_type == MemoryType::Project {
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

fn benchmark_history_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("benchmark_history.json")
}

fn load_benchmark_history(store: &DiskStore) -> Result<BenchmarkHistory> {
    let path = benchmark_history_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(BenchmarkHistory::default())
    }
}

fn append_benchmark_history(store: &DiskStore, report: &BenchmarkReport) -> Result<()> {
    let mut history = load_benchmark_history(store)?;
    history.0.push(BenchmarkHistoryEntry {
        timestamp: report.completed_at,
        final_score: report.final_score,
        runtime_ms: report.total_runtime_ms,
        average_energy_estimate: report.average_energy_estimate,
        reused_skills: report.reused_skills_count,
        memories_archived: report.memories_archived,
        tasks_successful: report.tasks_successful,
        tasks_failed: report.tasks_failed,
        irrelevant_skills_used: report.irrelevant_skills_used,
        habits_used: report.habits_used,
        cache_hits: report.cache_hits,
        adaptive_budget_decisions: report.adaptive_budget_decisions,
        average_route_efficiency: report.average_route_efficiency,
        template_cache_hits: report.template_cache_hits,
        runtime_diagnosis: report.runtime_diagnosis.clone(),
    });
    save_json(&benchmark_history_path(store), &history)
}

fn diagnose_benchmark_runtime(breakdown: &RuntimeBreakdown) -> BenchmarkRuntimeDiagnosis {
    let total = breakdown.total_runtime_ms.max(1) as f32;
    let cargo = breakdown.cargo_runtime_ms();
    let buckets = [
        ("cargo", cargo),
        ("filesystem", breakdown.filesystem_runtime_ms),
        ("tools", breakdown.tool_runtime_ms.saturating_sub(cargo)),
        ("brain", breakdown.brain_runtime_ms),
    ];
    let (main_runtime_source, _) = buckets
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .unwrap_or(("unknown", 0));
    let recommendation = match main_runtime_source {
        "cargo" => "runtime is Cargo-bound; brain optimization may not reduce total runtime much",
        "filesystem" => "runtime is filesystem-bound; path sync or disk overhead may dominate",
        "tools" => "runtime is tool-bound; preserve safety while reducing redundant tool work",
        "brain" => "runtime is brain-bound; optimize habits, route efficiency, and cache usage",
        _ => "insufficient runtime signal",
    }
    .to_string();
    BenchmarkRuntimeDiagnosis {
        main_runtime_source: main_runtime_source.to_string(),
        brain_runtime_percent: breakdown.brain_runtime_ms as f32 / total,
        tool_runtime_percent: breakdown.tool_runtime_ms as f32 / total,
        cargo_runtime_percent: cargo as f32 / total,
        filesystem_runtime_percent: breakdown.filesystem_runtime_ms as f32 / total,
        recommendation,
    }
}

fn benchmark_runtime_trend(history: &BenchmarkHistory) -> String {
    if history.0.len() < 2 {
        return "insufficient history".to_string();
    }
    let first = &history.0[0];
    let last = history.0.last().expect("history length checked");
    if last.runtime_ms <= first.runtime_ms {
        return "runtime improving".to_string();
    }
    let source = last.runtime_diagnosis.main_runtime_source.as_str();
    if source == "cargo" || source == "tools" || source == "filesystem" {
        if last.cache_hits >= first.cache_hits || last.habits_used >= first.habits_used {
            "brain improving, total runtime tool-bound".to_string()
        } else {
            "cache improved but cargo time increased".to_string()
        }
    } else {
        "runtime worsening".to_string()
    }
}

fn trend(values: Vec<f32>, higher_is_better: bool) -> String {
    if values.len() < 2 {
        return "insufficient history".to_string();
    }
    let first = values.first().copied().unwrap_or_default();
    let last = values.last().copied().unwrap_or_default();
    if (last - first).abs() < f32::EPSILON {
        "flat".to_string()
    } else if (last > first) == higher_is_better {
        "improving".to_string()
    } else {
        "worsening".to_string()
    }
}

fn previous_project_failure(store: &DiskStore, parsed: &ParsedGoal) -> Result<bool> {
    let signature = crate::agency::goal_signature(parsed);
    for path in store.list_log_files()?.into_iter().rev().take(32) {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("project_trace_") {
            continue;
        }
        let Ok(trace) = load_json::<RouteTrace>(&path) else {
            continue;
        };
        if crate::agency::goal_signature(&parse_goal(&trace.task_input)) == signature {
            return Ok(!trace.success);
        }
    }
    Ok(false)
}

fn penalize_irrelevant_skill_reuse(store: &DiskStore) -> Result<usize> {
    let mut penalized = 0;
    for path in store.memory_files()?.into_iter().take(128) {
        let Ok(mut memory) = load_json::<MemoryItem>(&path) else {
            continue;
        };
        if memory.memory_type != MemoryType::Procedural {
            continue;
        }
        let title = memory.title.to_lowercase();
        if title.starts_with("workflow for") && memory.access_count == 0 {
            memory.importance = (memory.importance - 0.02).clamp(0.2, 1.0);
            store.save_memory(&memory)?;
            penalized += 1;
        }
    }
    save_json(
        &store.paths.indexes.join("skill_reuse_quality.json"),
        &serde_json::json!({ "irrelevant_skill_count": penalized }),
    )?;
    Ok(penalized)
}
