use onyx_brain::{
    agency::{
        fail_journal_entry, latest_journal_entries, session_path, snapshot_create,
        snapshot_restore, start_journal_entry, ActionType, FailureKind,
    },
    storage::DiskStore,
    tools::{simulate_failed_transaction, CodeEditorTool},
    Brain,
};

fn temp_brain() -> (tempfile::TempDir, Brain) {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    (temp, brain)
}

#[test]
fn action_journal_records_file_and_command_actions() {
    let (_temp, brain) = temp_brain();
    let store = brain.store();
    let mut file = start_journal_entry(
        store,
        "test_session",
        ActionType::CreateFile,
        None,
        Some("journal_demo".to_string()),
        Some("sandbox/projects/journal_demo/src/main.rs".to_string()),
        None,
        None,
        serde_json::json!({}),
    )
    .expect("start file journal");
    onyx_brain::agency::complete_journal_entry(store, &mut file, None, None).expect("complete");
    let mut command = start_journal_entry(
        store,
        "test_session",
        ActionType::RunCommand,
        None,
        Some("journal_demo".to_string()),
        None,
        Some("cargo check".to_string()),
        None,
        serde_json::json!({}),
    )
    .expect("start command journal");
    onyx_brain::agency::complete_journal_entry(store, &mut command, None, None).expect("complete");
    let rows = brain.journal(None).expect("journal");
    assert!(rows
        .iter()
        .any(|row| row.action_type == ActionType::CreateFile));
    assert!(rows
        .iter()
        .any(|row| row.action_type == ActionType::RunCommand));
}

#[test]
fn snapshot_create_and_restore_round_trip_file_content() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .run_project("Create a Rust CLI project called snap_calc with tests and README".to_string())
        .expect("project");
    assert_eq!(output.final_status, "Completed");
    let snapshot = snapshot_create(brain.store(), "snap_calc", "test snapshot").expect("snapshot");
    let editor = CodeEditorTool::new(&brain.store().paths.sandbox).expect("editor");
    editor
        .write_project_file("snap_calc", "README.md", "changed")
        .expect("change");
    let restore = snapshot_restore(brain.store(), &snapshot.snapshot_id).expect("restore");
    assert_eq!(restore.status, "Completed");
    let readme = editor
        .read_project_file("snap_calc", "README.md")
        .expect("readme");
    assert!(readme.contains("snap_calc"));
}

#[test]
fn rollback_latest_restores_previous_file_state_and_refuses_outside_sandbox() {
    let (_temp, brain) = temp_brain();
    brain
        .run_project("Create a Rust CLI project called roll_calc with tests".to_string())
        .expect("project");
    let editor = CodeEditorTool::new(&brain.store().paths.sandbox).expect("editor");
    editor
        .write_project_file("roll_calc", "README.md", "before")
        .expect("before");
    editor
        .write_project_file("roll_calc", "README.md", "after")
        .expect("after");
    let rollback = brain.rollback_latest(Some("roll_calc")).expect("rollback");
    assert_eq!(rollback.status, "Completed");
    let readme = editor
        .read_project_file("roll_calc", "README.md")
        .expect("readme");
    assert_eq!(readme, "before");

    let mut unsafe_entry = start_journal_entry(
        brain.store(),
        "unsafe",
        ActionType::ModifyFile,
        None,
        Some("unsafe".to_string()),
        Some("C:\\outside.txt".to_string()),
        None,
        Some("C:\\outside.bak".to_string()),
        serde_json::json!({}),
    )
    .expect("unsafe entry");
    onyx_brain::agency::complete_journal_entry(
        brain.store(),
        &mut unsafe_entry,
        None,
        Some("C:\\outside.bak".to_string()),
    )
    .expect("complete unsafe");
    assert!(brain.rollback_latest(Some("unsafe")).is_err());
}

#[test]
fn transactional_edit_commits_and_simulated_failure_is_recorded() {
    let (_temp, brain) = temp_brain();
    brain
        .run_project("Create a Rust CLI project called tx_calc with tests".to_string())
        .expect("project");
    let editor = CodeEditorTool::new(&brain.store().paths.sandbox).expect("editor");
    editor
        .write_project_file("tx_calc", "README.md", "transactional")
        .expect("write");
    let overview = brain.transactions().expect("transactions");
    assert!(overview.count > 0);
    let failed = simulate_failed_transaction(&brain.store().paths.sandbox, "tx_calc", "README.md")
        .expect("failed transaction");
    assert_eq!(failed.status, onyx_brain::tools::TransactionStatus::Failed);
}

#[test]
fn doctor_detects_repairs_and_archives_corrupt_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let missing = brain
        .store()
        .paths
        .indexes
        .join("action_journal_index.json");
    let _ = std::fs::remove_file(&missing);
    let report = brain.doctor(false).expect("doctor");
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.path.ends_with("action_journal_index.json")));
    let repaired = brain.doctor(true).expect("repair");
    assert!(repaired.repaired > 0);

    let registry = brain.store().paths.projects.join("project_registry.json");
    std::fs::write(&registry, "{not-json").expect("corrupt");
    let repaired = brain.doctor(true).expect("repair corrupt");
    assert!(repaired.critical > 0 || repaired.repaired > 0);
    assert!(brain
        .store()
        .paths
        .recovery
        .join("corrupt_archive")
        .exists());
}

#[test]
fn recovery_plan_and_recover_latest_are_conservative() {
    let (_temp, brain) = temp_brain();
    let plan = onyx_brain::agency::recovery_plan_for_failure(
        "sandbox violation",
        Some("safe_calc".to_string()),
        None,
    );
    assert_eq!(plan.failure_kind, FailureKind::SandboxViolation);
    assert!(!plan.safe_to_auto_run);

    let mut entry = start_journal_entry(
        brain.store(),
        "recover",
        ActionType::Unknown,
        None,
        Some("recover_calc".to_string()),
        Some("sandbox violation".to_string()),
        None,
        None,
        serde_json::json!({}),
    )
    .expect("entry");
    fail_journal_entry(brain.store(), &mut entry, "sandbox violation").expect("fail");
    let result = brain.recover_latest(Some("recover_calc")).expect("recover");
    assert!(!result.executed);
}

#[test]
fn sessions_start_status_end_and_resume_without_duplication() {
    let (_temp, brain) = temp_brain();
    let session = brain
        .session_start("Build calculator improvements".to_string())
        .expect("start");
    assert!(session_path(brain.store(), &session.session_id).exists());
    let status = brain.session_status(&session.session_id).expect("status");
    assert_eq!(status.session_id, session.session_id);
    let ended = brain.session_end(&session.session_id).expect("end");
    assert_eq!(ended.status, onyx_brain::agency::SessionStatus::Completed);
    let resumed = brain.session_resume(&session.session_id).expect("resume");
    assert_eq!(resumed.status, onyx_brain::agency::SessionStatus::Active);
    assert!(resumed
        .checkpoints
        .iter()
        .any(|checkpoint| checkpoint.contains("without duplicating")));
}

#[test]
fn worker_project_report_and_reliability_outputs_exist() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .worker("Create and improve a Rust calculator project called worker_calc".to_string())
        .expect("worker");
    assert_eq!(output.phases_completed, 5);
    assert!(output.failures.is_empty());
    let project = brain
        .projects()
        .expect("projects")
        .into_iter()
        .find(|project| project.project_name == "worker_calc")
        .expect("worker project");
    let json_report = brain
        .store()
        .paths
        .projects
        .join(project.goal_id)
        .join("final_report.json");
    assert!(json_report.exists());
    let markdown = std::fs::read_to_string(output.final_report).expect("report");
    assert!(markdown.contains("Reliability score"));
}

#[test]
fn regression_check_benchmark_status_and_inspect_include_reliability() {
    let (_temp, brain) = temp_brain();
    brain.init().expect("init");
    let regression = brain.regression_check().expect("regression");
    assert_eq!(regression.status, "pass");
    let benchmark = brain.benchmark_reliability().expect("benchmark");
    assert!(std::path::Path::new(&benchmark.report_path).exists());
    let status = brain.brain_status().expect("status");
    assert!(status.reliability_score.overall > 0.0);
    let inspect = brain.inspect_summary().expect("inspect");
    assert!(inspect.reliability_summary.contains("score"));
}

#[test]
fn stale_or_broken_index_reference_is_reported_without_crash() {
    let (_temp, brain) = temp_brain();
    let store = DiskStore::new(brain.store().paths.root.clone());
    std::fs::write(
        store.paths.indexes.join("plan_cache_index.json"),
        "{\"plans\":{\"missing\":{\"cache_id\":\"missing\"}}}",
    )
    .expect("stale");
    let report = brain.doctor(false).expect("doctor");
    assert!(report.issues_found <= report.issues.len());
    assert!(latest_journal_entries(brain.store(), 10, None).is_ok());
}
