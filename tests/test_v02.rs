use onyx_brain::{
    core::{brain::project_name_from_input, RouteTrace, Task, TaskType},
    experts::ExpertStatsIndex,
    memory::{retrieve_relevant_memories, score_memory, tokenize, MemoryItem, MemoryType},
    storage::{load_json, DiskStore},
    Brain,
};

#[test]
fn memory_keyword_scoring_rewards_overlap() {
    let memory = MemoryItem::new(
        "m1",
        MemoryType::Semantic,
        "Rust Ownership",
        "Ownership gives Rust memory safety.",
        vec![],
        vec![],
    );
    let task = Task::new("Explain Rust ownership".to_string(), TaskType::MemoryQuery);
    let score = score_memory(&memory, &task, &tokenize(&task.input), &[]);
    assert!(score > 0.25);
}

#[test]
fn memory_tag_scoring_rewards_tag_overlap() {
    let memory = MemoryItem::new(
        "m1",
        MemoryType::Semantic,
        "Ownership",
        "A concept.",
        vec!["rust".to_string(), "ownership".to_string()],
        vec![],
    );
    let task = Task::new("Rust ownership notes".to_string(), TaskType::MemoryQuery);
    let score = score_memory(&memory, &task, &tokenize(&task.input), &[]);
    assert!(score > 0.30);
}

#[test]
fn procedural_memory_is_preferred_for_code_tasks() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    store
        .save_memory(&MemoryItem::new(
            "semantic_rust_project",
            MemoryType::Semantic,
            "Rust Project",
            "Cargo creates Rust projects.",
            vec!["rust".to_string(), "project".to_string()],
            vec![],
        ))
        .expect("save semantic");
    store
        .save_memory(&MemoryItem::new(
            "procedural_rust_project",
            MemoryType::Procedural,
            "Create Rust Project",
            "Create Cargo.toml, create src/main.rs, run cargo check.",
            vec![
                "rust".to_string(),
                "cargo".to_string(),
                "project".to_string(),
            ],
            vec![],
        ))
        .expect("save procedural");

    let task = Task::new(
        "Create a Rust CLI project called hello_cli".to_string(),
        TaskType::Code,
    );
    let memories = retrieve_relevant_memories(&store, &task, &[], 2).expect("retrieve");
    assert_eq!(memories[0].id, "procedural_rust_project");
}

#[test]
fn inspect_command_data_does_not_crash() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let summary = brain.inspect().expect("inspect");
    assert_eq!(summary.neurons, 14);
    assert!(summary.top_strongest_synapses.len() <= 10);
}

#[test]
fn route_trace_is_saved_after_think() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .think("Create a Rust hello world project called mini_hello".to_string())
        .expect("think");
    let trace: RouteTrace = load_json(
        &temp
            .path()
            .join("data/logs")
            .join(format!("{}.json", output.task_id)),
    )
    .expect("trace");
    assert_eq!(trace.task_input, output.task);
    assert!(!trace.activated_synapses.is_empty());
}

#[test]
fn expert_confidence_updates_after_success() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .think("Create a Rust hello world project called mini_hello".to_string())
        .expect("think");
    let stats: ExpertStatsIndex =
        load_json(&temp.path().join("data/indexes/expert_stats.json")).expect("stats");
    let code = stats.0.get("CodeExpert").expect("code stats");
    assert!(code.total_runs >= 1);
    assert!(code.confidence > 0.5);
}

#[test]
fn project_name_extraction_handles_common_forms() {
    assert_eq!(
        project_name_from_input("Create a Rust hello world project called mini_hello"),
        "mini_hello"
    );
    assert_eq!(
        project_name_from_input("Create a Rust CLI project named test_app"),
        "test_app"
    );
    assert_eq!(
        project_name_from_input("Make a project called neuron_demo"),
        "neuron_demo"
    );
    assert!(project_name_from_input("Make a project").starts_with("project_"));
}

#[test]
fn checkpoint_file_is_created_with_v02_fields() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .think("Create a Rust hello world project called mini_hello".to_string())
        .expect("think");
    let checkpoint: serde_json::Value = load_json(
        &temp
            .path()
            .join("data/projects")
            .join(format!("{}_checkpoint.json", output.task_id)),
    )
    .expect("checkpoint");
    assert!(checkpoint.get("current_goal").is_some());
    assert!(checkpoint.get("planned_steps").is_some());
    assert!(checkpoint.get("failed_steps").is_some());
}

#[test]
fn self_review_reports_success_for_mini_hello() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .think("Create a Rust hello world project called mini_hello".to_string())
        .expect("think");
    assert!(output.self_review.success);
}

#[test]
fn budget_is_still_respected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .think("Create a Rust CLI project called hello_cli".to_string())
        .expect("think");
    assert!(output.activated_neurons.len() <= 32);
}
