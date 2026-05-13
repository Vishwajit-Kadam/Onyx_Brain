use onyx_brain::{
    agency::{
        capability_matrix, create_knowledge_gap_report, generate_done_definition, run_final_audit,
        status_from_error_budget, understand_goal, AutonomyLevel, DeliverableKind, ErrorBudget,
        GoalType,
    },
    artifacts::{
        add_cross_links, build_artifact_pack, build_for_deliverables, check_consistency,
        pitch_deck, product_spec, repair_consistency, technical_report, write_artifact,
        ArtifactKind,
    },
    Brain,
};

#[test]
fn advanced_goal_understanding_detects_launch_technical_and_product_packs() {
    let launch = understand_goal(
        "Create a startup launch package with pitch deck, landing page copy, FAQ, risk register, roadmap, release notes, social posts, demo script, metrics plan, launch checklist, and final report",
    );
    assert_eq!(launch.goal_type, GoalType::StartupPack);
    let kinds = launch
        .deliverables
        .iter()
        .map(|deliverable| deliverable.kind.clone())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&DeliverableKind::PitchDeck));
    assert!(kinds.contains(&DeliverableKind::LandingPageCopy));
    assert!(kinds.contains(&DeliverableKind::MetricsPlan));

    let technical =
        understand_goal("Create a technical report pack with architecture report and test plan");
    assert_eq!(technical.goal_type, GoalType::TechnicalReport);

    let product = understand_goal("Create a product spec / PRD with acceptance criteria");
    assert_eq!(product.goal_type, GoalType::ProductSpec);
}

#[test]
fn master_orchestrator_writes_contract_done_trace_and_audit() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let result = brain
        .autonomize(
            "Create a complete startup launch package for Onyx Brain v0.0.2 including pitch deck, speaker notes, landing page copy, FAQ, technical overview, risk register, roadmap, release notes, social posts, demo script, metrics plan, launch checklist, and final export package".to_string(),
            AutonomyLevel::FullBounded,
        )
        .unwrap();
    let workspace = dir
        .path()
        .join("sandbox")
        .join("workspaces")
        .join(&result.session_id);
    assert!(workspace.join("work_contract.json").exists());
    assert!(workspace.join("done_definition.json").exists());
    assert!(workspace.join("workspace_profile.json").exists());
    assert!(workspace.join("reports").join("final_audit.json").exists());
    assert!(workspace
        .join("reports")
        .join("execution_trace.md")
        .exists());
    assert!(dir
        .path()
        .join("data")
        .join("traces")
        .join(format!("{}_execution_trace.json", result.session_id))
        .exists());
}

#[test]
fn launch_technical_and_product_generators_create_required_content() {
    assert!(pitch_deck("Onyx Brain", 10).contains("## Slide 10"));
    assert!(technical_report("Onyx Brain").contains("## Design"));
    assert!(product_spec("Onyx Brain").contains("## Acceptance Criteria"));
}

#[test]
fn ppt_prompt_records_markdown_limitation_and_deck_files() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let result = brain
        .autonomize(
            "Make a PPT about brain-inspired AI with speaker notes".to_string(),
            AutonomyLevel::FullBounded,
        )
        .unwrap();
    let artifacts = dir
        .path()
        .join("sandbox")
        .join("workspaces")
        .join(&result.session_id)
        .join("artifacts");
    assert!(artifacts.join("presentation.md").exists());
    assert!(artifacts.join("speaker_notes.md").exists());
    assert!(artifacts.join("design_guide.md").exists());
    let limitations = std::fs::read_to_string(artifacts.join("limitations.md")).unwrap();
    assert!(limitations.contains("Binary PPTX export is not supported"));
}

#[test]
fn cross_links_and_consistency_repair_work() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let store = brain.store();
    let deck = write_artifact(
        store,
        "s1",
        ArtifactKind::PresentationMarkdown,
        "presentation.md",
        "# Deck\n\n## Slide 1\n- A\n\n## Slide 2\n- B\n",
        0.9,
    )
    .unwrap();
    let notes = write_artifact(
        store,
        "s1",
        ArtifactKind::SpeakerNotes,
        "speaker_notes.md",
        "# Notes\n\n## Slide 1\nOnly one note.\n",
        0.8,
    )
    .unwrap();
    let pack = build_artifact_pack(
        store,
        "s1",
        "Pack",
        None,
        &[deck.clone(), notes.clone()],
        Vec::new(),
        0.8,
    )
    .unwrap();
    let links = add_cross_links(store, &pack).unwrap();
    assert!(links.links_added > 0);
    let before = check_consistency(&[deck.path.clone(), notes.path.clone()]);
    assert!(!before.issues.is_empty());
    let repaired = repair_consistency(&[deck.path, notes.path]);
    assert!(repaired.issues_repaired >= 1);
}

#[test]
fn done_definition_final_audit_and_error_budget_behave() {
    let understanding = understand_goal("Create a product spec with final report");
    let done = generate_done_definition("s1", &understanding);
    assert!(!done.required_artifacts.is_empty());
    let completeness = onyx_brain::agency::DeliverableCompletenessReport {
        session_id: "s1".to_string(),
        required_deliverables: vec!["product_spec.md".to_string()],
        completed_deliverables: Vec::new(),
        missing_deliverables: vec!["product_spec.md".to_string()],
        incomplete_deliverables: Vec::new(),
        completion_score: 0.0,
    };
    let audit = run_final_audit("s1", &done, &completeness, false, 0.2, false);
    assert!(!audit.done_definition_met);
    let budget = ErrorBudget::default();
    assert_eq!(
        status_from_error_budget(&budget, 0, 0, 0, 1),
        "SafetyStopped"
    );
}

#[test]
fn local_research_knowledge_gaps_capabilities_and_export_manifest_work() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let result = brain
        .autonomize(
            "Create a launch kit with citations and final export package".to_string(),
            AutonomyLevel::FullBounded,
        )
        .unwrap();
    let understanding = understand_goal("Create a launch kit with citations");
    let gaps = create_knowledge_gap_report(
        brain.store(),
        &result.session_id,
        &understanding.original_prompt,
    )
    .unwrap();
    assert!(!gaps.gaps.is_empty());
    assert!(!capability_matrix().can_do.is_empty());

    let export = brain.export_package("latest").unwrap();
    let manifest = std::path::PathBuf::from(&export.export_path).join("export_manifest.json");
    assert!(manifest.exists());
    let manifest_text = std::fs::read_to_string(manifest).unwrap();
    assert!(manifest_text.contains("size_bytes"));
    assert!(manifest_text.contains("hash"));
}

#[test]
fn new_commands_data_paths_do_not_crash() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    brain.autonomize(
        "Create a technical report pack about Onyx Brain with architecture report and final report"
            .to_string(),
        AutonomyLevel::FullBounded,
    )
    .unwrap();
    assert!(!brain.capabilities().unwrap().cannot_do.is_empty());
    assert!(!brain.trace("latest").unwrap().events.is_empty());
    assert!(brain.autonomy_history().unwrap().count >= 1);
    let cleanup = brain.cleanup_autonomy().unwrap();
    assert!(cleanup.report_path.contains("autonomy_cleanup"));
}

#[test]
fn advanced_autonomy_benchmark_saves_report() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let report = brain.benchmark_advanced_autonomy().unwrap();
    assert!(report.report_path.contains("benchmark_advanced_autonomy"));
    assert!(std::path::PathBuf::from(report.report_path).exists());
}

#[test]
fn dependency_graph_accepts_new_deliverables() {
    let understanding = understand_goal(
        "Create a launch kit with pitch deck, speaker notes, roadmap, and final report",
    );
    let graph = build_for_deliverables(&understanding.deliverables);
    let order = graph.topological_order();
    assert!(!order.is_empty());
    assert!(graph.detect_cycles().is_empty());
}
