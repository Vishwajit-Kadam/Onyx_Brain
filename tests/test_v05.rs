use onyx_brain::{
    agency::parse_goal,
    energy::EnergyBudget,
    learning::{update_skill_usage, SkillReuseEngine},
    memory::{dedup::dedup_memories, MemoryItem, MemoryType},
    storage::{load_json, DiskStore},
    Brain,
};

fn skill(id: &str, title: &str, tags: &[&str]) -> MemoryItem {
    let mut memory = MemoryItem::new(
        id,
        MemoryType::Procedural,
        title,
        "Reusable Rust project workflow skill.",
        tags.iter().map(|tag| (*tag).to_string()).collect(),
        vec![],
    );
    memory.importance = 0.9;
    memory
}

#[test]
fn skill_matcher_finds_procedural_skill_for_rust_cli_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    store
        .save_memory(&skill(
            "skill_create",
            "Create Rust CLI project",
            &["rust", "project", "skill", "workflow", "cli"],
        ))
        .expect("save");
    let parsed = parse_goal("Create a Rust CLI calculator project called demo");
    let matches =
        SkillReuseEngine::find_relevant_skills(&store, &parsed, &[], &EnergyBudget::default())
            .expect("match");
    assert!(matches.iter().any(|skill| skill.title.contains("Rust CLI")));
}

#[test]
fn skill_matcher_finds_add_unit_tests_skill() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    store
        .save_memory(&skill(
            "skill_tests",
            "Add Rust unit tests",
            &["rust", "project", "skill", "workflow", "tests"],
        ))
        .expect("save");
    let parsed = parse_goal("Modify the demo project to add tests");
    let matches =
        SkillReuseEngine::find_relevant_skills(&store, &parsed, &[], &EnergyBudget::default())
            .expect("match");
    assert!(matches.iter().any(|skill| skill.title.contains("tests")));
}

#[test]
fn skill_reuse_updates_usage_metadata() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    store
        .save_memory(&skill(
            "skill_create",
            "Create Rust CLI project",
            &["rust", "project", "skill", "workflow", "cli"],
        ))
        .expect("save");
    let parsed = parse_goal("Create a Rust CLI project called demo");
    let matches =
        SkillReuseEngine::find_relevant_skills(&store, &parsed, &[], &EnergyBudget::default())
            .expect("match");
    update_skill_usage(&store, &matches, true, "goal1").expect("update");
    let memory = store.load_memory("skill_create").expect("load");
    assert_eq!(
        memory
            .metadata
            .get("usage_count")
            .and_then(|value| value.as_u64()),
        Some(1)
    );
}

#[test]
fn project_creation_after_skill_extraction_reuses_skills_and_trace_records_them() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project(
            "Create a Rust CLI calculator project called calc_cli with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("first");
    let output = brain
        .run_project(
            "Create a Rust CLI calculator project called skill_calc with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("second");
    assert!(!output.reused_skills.is_empty());
    let trace_path = temp
        .path()
        .join("data/logs")
        .join(format!("project_trace_{}.json", output.goal_id));
    let trace: onyx_brain::core::RouteTrace = load_json(&trace_path).expect("trace");
    assert!(!trace.reused_skills.is_empty());
}

#[test]
fn modification_after_skill_extraction_reuses_calculator_operation_skill() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project(
            "Create a Rust CLI calculator project called calc_cli with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("create");
    let output = brain
        .run_project(
            "Modify the calc_cli project to add multiply and divide functions with tests"
                .to_string(),
        )
        .expect("modify");
    assert!(output
        .reused_skills
        .iter()
        .any(|skill| skill.title.contains("calculator") || skill.title.contains("tests")));
}

#[test]
fn memory_dedup_detects_and_archives_duplicates_and_writes_report() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    store
        .save_memory(&skill(
            "skill_a",
            "Add Rust unit tests",
            &["rust", "project", "skill", "workflow"],
        ))
        .expect("save a");
    store
        .save_memory(&skill(
            "skill_b",
            "Add Rust unit tests",
            &["rust", "project", "skill", "workflow"],
        ))
        .expect("save b");
    store
        .save_memory(&MemoryItem::new(
            "project_a",
            MemoryType::Project,
            "Project calc_cli",
            "one",
            vec!["project".to_string(), "calc_cli".to_string()],
            vec![],
        ))
        .expect("project a");
    store
        .save_memory(&MemoryItem::new(
            "project_b",
            MemoryType::Project,
            "Project calc_cli",
            "two",
            vec!["project".to_string(), "calc_cli".to_string()],
            vec![],
        ))
        .expect("project b");
    let report = dedup_memories(&store).expect("dedup");
    assert!(report.duplicate_groups >= 2);
    assert!(report.memories_archived >= 2);
    assert!(std::path::Path::new(&report.report_path).exists());
}

#[test]
fn consolidate_invokes_dedup_or_updates_dedup_stats() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain.consolidate().expect("consolidate");
    assert!(std::fs::read_dir(temp.path().join("data/logs"))
        .expect("logs")
        .filter_map(Result::ok)
        .any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with("memory_dedup_report_")));
}

#[test]
fn benchmark_basic_runs_and_saves_report() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let report = brain.benchmark("basic").expect("benchmark");
    assert!(report.tasks_successful > 0);
    assert!(std::path::Path::new(&report.report_path).exists());
}

#[test]
fn inspect_reports_memory_hygiene_warning_when_duplicates_exist() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .store()
        .save_memory(&skill(
            "skill_a",
            "Add Rust unit tests",
            &["rust", "project", "skill", "workflow"],
        ))
        .expect("save a");
    brain
        .store()
        .save_memory(&skill(
            "skill_b",
            "Add Rust unit tests",
            &["rust", "project", "skill", "workflow"],
        ))
        .expect("save b");
    let summary = brain.inspect().expect("inspect");
    assert!(summary.memory_hygiene.duplicate_procedural_skills > 0);
    assert_eq!(summary.memory_hygiene.recommendation, "run memory-dedup");
}

#[test]
fn memory_inspect_command_data_does_not_crash_and_self_eval_has_skill_score() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let hygiene = brain.memory_inspect().expect("memory inspect");
    assert!(hygiene.total_memories >= 2);
    let output = brain
        .run_project(
            "Create a Rust CLI calculator project called eval_skill with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("project");
    assert!(output.self_evaluation.skill_reuse_score >= 0.5);
}
