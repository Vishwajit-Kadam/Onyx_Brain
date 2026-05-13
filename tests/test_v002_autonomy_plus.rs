use onyx_brain::{
    agency::{
        add_claim_caution, build_from_goal_understanding, build_report_card,
        check_deliverable_completeness, generate_self_questions, schedule_ready_tasks,
        understand_goal, AutonomyLevel, DeliverableKind, GraphTaskStatus,
    },
    artifacts::{check_consistency, documentation_pack_files, release_kit_files},
    memory::reflection::recent_reflections,
    Brain,
};

fn temp_brain() -> (tempfile::TempDir, Brain) {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    (temp, brain)
}

#[test]
fn task_graph_builds_dependency_order_and_scheduler_waits() {
    let goal =
        understand_goal("Create a learning pack about brain-inspired AI with quiz and glossary");
    let mut graph =
        build_from_goal_understanding("session_graph", Some("goal_graph".to_string()), &goal);
    let order = graph.topological_order();
    assert_eq!(order.first().map(String::as_str), Some("understand_goal"));
    let decisions = schedule_ready_tasks(&graph, 8);
    assert!(decisions
        .iter()
        .any(|decision| decision.task_id == "understand_goal" && decision.selected));
    assert!(decisions
        .iter()
        .any(|decision| decision.task_id.starts_with("generate_") && !decision.selected));
    graph.mark_task_completed("understand_goal");
    assert_eq!(graph.nodes[0].status, GraphTaskStatus::Completed);
}

#[test]
fn deliverable_completeness_detects_missing_required_artifact() {
    let (_temp, brain) = temp_brain();
    brain
        .autonomize(
            "Create a learning pack about brain-inspired AI with quiz and glossary".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("autonomize");
    let report = check_deliverable_completeness(
        brain.store(),
        "latest",
        &["quiz.md".to_string(), "missing_required.md".to_string()],
    )
    .expect("completeness");
    assert!(report
        .completed_deliverables
        .contains(&"quiz.md".to_string()));
    assert!(report
        .missing_deliverables
        .contains(&"missing_required.md".to_string()));
}

#[test]
fn release_docs_and_learning_pack_generators_are_selected() {
    let release = understand_goal(
        "Create a full launch kit with release notes, FAQ, demo script, and final report",
    );
    assert!(release
        .deliverables
        .iter()
        .any(|deliverable| deliverable.path_hint.as_deref() == Some("release_notes.md")));
    assert!(release_kit_files()
        .iter()
        .any(|(file, _)| *file == "github_release_draft.md"));

    let docs = understand_goal("Create a documentation pack with user guide and command guide");
    assert!(docs
        .deliverables
        .iter()
        .any(|deliverable| deliverable.kind == DeliverableKind::UserGuide));
    assert!(documentation_pack_files()
        .iter()
        .any(|(file, _)| *file == "command_reference.md"));

    let learning =
        understand_goal("Create a learning pack with lesson plan, quiz, answer key, and glossary");
    assert!(learning
        .deliverables
        .iter()
        .any(|deliverable| deliverable.path_hint.as_deref() == Some("answer_key.md")));
}

#[test]
fn self_questioning_claim_caution_consistency_and_report_card_work() {
    let questions = generate_self_questions("Create a 10-slide deck for students");
    assert!(questions.iter().any(|question| question.assumption_created));

    let cautioned = add_claim_caution("# Research Brief\n", true);
    assert!(cautioned.contains("Verification Notes"));
    assert!(cautioned.contains("Citation Placeholders"));

    let temp = tempfile::tempdir().expect("temp");
    let deck = temp.path().join("presentation.md");
    let notes = temp.path().join("speaker_notes.md");
    std::fs::write(&deck, "## Slide 1\n## Slide 2\n").expect("deck");
    std::fs::write(&notes, "## Slide 1\n").expect("notes");
    let consistency = check_consistency(&[deck.display().to_string(), notes.display().to_string()]);
    assert!(consistency.score < 1.0);

    let card = build_report_card("session", 0.9, 0.9, 0.8, 0.95, 1.0);
    assert!(matches!(card.overall_grade.as_str(), "A" | "B"));
}

#[test]
fn full_bounded_records_self_questions_reflection_and_export_reports() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .autonomize(
            "Create a full launch kit for Onyx Brain v0.0.2 including release notes, changelog entry, GitHub release draft, demo script, technical overview, FAQ, risk notes, social posts, launch checklist, and final report".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("launch kit");
    assert!(output.validation_passed);
    assert!(output
        .artifacts_created
        .iter()
        .any(|path| path.ends_with("artifact_pack.json")));
    let pack = brain.artifact_pack_inspect("latest").expect("pack");
    assert!(pack
        .artifacts
        .iter()
        .any(|row| row.contains("release_notes.md")));
    let export = brain.export_package("latest").expect("export");
    let export_inspection = brain.export_inspect("latest").expect("export inspect");
    assert_eq!(export.export_path, export_inspection.export_path);
    assert!(export_inspection
        .files
        .iter()
        .any(|file| file.ends_with("report_card.json")));
    let reflections = recent_reflections(brain.store()).expect("reflections");
    assert!(!reflections.is_empty());
}

#[test]
fn queue_run_and_artifacts_benchmark_work() {
    let (_temp, brain) = temp_brain();
    let queue = brain
        .queue_run("Create a learning pack about AI safety || Create a documentation pack for Onyx Brain commands || Run doctor")
        .expect("queue");
    assert_eq!(queue.goals_total, 3);
    assert!(queue.goals_completed >= 2);
    let benchmark = brain.benchmark_artifacts().expect("benchmark artifacts");
    assert!(std::path::Path::new(&benchmark.report_path).exists());
    assert!(benchmark.artifact_completion_rate > 0.0);
}

#[test]
fn command_aliases_do_not_crash() {
    let temp = tempfile::tempdir().expect("temp");
    let exe = env!("CARGO_BIN_EXE_onyx_brain");
    let root = temp.path();
    let run = |args: &[&str]| {
        let output = std::process::Command::new(exe)
            .current_dir(root)
            .args(args)
            .output()
            .expect("command");
        assert!(
            output.status.success(),
            "command {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    };
    run(&["init"]);
    run(&[
        "auto",
        "--level",
        "full-bounded",
        "Create a learning pack about AI safety with quiz and glossary",
    ]);
    run(&["packs"]);
    run(&["pack-inspect", "latest"]);
    run(&["report", "latest"]);
    run(&["auto-status"]);
    run(&["export", "latest"]);
}
