use onyx_brain::{
    creative::{
        benchmark_creative, create_creative_project, parse_creative_project,
        validate_creative_workspace, CreativeProjectType,
    },
    Brain,
};

#[test]
fn creative_parser_detects_feature_film() {
    let project = parse_creative_project(
        "session",
        "Create a cinematic editing plan for a 3-hour original sci-fi feature film",
    );
    assert_eq!(project.project_type, CreativeProjectType::FeatureFilm);
    assert_eq!(project.duration_minutes, Some(180));
}

#[test]
fn creative_workflow_creates_production_deliverables() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let report = brain
        .creative("Create a cinematic editing plan for a 3-hour original sci-fi feature film with scene breakdown, timeline, sound design, VFX notes, color grading notes, and final production package")
        .unwrap();
    assert!(report.validation_passed);
    assert!(report
        .artifacts_created
        .iter()
        .any(|path| path.contains("timeline_plan.md")));
    assert!(report
        .artifacts_created
        .iter()
        .any(|path| path.contains("vfx_plan.md")));
}

#[test]
fn creative_validator_catches_missing_timeline() {
    let dir = tempfile::tempdir().unwrap();
    let store = onyx_brain::storage::DiskStore::new(dir.path());
    store.ensure_layout().unwrap();
    let project = parse_creative_project("session", "feature film");
    let artifacts = dir.path().join("artifacts");
    std::fs::create_dir_all(&artifacts).unwrap();
    let validation = validate_creative_workspace(&project, &artifacts).unwrap();
    assert!(!validation.passed);
    assert!(validation
        .issues
        .iter()
        .any(|issue| issue.contains("timeline_plan")));
}

#[test]
fn avatar_like_prompt_gets_originality_caution_and_no_render_claim() {
    let dir = tempfile::tempdir().unwrap();
    let store = onyx_brain::storage::DiskStore::new(dir.path());
    let report = create_creative_project(
        &store,
        "Create a cinematic plan like Avatar for an original sci-fi feature film",
    )
    .unwrap();
    assert!(report.originality_caution.unwrap().contains("do not copy"));
    let final_report = std::fs::read_to_string(report.final_report_path).unwrap();
    assert!(final_report.contains("No actual video was rendered"));
}

#[test]
fn benchmark_creative_runs_and_saves_report() {
    let dir = tempfile::tempdir().unwrap();
    let store = onyx_brain::storage::DiskStore::new(dir.path());
    let report = benchmark_creative(&store).unwrap();
    assert!(report.validation_passed);
    assert!(std::path::Path::new(&report.report_path).exists());
}
