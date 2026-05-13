use onyx_brain::{
    agency::{
        discover_local_context, plan_autonomous_work, repair_presentation_artifacts,
        requested_slide_count, understand_goal, validate_presentation_artifacts, AutonomyLevel,
        GoalType, WorkerStatus,
    },
    artifacts::{
        artifact_session_dir, build_presentation, render_design_guide,
        render_presentation_markdown, write_artifact, ArtifactKind,
    },
    Brain,
};

fn temp_brain() -> (tempfile::TempDir, Brain) {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    (temp, brain)
}

#[test]
fn goal_understanding_detects_presentation_and_code_tasks() {
    let presentation =
        understand_goal("Create a 10-slide presentation about brain-inspired AI with notes");
    assert_eq!(presentation.goal_type, GoalType::Presentation);
    assert!(presentation.needs_artifact);
    assert_eq!(requested_slide_count(&presentation.original_prompt), 10);

    let code = understand_goal("make a Rust project called demo_cli");
    assert_eq!(code.goal_type, GoalType::CodeProject);
    assert!(code.needs_code);
}

#[test]
fn autonomous_planner_creates_phases() {
    let understanding = understand_goal("Create a presentation about AI safety");
    let plan = plan_autonomous_work("goal_demo", &understanding);
    assert!(!plan.phases.is_empty());
    assert!(plan
        .phases
        .iter()
        .any(|phase| phase.title.contains("create slide content")));
}

#[test]
fn autonomize_presentation_creates_artifacts_manifest_and_report() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .autonomize(
            "Create a 10-slide presentation about brain-inspired AI for students with speaker notes and a design guide".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("autonomize");
    assert!(matches!(
        output.status,
        WorkerStatus::Completed | WorkerStatus::CompletedWithWarnings
    ));
    assert!(output.validation_passed);
    assert!(output.autonomy_score > 0.8);
    assert!(output
        .artifacts_created
        .iter()
        .any(|path| path.ends_with("artifact_manifest.json")));
    assert!(std::path::Path::new(&output.final_report_path).exists());
}

#[test]
fn presentation_validator_catches_and_repairs_missing_speaker_notes() {
    let (_temp, brain) = temp_brain();
    let session_id = "repair_demo";
    let presentation = build_presentation("Brain-Inspired AI", "students", 3);
    write_artifact(
        brain.store(),
        session_id,
        ArtifactKind::PresentationMarkdown,
        "presentation.md",
        &render_presentation_markdown(&presentation),
        0.5,
    )
    .expect("presentation");
    write_artifact(
        brain.store(),
        session_id,
        ArtifactKind::DesignGuide,
        "design_guide.md",
        &render_design_guide(&presentation),
        0.5,
    )
    .expect("design");
    let dir = artifact_session_dir(brain.store(), session_id);
    let validation = validate_presentation_artifacts(&dir, 3, Some(&presentation));
    assert!(!validation.passed);
    assert!(validation
        .issues
        .iter()
        .any(|issue| issue.message.contains("speaker notes")));
    let repairs =
        repair_presentation_artifacts(brain.store(), session_id, &presentation, &validation)
            .expect("repair");
    assert!(repairs >= 1);
    assert!(dir.join("speaker_notes.md").exists());
}

#[test]
fn full_bounded_mode_and_limits_behave_safely() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .autonomize(
            "Create 100 tasks for an excessive autonomous run".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("safety stop");
    assert_eq!(output.status, WorkerStatus::SafetyStopped);
    assert!(!output.validation_passed);
}

#[test]
fn context_discovery_stays_inside_allowed_paths() {
    let (temp, brain) = temp_brain();
    std::fs::write(temp.path().join("outside_secret.md"), "secret").expect("outside");
    let context = discover_local_context(brain.store(), 10).expect("context");
    assert!(context
        .files_considered
        .iter()
        .all(|path| path.starts_with(&temp.path().display().to_string())));
    assert!(context
        .files_considered
        .iter()
        .all(|path| !path.contains("outside_secret")));
}

#[test]
fn artifact_and_session_report_commands_do_not_crash() {
    let (_temp, brain) = temp_brain();
    brain
        .autonomize(
            "Create a 5-slide presentation about bounded autonomy with speaker notes".to_string(),
            AutonomyLevel::Standard,
        )
        .expect("autonomize");
    let artifacts = brain.artifacts().expect("artifacts");
    assert!(artifacts.count >= 1);
    let inspection = brain.artifact_inspect("latest").expect("inspect");
    assert!(inspection.manifest_path.is_some());
    let session = brain.session_report("latest").expect("session report");
    assert!(std::path::Path::new(&session.markdown_report_path).exists());
}

#[test]
fn autonomy_policy_benchmark_doctor_and_status_include_autonomy() {
    let (_temp, brain) = temp_brain();
    let policy = brain.autonomy_policy().expect("policy");
    assert!(!policy.limits.network_allowed);
    let benchmark = brain.benchmark_autonomy().expect("benchmark");
    assert!(std::path::Path::new(&benchmark.report_path).exists());
    assert!(benchmark.autonomy_score > 0.0);
    let doctor = brain.doctor(false).expect("doctor");
    assert_eq!(doctor.critical, 0);
    let status = brain.brain_status().expect("status");
    assert!(status.artifacts_count >= 1);
    assert!(status.autonomous_sessions_count >= 1);
}
