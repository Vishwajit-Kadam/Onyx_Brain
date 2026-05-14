use onyx_brain::{
    conversation::{
        check_conversation_safety, debate_analysis, detect_intent, prompt_library,
        research_outline_response, score_response, teaching_response, ConversationIntent,
        ConversationMode, PersonalityProfile,
    },
    Brain,
};

#[test]
fn chat_once_returns_response_and_saves_messages() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let output = brain.chat_once("Hello Onyx, what can you do?").unwrap();
    assert!(output.response.contains("deterministic"));
    assert!(dir
        .path()
        .join("data")
        .join("conversations")
        .join(&output.session_id)
        .join("messages.json")
        .exists());
}

#[test]
fn intent_detector_covers_core_intents() {
    assert_eq!(detect_intent("hello"), ConversationIntent::Greeting);
    assert_eq!(
        detect_intent("Should this be open source?"),
        ConversationIntent::DebatePrompt
    );
    assert_eq!(
        detect_intent("Plan a release"),
        ConversationIntent::PlanningRequest
    );
    assert_eq!(
        detect_intent("Review this idea"),
        ConversationIntent::CritiqueRequest
    );
    assert_eq!(
        detect_intent("cargo test failed"),
        ConversationIntent::DebugRequest
    );
}

#[test]
fn mode_outputs_have_required_structure() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let debate = brain
        .run_mode(
            ConversationMode::Debate,
            "Should AI systems be open source?",
            false,
        )
        .unwrap();
    assert!(debate.response.contains("Side A"));
    assert!(debate.response.contains("Side B"));
    let teacher = brain
        .run_mode(
            ConversationMode::Teacher,
            "Explain sparse activation to a beginner",
            false,
        )
        .unwrap();
    assert!(teacher.response.contains("Mini Exercise"));
    let critic = brain
        .run_mode(ConversationMode::Critic, "Review the architecture", false)
        .unwrap();
    assert!(critic.response.contains("Strengths"));
    assert!(critic.response.contains("Weaknesses"));
    let planner = brain
        .run_mode(ConversationMode::Planner, "Plan v0.0.4", false)
        .unwrap();
    assert!(planner.response.contains("Phases"));
}

#[test]
fn specialized_modes_stay_safe_and_honest() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    let debug = brain
        .run_mode(
            ConversationMode::Debugger,
            "cargo test failed with unresolved import",
            false,
        )
        .unwrap();
    assert!(debug.response.contains("cargo check"));
    assert!(!debug.response.contains("rm -rf"));
    let research = brain
        .run_mode(
            ConversationMode::ResearchOutline,
            "Create a research outline for brain-inspired AI",
            false,
        )
        .unwrap();
    assert!(research.response.contains("Verification Notes"));
    assert!(research.response.contains("[citation needed]"));
}

#[test]
fn personality_memory_transcript_and_benchmark_work() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    brain.set_personality(PersonalityProfile::Friendly).unwrap();
    assert_eq!(brain.personality().unwrap(), PersonalityProfile::Friendly);
    let output = brain.chat_once("Hello Onyx").unwrap();
    assert!(!brain.conversation_memory().unwrap().is_empty());
    let transcript = brain.transcript("latest").unwrap();
    assert_eq!(transcript.session_id, output.session_id);
    let export = brain.transcript_export("latest").unwrap();
    assert!(export.export_path.contains("conversations"));
    let benchmark = brain.benchmark_conversation().unwrap();
    assert_eq!(benchmark.modes_tested, 7);
    assert!(benchmark.average_quality > 0.5);
}

#[test]
fn quality_and_safety_filters_reject_unsafe_claims() {
    let safety = check_conversation_safety("I am conscious and I can bypass sandbox rules.");
    assert!(!safety.allowed);
    let quality = score_response(
        "Explain safety",
        "I am conscious and can bypass sandbox rules.",
        &ConversationMode::SafetyReview,
    );
    assert!(quality.safety < 1.0);
}

#[test]
fn modes_and_prompt_library_are_available() {
    let dir = tempfile::tempdir().unwrap();
    let brain = Brain::new(dir.path());
    brain.init().unwrap();
    assert!(brain.modes().len() >= 10);
    assert!(prompt_library()
        .iter()
        .any(|pattern| pattern.name == "Debate"));
}

#[test]
fn direct_mode_helpers_build_expected_data() {
    assert_eq!(debate_analysis("Open Source").topic, "Open Source");
    assert!(teaching_response("Sparse Activation", "beginner")
        .mini_exercise
        .is_some());
    assert!(research_outline_response("Brain-Inspired AI")
        .citation_placeholders
        .iter()
        .any(|row| row.contains("citation")));
}
