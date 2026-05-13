use onyx_brain::{
    agency::{
        create_workspace, default_assumptions, default_limitations, match_recipe, record_progress,
        review_artifact_pack, run_revision_cycle, understand_goal, write_assumptions,
        write_limitations, AutonomyLevel, DeliverableKind,
    },
    artifacts::{
        build_for_deliverables, glossary, quiz, risk_register, roadmap, study_guide,
        ArtifactDependencyGraph, ArtifactEdge,
    },
    memory::self_critique::save_self_critique,
    Brain,
};

fn temp_brain() -> (tempfile::TempDir, Brain) {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    (temp, brain)
}

#[test]
fn goal_understanding_detects_learning_pack_deliverables() {
    let goal = understand_goal(
        "Create a learning pack about brain-inspired AI with slides, notes, quiz, glossary, and study plan",
    );
    let kinds = goal
        .deliverables
        .iter()
        .map(|deliverable| deliverable.kind.clone())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&DeliverableKind::PresentationMarkdown));
    assert!(kinds.contains(&DeliverableKind::StudyGuide));
    assert!(kinds.contains(&DeliverableKind::Quiz));
    assert!(kinds.contains(&DeliverableKind::Glossary));
}

#[test]
fn dependency_graph_orders_speaker_notes_after_slides_and_detects_cycles() {
    let goal = understand_goal("Create a presentation about safety with speaker notes");
    let graph = build_for_deliverables(&goal.deliverables);
    let order = graph.topological_order();
    let presentation = order.iter().position(|id| id == "presentation").unwrap();
    let notes = order.iter().position(|id| id == "speaker_notes").unwrap();
    assert!(presentation < notes);

    let cyclic = ArtifactDependencyGraph {
        nodes: graph.nodes.iter().take(2).cloned().collect(),
        edges: vec![
            ArtifactEdge {
                from: "presentation".to_string(),
                to: "speaker_notes".to_string(),
                relation: "test".to_string(),
            },
            ArtifactEdge {
                from: "speaker_notes".to_string(),
                to: "presentation".to_string(),
                relation: "test".to_string(),
            },
        ],
    };
    assert!(!cyclic.detect_cycles().is_empty());
}

#[test]
fn deterministic_generators_include_required_sections() {
    assert!(study_guide("Brain-Inspired AI").contains("## Review Questions"));
    assert!(quiz("Brain-Inspired AI").contains("## Answer Key"));
    assert!(glossary("Brain-Inspired AI").contains("Sparse activation"));
    assert!(roadmap("Brain-Inspired AI").contains("## Milestones"));
    assert!(risk_register("Brain-Inspired AI").contains("| Risk | Severity |"));
}

#[test]
fn autonomize_learning_pack_creates_pack_manifest_and_workspace() {
    let (_temp, brain) = temp_brain();
    let output = brain
        .autonomize(
            "Create a complete learning pack about brain-inspired AI for students with a 10-slide deck, speaker notes, study guide, quiz, glossary, design guide, and final report".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("autonomize");
    assert!(output.validation_passed);
    assert!(output
        .artifacts_created
        .iter()
        .any(|path| path.ends_with("artifact_pack.json")));
    let pack = brain.artifact_pack_inspect("latest").expect("pack inspect");
    assert!(pack.artifacts.iter().any(|row| row.contains("quiz.md")));
    assert!(pack.artifacts.iter().any(|row| row.contains("glossary.md")));
    let workspace = brain
        .workspace_inspect("latest")
        .expect("workspace inspect");
    assert!(std::path::Path::new(&workspace.root_path)
        .join("artifacts")
        .exists());
}

#[test]
fn quality_review_and_revision_fix_missing_answer_key() {
    let (_temp, brain) = temp_brain();
    brain
        .autonomize(
            "Create a learning pack about brain-inspired AI with quiz and glossary".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("autonomize");
    let pack = brain.artifact_pack_inspect("latest").expect("pack");
    let quiz_path = pack
        .artifacts
        .iter()
        .find(|row| row.contains("quiz.md"))
        .unwrap()
        .split('|')
        .next()
        .unwrap()
        .trim()
        .to_string();
    let without_key = std::fs::read_to_string(&quiz_path)
        .expect("quiz")
        .replace("## Answer Key", "## Removed Key");
    std::fs::write(&quiz_path, without_key).expect("remove key");
    let review = review_artifact_pack(brain.store(), "latest").expect("review");
    assert!(review
        .issues
        .iter()
        .any(|issue| issue.issue_id == "missing_answer_key"));
    let revision = run_revision_cycle(brain.store(), &review).expect("revision");
    assert!(revision.issues_fixed >= 1);
    assert!(std::fs::read_to_string(&quiz_path)
        .expect("quiz fixed")
        .contains("## Answer Key"));
}

#[test]
fn assumptions_limitations_workspace_recipes_and_progress_are_disk_backed() {
    let (_temp, brain) = temp_brain();
    create_workspace(brain.store(), "session_demo").expect("workspace");
    let assumptions = default_assumptions("session_demo", "Create 10 slides for students", 10);
    let limitations = default_limitations("session_demo");
    let (assumptions_md, _) = write_assumptions(brain.store(), &assumptions).expect("assumptions");
    let (limitations_md, _) = write_limitations(brain.store(), &limitations).expect("limitations");
    assert!(std::path::Path::new(&assumptions_md).exists());
    assert!(std::path::Path::new(&limitations_md).exists());
    let recipe = match_recipe(brain.store(), "Create a learning pack with quiz").expect("recipe");
    assert!(recipe.unwrap().title.contains("Learning"));
    let progress = record_progress(
        brain.store(),
        "session_demo",
        "1/1",
        "test",
        "completed",
        1.0,
        "done",
    )
    .expect("progress");
    assert_eq!(progress.session_id, "session_demo");
}

#[test]
fn review_only_does_not_modify_artifacts_and_repair_only_fixes_validation_issue() {
    let (_temp, brain) = temp_brain();
    brain
        .autonomize(
            "Create a learning pack about brain-inspired AI with quiz and glossary".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("autonomize");
    let pack = brain.artifact_pack_inspect("latest").expect("pack");
    let quiz_path = pack
        .artifacts
        .iter()
        .find(|row| row.contains("quiz.md"))
        .unwrap()
        .split('|')
        .next()
        .unwrap()
        .trim()
        .to_string();
    let before = std::fs::read_to_string(&quiz_path).expect("before");
    brain
        .autonomize(
            "Review latest artifact pack".to_string(),
            AutonomyLevel::ReviewOnly,
        )
        .expect("review only");
    assert_eq!(
        before,
        std::fs::read_to_string(&quiz_path).expect("after review")
    );
    std::fs::write(
        &quiz_path,
        before.replace("## Answer Key", "## Removed Key"),
    )
    .expect("break quiz");
    brain
        .autonomize(
            "Repair latest artifact pack".to_string(),
            AutonomyLevel::RepairOnly,
        )
        .expect("repair only");
    assert!(std::fs::read_to_string(&quiz_path)
        .expect("after repair")
        .contains("## Answer Key"));
}

#[test]
fn self_critique_autonomy_status_benchmark_and_export_work() {
    let (_temp, brain) = temp_brain();
    brain
        .autonomize(
            "Create a complete learning pack about brain-inspired AI with a 5-slide deck, speaker notes, study guide, quiz, glossary, design guide, and final report".to_string(),
            AutonomyLevel::FullBounded,
        )
        .expect("autonomize");
    let critique = save_self_critique(
        brain.store(),
        "session_demo",
        "Quiz needed an answer key",
        "Add answer key by default",
        true,
    )
    .expect("critique");
    assert!(critique.success);
    let status = brain.autonomy_status().expect("status");
    assert!(status.artifact_packs >= 1);
    let export = brain.export_package("latest").expect("export");
    assert!(std::path::Path::new(&export.export_path).exists());
    let exports = brain.exports().expect("exports");
    assert!(exports.count >= 1);
    let benchmark = brain.benchmark_autonomy().expect("benchmark");
    assert!(benchmark.average_quality_score > 0.0);
    assert!(benchmark.artifact_completion_rate > 0.0);
}
