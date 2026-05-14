use onyx_brain::{
    agency::{
        extract_features, load_project_registry, parse_goal, register_project, IntentKind,
        ProjectRecord,
    },
    tools::{diagnose_command, CodeEditorTool, CommandResult, DiagnosticKind},
    Brain,
};

#[test]
fn project_registry_saves_and_finds_project_by_name() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = onyx_brain::storage::DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let now = chrono::Utc::now();
    register_project(
        &store,
        ProjectRecord {
            goal_id: "goal1".to_string(),
            project_name: "calc_cli".to_string(),
            root_path: "sandbox/projects/calc_cli".to_string(),
            status: "Completed".to_string(),
            created_at: now,
            updated_at: now,
            last_report_path: None,
            tags: vec!["rust".to_string()],
            summary: "ok".to_string(),
        },
    )
    .expect("register");
    let registry = load_project_registry(&store).expect("registry");
    assert!(registry.find_by_name("calc_cli").is_some());
}

#[test]
fn parser_detects_create_and_modify_intents() {
    assert_eq!(
        parse_goal("Create a Rust project called calc_cli").intent,
        IntentKind::CreateProject
    );
    let parsed =
        parse_goal("Modify the calc_cli project to add multiply and divide functions with tests");
    assert_eq!(parsed.intent, IntentKind::ModifyProject);
    assert_eq!(parsed.project_name.as_deref(), Some("calc_cli"));
}

#[test]
fn parser_extracts_features_multiply_divide_tests() {
    let features = extract_features("add multiply and divide functions with tests");
    assert!(features.contains(&"multiply".to_string()));
    assert!(features.contains(&"divide".to_string()));
    assert!(features.contains(&"tests".to_string()));
}

#[test]
fn modification_command_updates_existing_calc_cli_project() {
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
    assert_eq!(output.intent, IntentKind::ModifyProject);
    assert_eq!(output.final_status, "Completed");
    assert!(output
        .files_modified
        .iter()
        .any(|file| file.contains("lib.rs")));
}

#[test]
fn modified_project_has_multiply_and_divide_functions_and_tests() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project(
            "Create a Rust CLI calculator project called calc_cli with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("create");
    brain
        .run_project(
            "Modify the calc_cli project to add multiply and divide functions with tests"
                .to_string(),
        )
        .expect("modify");
    let lib = std::fs::read_to_string(temp.path().join("sandbox/projects/calc_cli/src/lib.rs"))
        .expect("lib");
    let tests = std::fs::read_to_string(
        temp.path()
            .join("sandbox/projects/calc_cli/tests/calculator.rs"),
    )
    .expect("tests");
    assert!(lib.contains("fn multiply("));
    assert!(lib.contains("fn divide("));
    assert!(tests.contains("multiply_works"));
    assert!(tests.contains("divide_works"));
}

#[test]
fn cargo_test_passes_after_modification_if_available() {
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
    assert!(output.cargo_test_result.contains("passed"));
}

#[test]
fn diagnostics_detect_missing_function_and_type_mismatch() {
    let missing = CommandResult {
        command: vec!["cargo".to_string(), "check".to_string()],
        status: Some(1),
        stdout: String::new(),
        stderr: "error[E0425]: cannot find function `multiply` in this scope".to_string(),
        duration_ms: 0,
        allowed: true,
        sandbox_valid: true,
        executed_at: chrono::Utc::now(),
    };
    assert_eq!(
        diagnose_command(&missing).kind,
        DiagnosticKind::MissingFunction
    );
    let mismatch = CommandResult {
        command: vec!["cargo".to_string(), "check".to_string()],
        status: Some(1),
        stdout: String::new(),
        stderr: "error[E0308]: mismatched types".to_string(),
        duration_ms: 0,
        allowed: true,
        sandbox_valid: true,
        executed_at: chrono::Utc::now(),
    };
    assert_eq!(
        diagnose_command(&mismatch).kind,
        DiagnosticKind::TypeMismatch
    );
}

#[test]
fn code_editor_creates_backups_and_rejects_path_traversal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let editor = CodeEditorTool::new(temp.path()).expect("editor");
    editor
        .write_project_file("demo", "src/lib.rs", "pub fn one() -> i32 { 1 }\n")
        .expect("write");
    editor
        .write_project_file("demo", "src/lib.rs", "pub fn two() -> i32 { 2 }\n")
        .expect("rewrite");
    assert!(editor
        .write_project_file("demo", "../escape.rs", "no")
        .is_err());
    let backups = std::fs::read_dir(temp.path().join("projects/demo/src"))
        .expect("list")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".bak."))
        .count();
    assert!(backups >= 1);
}

#[test]
fn resume_latest_and_project_inspect_do_not_crash() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project("Create a Rust CLI project called resume_demo with README".to_string())
        .expect("project");
    let resumed = brain.resume_project("latest").expect("resume");
    assert_eq!(resumed.final_status, "Completed");
    let inspected = brain.project_inspect("resume_demo").expect("inspect");
    assert_eq!(inspected.project_name, "resume_demo");
}

#[test]
fn skill_extraction_creates_procedural_memory_without_duplicates() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project(
            "Create a Rust CLI calculator project called skill_demo with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("project");
    let first = brain
        .store()
        .memory_files()
        .expect("memories")
        .into_iter()
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("skill_")
        })
        .count();
    brain
        .run_project(
            "Modify the skill_demo project to add multiply and divide functions with tests"
                .to_string(),
        )
        .expect("modify");
    let second = brain
        .store()
        .memory_files()
        .expect("memories")
        .into_iter()
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("skill_")
        })
        .count();
    assert_eq!(first, second);
}

#[test]
fn self_evaluation_scores_successful_project_above_point_eight() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .run_project(
            "Create a Rust CLI calculator project called eval_demo with add and subtract functions, tests, and README"
                .to_string(),
        )
        .expect("project");
    assert!(output.self_evaluation.overall_score > 0.8);
}

#[test]
fn inspect_includes_project_registry_stats() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .run_project("Create a Rust CLI project called registry_demo with README".to_string())
        .expect("project");
    let summary = brain.inspect().expect("inspect");
    assert!(summary.registered_project_count >= 1);
    assert!(summary.last_modified_project.is_some());
}
