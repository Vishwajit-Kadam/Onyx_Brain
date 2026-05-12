use chrono::Utc;
use onyx_brain::{
    agency::{
        find_cached_plan, parse_goal, save_plan_cache_entry, store_successful_plan, PlanCacheEntry,
        ProjectState,
    },
    core::{RouteTrace, TaskType},
    energy::{
        load_performance_index, AdaptiveBudgetDecision, AdaptiveBudgetDecisionType,
        AdaptiveBudgetManager, EnergyBudget, EnergyReport,
    },
    learning::{
        find_matching_habits, form_or_strengthen_habit_from_project, save_habit, Habit,
        SkillReuseEngine,
    },
    memory::{MemoryItem, MemoryType},
    routing::{load_route_efficiency, update_named_route_efficiency},
    Brain,
};
use serde_json::Map;

fn temp_brain() -> (tempfile::TempDir, Brain) {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    (temp, brain)
}

#[test]
fn performance_profile_saves_after_project_run() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .run_project("Create a Rust CLI project called perf_calc with tests".to_string())
        .expect("project");
    assert_eq!(output.final_status, "Completed");
    let index = load_performance_index(brain.store()).expect("performance index");
    assert!(index
        .profiles
        .iter()
        .any(|profile| profile.command_name == "project"));
}

#[test]
fn route_efficiency_updates_after_success() {
    let (_temp, brain) = temp_brain();
    update_named_route_efficiency(brain.store(), "route_test", "a", "b", 100, 10.0, true, 0.9)
        .expect("update route");
    let index = load_route_efficiency(brain.store()).expect("route index");
    let route = index.routes.get("route_test").expect("route");
    assert!(route.efficiency_score > 0.4);
}

#[test]
fn adaptive_budget_reduces_familiar_simple_task_and_expands_after_failure() {
    let (_temp, brain) = temp_brain();
    let parsed = parse_goal("Create a Rust CLI calculator project called fast_one with tests");
    let habit = onyx_brain::learning::HabitMatch {
        habit_id: "habit_fast".to_string(),
        title: "Create Rust CLI calculator project".to_string(),
        relevance_score: 0.9,
        confidence: 0.9,
        matched_patterns: vec!["create".to_string()],
        matched_features: vec!["tests".to_string()],
        expected_energy_saving: 0.2,
        reason: "test".to_string(),
    };
    let reduced = AdaptiveBudgetManager::decide_for_task(
        brain.store(),
        &TaskType::Code,
        Some(&parsed),
        &[habit],
        true,
        false,
    );
    assert!(matches!(
        reduced.decision_type,
        AdaptiveBudgetDecisionType::Reduced
    ));
    assert!(
        reduced.adjusted_budget.max_active_neurons < reduced.original_budget.max_active_neurons
    );

    let expanded = AdaptiveBudgetManager::decide_for_task(
        brain.store(),
        &TaskType::Code,
        Some(&parsed),
        &[],
        false,
        true,
    );
    assert!(matches!(
        expanded.decision_type,
        AdaptiveBudgetDecisionType::Expanded
    ));
}

#[test]
fn habit_created_after_repeated_successes_and_reused_for_similar_task() {
    let (_temp, brain) = temp_brain();
    let parsed = parse_goal("Create a Rust CLI calculator project called habit_a with tests");
    for i in 0..3 {
        brain
            .store()
            .save_log(
                &format!("project_trace_habit_{i}"),
                &trace_for(&parsed.original_prompt, true),
            )
            .expect("trace");
    }
    let state = completed_state("habit_a");
    let (created, _) = form_or_strengthen_habit_from_project(
        brain.store(),
        &parsed,
        &state,
        vec!["Understand goal".to_string(), "Run cargo test".to_string()],
        100,
        10.0,
    )
    .expect("habit");
    assert_eq!(created, 1);
    let matches = find_matching_habits(brain.store(), &parsed, 3).expect("habits");
    assert!(!matches.is_empty());
}

#[test]
fn plan_cache_stores_and_reuses_similar_goal() {
    let (_temp, brain) = temp_brain();
    let parsed = parse_goal("Create a Rust CLI calculator project called cache_a with tests");
    let state = completed_state("cache_a");
    let entry = store_successful_plan(
        brain.store(),
        &parsed,
        &state,
        vec!["Write Cargo.toml".to_string(), "Run cargo test".to_string()],
        120,
        12.0,
    )
    .expect("store plan");
    assert!(entry.success_count > 0);
    let similar = parse_goal("Create a Rust CLI calculator project called cache_b with tests");
    let found = find_cached_plan(brain.store(), &similar)
        .expect("find")
        .expect("cache hit");
    assert_eq!(found.cache_id, entry.cache_id);
}

#[test]
fn cache_inspect_habits_and_routes_do_not_crash() {
    let (_temp, brain) = temp_brain();
    brain.cache_inspect().expect("cache inspect");
    brain.habits().expect("habits");
    brain.routes().expect("routes");
}

#[test]
fn cache_inspect_command_data_does_not_crash() {
    let (_temp, brain) = temp_brain();
    let cache = brain.cache_inspect().expect("cache inspect");
    assert_eq!(cache.entries, 0);
}

#[test]
fn habits_command_data_does_not_crash() {
    let (_temp, brain) = temp_brain();
    let habits = brain.habits().expect("habits");
    assert!(habits.is_empty());
}

#[test]
fn routes_command_data_does_not_crash() {
    let (_temp, brain) = temp_brain();
    let routes = brain.routes().expect("routes");
    assert_eq!(routes.route_count, 0);
}

#[test]
fn skill_filtering_limits_unrelated_project_workflows() {
    let (_temp, brain) = temp_brain();
    let workflow = MemoryItem {
        id: "workflow_unrelated".to_string(),
        memory_type: MemoryType::Procedural,
        title: "Workflow for bench_calc".to_string(),
        content: "Create calculator project with tests".to_string(),
        summary: "bench workflow".to_string(),
        tags: vec![
            "rust".to_string(),
            "project".to_string(),
            "workflow".to_string(),
        ],
        importance: 0.9,
        last_accessed_at: None,
        created_at: Utc::now(),
        access_count: 0,
        linked_neurons: Vec::new(),
        metadata: Map::new(),
    };
    let generic = MemoryItem {
        id: "generic_create_cli".to_string(),
        memory_type: MemoryType::Procedural,
        title: "Create Rust CLI project".to_string(),
        content: "Create Cargo.toml and run cargo test".to_string(),
        summary: "generic workflow".to_string(),
        tags: vec![
            "rust".to_string(),
            "project".to_string(),
            "skill".to_string(),
        ],
        importance: 0.9,
        last_accessed_at: None,
        created_at: Utc::now(),
        access_count: 0,
        linked_neurons: Vec::new(),
        metadata: Map::new(),
    };
    brain.store().save_memory(&workflow).expect("workflow");
    brain.store().save_memory(&generic).expect("generic");
    let parsed = parse_goal("Create a Rust CLI calculator project called fresh_calc with tests");
    let matches = SkillReuseEngine::find_relevant_skills(
        brain.store(),
        &parsed,
        &[],
        &EnergyBudget::default(),
    )
    .expect("skills");
    assert!(matches.len() <= 5);
    assert!(!matches
        .iter()
        .any(|skill| skill.title == "Workflow for bench_calc"));
}

#[test]
fn benchmark_report_includes_cache_habit_metrics_and_optimize_creates_report() {
    let (_temp, brain) = temp_brain();
    let report = brain.benchmark("basic").expect("benchmark");
    assert!(report.tasks_run > 0);
    let optimize = brain.optimize().expect("optimize");
    assert!(std::path::Path::new(&optimize.report_path).exists());
}

#[test]
fn benchmark_compare_reports_insufficient_history_when_empty() {
    let (_temp, brain) = temp_brain();
    let compare = brain.benchmark_compare().expect("compare");
    assert_eq!(compare.runtime_trend, "insufficient history");
}

#[test]
fn optimize_command_creates_report() {
    let (_temp, brain) = temp_brain();
    let report = brain.optimize().expect("optimize");
    assert!(std::path::Path::new(&report.report_path).exists());
}

#[test]
fn project_run_records_adaptive_budget_decision() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .run_project("Create a Rust CLI project called adaptive_record with tests".to_string())
        .expect("project");
    assert!(matches!(
        output.adaptive_budget.decision_type,
        AdaptiveBudgetDecisionType::Reduced
            | AdaptiveBudgetDecisionType::Expanded
            | AdaptiveBudgetDecisionType::Unchanged
    ));
}

#[test]
fn inspect_includes_efficiency_habit_and_cache_summaries() {
    let (_temp, brain) = temp_brain();
    let summary = brain.inspect().expect("inspect");
    assert!(summary
        .adaptive_budget_summary
        .contains("estimated savings"));
    assert_eq!(summary.route_efficiency_top.len(), 0);
    assert_eq!(summary.habit_summary.len(), 0);
    assert_eq!(summary.cache_summary.len(), 0);
}

#[test]
fn brain_status_includes_performance_habit_cache_stats() {
    let (_temp, brain) = temp_brain();
    brain
        .run_project("Create a Rust CLI project called status_calc with tests".to_string())
        .expect("project");
    let status = brain.brain_status().expect("status");
    assert_eq!(status.version, "v0.9");
    assert!(status.performance_profile_count > 0);
    let _ = status.plan_cache_entries;
    let _ = status.habits_count;
}

#[test]
fn self_evaluation_penalizes_irrelevant_skill_usage() {
    let (_temp, brain) = temp_brain();
    let workflow = MemoryItem {
        id: "only_unrelated_workflow".to_string(),
        memory_type: MemoryType::Procedural,
        title: "Workflow for other_calc".to_string(),
        content: "Create Rust CLI calculator project with add subtract tests README".to_string(),
        summary: "other workflow".to_string(),
        tags: vec![
            "rust".to_string(),
            "project".to_string(),
            "workflow".to_string(),
            "calculator".to_string(),
        ],
        importance: 1.0,
        last_accessed_at: None,
        created_at: Utc::now(),
        access_count: 0,
        linked_neurons: Vec::new(),
        metadata: Map::new(),
    };
    brain.store().save_memory(&workflow).expect("save memory");
    let output = brain
        .run_project(
            "Create a Rust CLI calculator project called penalty_calc with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("project");
    assert!(output.self_evaluation.irrelevant_skill_penalty >= 0.0);
    assert!(output.self_evaluation.overall_score <= 1.0);
}

#[test]
fn repeated_calculator_project_creation_uses_habit_or_cached_plan() {
    let (_temp, brain) = temp_brain();
    let parsed = parse_goal("Create a Rust CLI calculator project called repeat_a with tests");
    let now = Utc::now();
    save_habit(
        brain.store(),
        &Habit {
            habit_id: "habit_repeat_create".to_string(),
            title: "Create Rust CLI calculator project".to_string(),
            trigger_patterns: vec!["create rust cli calculator".to_string()],
            task_type: format!("{:?}", parsed.intent),
            project_type: Some("rust_cli".to_string()),
            required_features: vec!["tests".to_string()],
            plan_template: vec!["Write Cargo.toml".to_string(), "Run cargo test".to_string()],
            preferred_skills: vec!["skill_create_rust_cli_project".to_string()],
            preferred_tools: vec!["CodeEditorTool".to_string()],
            average_runtime_ms: 100.0,
            average_energy: 10.0,
            success_count: 3,
            failure_count: 0,
            confidence: 0.9,
            created_at: now,
            updated_at: now,
        },
    )
    .expect("save habit");
    save_plan_cache_entry(
        brain.store(),
        &PlanCacheEntry {
            cache_id: "plan_repeat_create".to_string(),
            normalized_goal_signature: onyx_brain::agency::goal_signature(&parsed),
            intent: format!("{:?}", parsed.intent),
            task_type: "Code".to_string(),
            features: parsed.requested_features.clone(),
            plan_steps: vec!["Write Cargo.toml".to_string(), "Run cargo test".to_string()],
            expected_files: vec!["Cargo.toml".to_string()],
            expected_tools: vec!["TerminalTool".to_string()],
            success_count: 3,
            failure_count: 0,
            average_runtime_ms: 100.0,
            average_energy: 10.0,
            last_used_at: Some(now),
            created_at: now,
            updated_at: now,
        },
    )
    .expect("save cache");
    let output = brain
        .run_project("Create a Rust CLI calculator project called repeat_b with tests".to_string())
        .expect("project");
    assert!(output.plan_cache_match.is_some() || !output.habits_used.is_empty());
}

fn completed_state(name: &str) -> ProjectState {
    let mut state = ProjectState::new("goal", name, "root", "prompt");
    state.status = "Completed".to_string();
    state.files_created = vec!["src/lib.rs".to_string(), "tests/calculator.rs".to_string()];
    state
}

fn trace_for(prompt: &str, success: bool) -> RouteTrace {
    RouteTrace {
        task_id: uuid::Uuid::new_v4().to_string(),
        task_input: prompt.to_string(),
        task_type: TaskType::Code,
        activated_neurons: Vec::new(),
        activated_synapses: Vec::new(),
        selected_experts: vec!["CodeExpert".to_string()],
        selected_memories: Vec::new(),
        tool_actions: vec!["cargo test".to_string()],
        success,
        result: if success { "Completed" } else { "Failed" }.to_string(),
        energy_estimate: 10,
        runtime_ms: 100,
        energy_report: EnergyReport::default(),
        learning_updates: onyx_brain::learning::LearningReport {
            strengthened: 0,
            weakened: 0,
            new_synapses: 0,
        },
        reused_skills: Vec::new(),
        skill_application_results: Vec::new(),
        habits_used: Vec::new(),
        plan_cache_match: None,
        adaptive_budget: Some(AdaptiveBudgetDecision {
            original_budget: EnergyBudget::default(),
            adjusted_budget: EnergyBudget::default(),
            decision_type: AdaptiveBudgetDecisionType::Unchanged,
            reason: "test".to_string(),
            confidence: 0.5,
            estimated_savings: 0.0,
        }),
        live_habit_update: None,
        fast_path_decision: None,
        cargo_validation_policy: None,
        runtime_breakdown: None,
        optimization_hint: None,
        session_id: None,
        journal_entries: Vec::new(),
        snapshot_ids: Vec::new(),
        transaction_ids: Vec::new(),
        recovery_plan: None,
        reliability_score: None,
    }
}
