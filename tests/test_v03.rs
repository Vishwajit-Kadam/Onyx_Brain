use onyx_brain::{
    agency::{
        decompose_goal, load_project_state, load_task_queue, retry_allowed, save_project_state,
        save_task_queue, ProjectState,
    },
    tools::{diagnose_command, CodeEditorTool, CommandResult, DiagnosticKind},
    Brain,
};

#[test]
fn goal_decomposes_into_task_queue() {
    let queue = decompose_goal(
        "goal1",
        "Create a Rust CLI calculator project called calc_cli with tests and README",
    );
    assert!(queue.iter().any(|task| task.title == "Write tests"));
    assert!(queue.iter().any(|task| task.title == "Write README"));
    assert!(queue.iter().any(|task| task.title == "Run cargo check"));
}

#[test]
fn task_queue_persists_to_disk() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = onyx_brain::storage::DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let queue = decompose_goal("goal1", "Create a Rust project called demo");
    save_task_queue(&store, "goal1", &queue).expect("save queue");
    let loaded = load_task_queue(&store, "goal1").expect("load queue");
    assert_eq!(loaded.len(), queue.len());
}

#[test]
fn project_state_persists_to_disk() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = onyx_brain::storage::DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let state = ProjectState::new("goal1", "demo", "sandbox/projects/demo", "make demo");
    save_project_state(&store, &state).expect("save state");
    let loaded = load_project_state(&store, "goal1").expect("load state");
    assert_eq!(loaded.project_name, "demo");
}

#[test]
fn code_editor_rejects_path_traversal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let editor = CodeEditorTool::new(temp.path()).expect("editor");
    assert!(editor
        .write_project_file("demo", "../escape.rs", "fn main() {}")
        .is_err());
}

#[test]
fn code_editor_can_write_and_replace_sandbox_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let editor = CodeEditorTool::new(temp.path()).expect("editor");
    editor
        .write_project_file("demo", "src/lib.rs", "pub fn value() -> i32 { 1 }\n")
        .expect("write");
    assert!(editor
        .replace_in_project_file("demo", "src/lib.rs", "1", "2")
        .expect("replace"));
    let content = editor
        .read_project_file("demo", "src/lib.rs")
        .expect("read");
    assert!(content.contains("{ 2 }"));
}

#[test]
fn diagnostics_detect_cargo_success() {
    let command = CommandResult {
        command: vec!["cargo".to_string(), "check".to_string()],
        status: Some(0),
        stdout: "Finished".to_string(),
        stderr: String::new(),
        duration_ms: 0,
        allowed: true,
        sandbox_valid: true,
        executed_at: chrono::Utc::now(),
    };
    let report = diagnose_command(&command);
    assert_eq!(report.kind, DiagnosticKind::CargoCheckPassed);
}

#[test]
fn diagnostics_detect_simple_rust_syntax_error_text() {
    let command = CommandResult {
        command: vec!["cargo".to_string(), "check".to_string()],
        status: Some(1),
        stdout: String::new(),
        stderr: "error: expected `;`, found `}`".to_string(),
        duration_ms: 0,
        allowed: true,
        sandbox_valid: true,
        executed_at: chrono::Utc::now(),
    };
    let report = diagnose_command(&command);
    assert_eq!(report.kind, DiagnosticKind::SyntaxError);
}

#[test]
fn retry_limit_is_respected() {
    assert!(retry_allowed(0, 2));
    assert!(retry_allowed(1, 2));
    assert!(!retry_allowed(2, 2));
}

#[test]
fn project_command_creates_project_with_readme_and_tests() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .run_project(
            "Create a Rust CLI calculator project called calc_cli with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("project");
    assert_eq!(output.project_name, "calc_cli");
    assert_eq!(output.final_status, "Completed");
    assert!(temp
        .path()
        .join("sandbox/projects/calc_cli/README.md")
        .exists());
    assert!(temp
        .path()
        .join("sandbox/projects/calc_cli/tests/calculator.rs")
        .exists());
}

#[test]
fn project_command_runs_cargo_check_or_test_if_available() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .run_project(
            "Create a Rust CLI calculator project called calc_cli with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("project");
    assert!(output.cargo_check_result.contains("passed"));
    assert!(output.cargo_test_result.contains("passed"));
}

#[test]
fn inspect_includes_project_status() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project("Create a Rust CLI project called inspect_demo with README".to_string())
        .expect("project");
    let summary = brain.inspect().expect("inspect");
    assert!(summary
        .known_projects
        .iter()
        .any(|row| row.contains("inspect_demo")));
}

#[test]
fn consolidation_creates_procedural_memory_from_successful_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project("Create a Rust CLI project called workflow_demo with README".to_string())
        .expect("project");
    brain.consolidate().expect("consolidate");
    assert!(temp
        .path()
        .join("data/memories/procedural_project_workflow_workflow_demo.json")
        .exists());
}
