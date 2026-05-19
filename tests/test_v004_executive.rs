use onyx_brain::{
    agency::AutonomyLevel,
    app_api::{load_events, record_event, AppApi, AppEventKind},
    executive::{
        attention_state, initialize_self_model, metacognitive_report, record_executive_decision,
    },
    Brain,
};

#[test]
fn app_api_status_chat_and_autonomy_paths_work() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    api.init().unwrap();
    let status = api.get_brain_status().unwrap();
    assert_eq!(status.version, "v0.0.4");
    let chat = api
        .send_chat_message(
            "Hello Onyx",
            onyx_brain::conversation::ConversationMode::Standard,
            onyx_brain::conversation::PersonalityProfile::Balanced,
        )
        .unwrap();
    assert!(chat.response.contains("deterministic"));
    let auto = api
        .run_autonomous_goal(
            "Create a 3-slide presentation about safety",
            AutonomyLevel::Executive,
        )
        .unwrap();
    assert!(auto.tasks_planned > 0);
}

#[test]
fn self_model_attention_metacognition_and_decisions_work() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let model = initialize_self_model("v0.0.4");
    assert!(model.name.contains("self-model"));
    assert!(model
        .limitations
        .iter()
        .any(|row| row.contains("not conscious")));
    let attention = attention_state(Some("test goal".to_string()), Some("task".to_string()));
    assert_eq!(attention.active_goal.as_deref(), Some("test goal"));
    let meta = metacognitive_report("Analyze current autonomous session");
    assert!(!meta.what_i_know.is_empty());
    assert!(!meta.what_i_do_not_know.is_empty());
    let decision =
        record_executive_decision(brain.store(), "session_test", "observed", "stop safely")
            .unwrap();
    assert!(decision.safety_checked);
    assert!(!brain
        .executive_status()
        .unwrap()
        .recent_decisions
        .is_empty());
}

#[test]
fn event_bus_records_events_and_benchmark_executive_runs() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    record_event(
        brain.store(),
        "session_events",
        AppEventKind::TaskCompleted,
        "done",
    )
    .unwrap();
    let events = load_events(brain.store(), "session_events").unwrap();
    assert_eq!(events.events.len(), 1);
    let bench = brain.benchmark_executive().unwrap();
    assert!(bench.safety_checked);
    assert!(bench.self_model_updated);
}
