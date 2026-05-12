use chrono::Utc;
use onyx_brain::{
    agency::{
        decide_fast_path, find_cached_plan, find_template_for_goal, parse_goal,
        save_plan_cache_entry, PlanCacheEntry,
    },
    core::brain::{BenchmarkHistory, BenchmarkHistoryEntry},
    energy::{load_performance_index, profile_dir, PerformanceProfile, RuntimeBreakdown},
    learning::{auto_optimize_hint, habit_signature, Habit, HabitMatch},
    storage::save_json,
    tools::decide_cargo_validation,
    utils::environment_report,
    Brain,
};

fn temp_brain() -> (tempfile::TempDir, Brain) {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    (temp, brain)
}

fn confident_habit(parsed: &onyx_brain::agency::ParsedGoal) -> Habit {
    let now = Utc::now();
    Habit {
        habit_id: "habit_fast_calc".to_string(),
        title: "Create Rust CLI calculator project".to_string(),
        trigger_patterns: vec![habit_signature(parsed)],
        task_type: format!("{:?}", parsed.intent),
        project_type: Some("rust_cli".to_string()),
        required_features: parsed.requested_features.clone(),
        plan_template: vec![
            "Create project directory".to_string(),
            "Write Cargo.toml".to_string(),
            "Run cargo check".to_string(),
        ],
        preferred_skills: vec!["Create Rust CLI project".to_string()],
        preferred_tools: vec!["CodeEditorTool".to_string(), "TerminalTool".to_string()],
        average_runtime_ms: 100.0,
        average_energy: 10.0,
        success_count: 4,
        failure_count: 0,
        confidence: 0.92,
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn live_habit_strengthens_and_creates_after_repeated_successes() {
    let (_temp, brain) = temp_brain();
    for name in ["live_a", "live_b", "live_c", "live_d"] {
        let output = brain
            .run_project(format!(
                "Create a Rust CLI calculator project called {name} with add and subtract functions, tests, and README"
            ))
            .expect("project");
        assert_eq!(output.final_status, "Completed");
    }
    let habits = brain.habits().expect("habits");
    assert!(!habits.is_empty());
    assert!(habits.iter().any(|habit| habit.success_count >= 3));
}

#[test]
fn runtime_breakdown_is_saved_and_totals_are_valid() {
    let (_temp, brain) = temp_brain();
    brain
        .run_project("Create a Rust CLI project called runtime_calc with tests".to_string())
        .expect("project");
    let index = load_performance_index(brain.store()).expect("index");
    let profile = index
        .profiles
        .iter()
        .rev()
        .find(|profile| profile.command_name == "project")
        .expect("project profile");
    let path = profile_dir(brain.store()).join(format!("{}.json", profile.id));
    let full: PerformanceProfile = onyx_brain::storage::load_json(&path).expect("profile");
    assert!(full.runtime_breakdown.total_runtime_ms >= full.runtime_breakdown.brain_runtime_ms);
    assert_eq!(full.runtime_breakdown.total_runtime_ms, full.runtime_ms);
}

#[test]
fn fast_path_uses_high_confidence_habit_and_preserves_cargo_validation() {
    let parsed = parse_goal(
        "Create a Rust CLI calculator project called fast_path_calc with add and subtract functions, tests, and README",
    );
    let _habit = confident_habit(&parsed);
    let habit_matches = vec![HabitMatch {
        habit_id: "habit_fast_calc".to_string(),
        title: "Create Rust CLI calculator project".to_string(),
        relevance_score: 1.0,
        confidence: 0.92,
        matched_patterns: vec!["create".to_string()],
        matched_features: parsed.requested_features.clone(),
        expected_energy_saving: 0.2,
        reason: "test high confidence habit".to_string(),
    }];
    let decision = decide_fast_path(&parsed, &habit_matches, None, false);
    assert!(decision.used_fast_path);
    assert!(decision
        .preserved_steps
        .iter()
        .any(|step| step.contains("cargo validation")));
    let policy = decide_cargo_validation(&parsed, &["src/lib.rs".to_string()], &[], false);
    assert!(policy.run_cargo_check);
    assert!(policy.run_cargo_test);
}

#[test]
fn template_cache_stores_and_reuses_rust_cli_calculator_template() {
    let (_temp, brain) = temp_brain();
    brain
        .run_project(
            "Create a Rust CLI calculator project called template_a with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("first project");
    let parsed = parse_goal(
        "Create a Rust CLI calculator project called template_b with add and subtract functions, tests, and README",
    );
    assert!(find_template_for_goal(brain.store(), &parsed)
        .expect("template lookup")
        .is_some());
    let output = brain
        .run_project(parsed.original_prompt.clone())
        .expect("second project");
    assert!(output.template_cache_used.is_some());
    assert!(brain.template_cache_inspect().expect("overview").entries > 0);
}

#[test]
fn cargo_policy_runs_for_code_and_skips_readme_only() {
    let parsed = parse_goal("Modify the docs_calc project README");
    let code = decide_cargo_validation(&parsed, &[], &["src/lib.rs".to_string()], false);
    assert!(code.run_cargo_check);
    assert!(code.run_cargo_test);
    let readme = decide_cargo_validation(&parsed, &[], &["README.md".to_string()], false);
    assert!(!readme.run_cargo_check);
    assert!(!readme.run_cargo_test);
}

#[test]
fn benchmark_report_and_compare_include_runtime_diagnosis() {
    let (_temp, brain) = temp_brain();
    let report = brain.benchmark("basic").expect("benchmark");
    assert!(!report.runtime_diagnosis.main_runtime_source.is_empty());
    assert!(report.runtime_breakdown.total_runtime_ms > 0);

    let history_path = brain.store().paths.indexes.join("benchmark_history.json");
    let now = Utc::now();
    save_json(
        &history_path,
        &BenchmarkHistory(vec![
            BenchmarkHistoryEntry {
                timestamp: now,
                final_score: 0.8,
                runtime_ms: 100,
                average_energy_estimate: 20.0,
                reused_skills: 1,
                memories_archived: 0,
                tasks_successful: 5,
                tasks_failed: 0,
                irrelevant_skills_used: 0,
                habits_used: 0,
                cache_hits: 0,
                adaptive_budget_decisions: 0,
                average_route_efficiency: 0.5,
                template_cache_hits: 0,
                runtime_diagnosis: Default::default(),
            },
            BenchmarkHistoryEntry {
                timestamp: now,
                final_score: 0.9,
                runtime_ms: 200,
                average_energy_estimate: 18.0,
                reused_skills: 2,
                memories_archived: 0,
                tasks_successful: 5,
                tasks_failed: 0,
                irrelevant_skills_used: 0,
                habits_used: 1,
                cache_hits: 1,
                adaptive_budget_decisions: 1,
                average_route_efficiency: 0.7,
                template_cache_hits: 1,
                runtime_diagnosis: onyx_brain::core::brain::BenchmarkRuntimeDiagnosis {
                    main_runtime_source: "cargo".to_string(),
                    ..Default::default()
                },
            },
        ]),
    )
    .expect("history");
    let compare = brain.benchmark_compare().expect("compare");
    assert!(
        compare.runtime_trend.contains("tool-bound") || compare.runtime_trend.contains("cargo")
    );
}

#[test]
fn environment_and_summary_commands_do_not_crash() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fake_cloud_sync = temp
        .path()
        .join(format!("{} Folder", ["One", "Drive"].concat()))
        .join("Onyx");
    let report = environment_report(&fake_cloud_sync);
    assert!(report.is_onedrive_path);
    assert!(report.path_has_spaces);

    let (_temp, brain) = temp_brain();
    assert_eq!(brain.inspect_summary().expect("inspect").version, "v0.0.1");
    assert_eq!(
        brain.brain_status_summary().expect("status").version,
        "v0.0.1"
    );
}

#[test]
fn auto_optimize_hint_and_maintain_work() {
    let (_temp, brain) = temp_brain();
    let now = Utc::now();
    for index in 0..10 {
        onyx_brain::energy::save_performance_profile(
            brain.store(),
            &PerformanceProfile {
                id: format!("profile_{index}"),
                command_name: "project".to_string(),
                task_type: "Code".to_string(),
                project_name: None,
                started_at: now,
                ended_at: now,
                runtime_ms: 10,
                estimated_energy: 10.0,
                active_neurons: 0,
                loaded_synapses: 0,
                memories_loaded: 0,
                skills_reused: 0,
                tool_actions: 0,
                cargo_check_runtime_ms: None,
                cargo_test_runtime_ms: None,
                success: true,
                final_score: 1.0,
                adaptive_budget: None,
                habits_used: 0,
                cache_hits: 0,
                runtime_breakdown: RuntimeBreakdown::new(10),
                habit_created: false,
                habit_strengthened: false,
                habit_id: None,
                fast_path_decision: None,
            },
        )
        .expect("profile");
    }
    let hint = auto_optimize_hint(brain.store(), 0, false).expect("hint");
    assert!(hint.should_optimize);
    brain.maintain().expect("maintain");
}

#[test]
fn final_report_includes_runtime_and_fast_path_sections() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .run_project("Create a Rust CLI project called report_calc with tests".to_string())
        .expect("project");
    let report = std::fs::read_to_string(output.project_report_path).expect("report");
    assert!(report.contains("Runtime breakdown"));
    assert!(report.contains("Fast path"));
    assert!(report.contains("Cargo validation policy"));
}

#[test]
fn plan_cache_reuses_similar_goal_data_path() {
    let (_temp, brain) = temp_brain();
    let parsed = parse_goal("Create a Rust CLI calculator project called cache_one with tests");
    let now = Utc::now();
    save_plan_cache_entry(
        brain.store(),
        &PlanCacheEntry {
            cache_id: "cache_rust_cli".to_string(),
            normalized_goal_signature: onyx_brain::agency::goal_signature(&parsed),
            intent: "CreateProject".to_string(),
            task_type: "Code".to_string(),
            features: parsed.requested_features.clone(),
            plan_steps: vec![
                "Write Cargo.toml".to_string(),
                "Run cargo check".to_string(),
            ],
            expected_files: vec!["Cargo.toml".to_string()],
            expected_tools: vec!["CodeEditorTool".to_string()],
            success_count: 3,
            failure_count: 0,
            average_runtime_ms: 100.0,
            average_energy: 10.0,
            last_used_at: None,
            created_at: now,
            updated_at: now,
        },
    )
    .expect("save cache");
    assert!(find_cached_plan(brain.store(), &parsed)
        .expect("find cache")
        .is_some());
}
