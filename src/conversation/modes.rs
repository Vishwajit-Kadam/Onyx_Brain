use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ConversationMode {
    #[default]
    Standard,
    Debate,
    Teacher,
    Socratic,
    Critic,
    Planner,
    Architect,
    Debugger,
    ResearchOutline,
    Creative,
    Summarizer,
    SafetyReview,
    ProductManager,
    Coach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationModeInfo {
    pub mode: ConversationMode,
    pub name: String,
    pub description: String,
    pub response_style: String,
    pub rules: Vec<String>,
    pub output_format: Vec<String>,
}

pub fn available_modes() -> Vec<ConversationModeInfo> {
    use ConversationMode::*;
    [
        (Standard, "Helpful, concise, structured responses"),
        (
            Debate,
            "Present both sides, counterarguments, and a balanced verdict",
        ),
        (
            Teacher,
            "Explain simply with examples, exercises, and recap",
        ),
        (Socratic, "Guide with questions and hints"),
        (
            Critic,
            "Review strengths, weaknesses, risks, and improvements",
        ),
        (
            Planner,
            "Break goals into phases, tasks, dependencies, and next steps",
        ),
        (
            Architect,
            "Design modules, data flow, storage, safety, and tradeoffs",
        ),
        (
            Debugger,
            "Summarize errors and suggest safe diagnostic commands",
        ),
        (
            ResearchOutline,
            "Create research questions, source types, and verification notes",
        ),
        (
            Creative,
            "Brainstorm variations while staying within safety boundaries",
        ),
        (
            Summarizer,
            "Extract key points and actions from provided text",
        ),
        (
            SafetyReview,
            "Identify sandbox, privacy, permission, and command risks",
        ),
        (
            ProductManager,
            "Prioritize roadmap, user stories, and release planning",
        ),
        (
            Coach,
            "Provide encouraging structured advice and next steps",
        ),
    ]
    .into_iter()
    .map(|(mode, description)| mode_info(mode, description))
    .collect()
}

pub fn mode_info(mode: ConversationMode, description: &str) -> ConversationModeInfo {
    ConversationModeInfo {
        name: mode_name(&mode).to_string(),
        response_style: "clear sections, honest limits, no fake citations".to_string(),
        rules: vec![
            "do not claim consciousness, sentience, AGI, or LLM capability".to_string(),
            "do not imply web access or verified citations by default".to_string(),
            "prefer safe, bounded next steps".to_string(),
        ],
        output_format: default_format(&mode),
        mode,
        description: description.to_string(),
    }
}

pub fn mode_name(mode: &ConversationMode) -> &'static str {
    match mode {
        ConversationMode::Standard => "standard",
        ConversationMode::Debate => "debate",
        ConversationMode::Teacher => "teacher",
        ConversationMode::Socratic => "socratic",
        ConversationMode::Critic => "critic",
        ConversationMode::Planner => "planner",
        ConversationMode::Architect => "architect",
        ConversationMode::Debugger => "debugger",
        ConversationMode::ResearchOutline => "research-outline",
        ConversationMode::Creative => "creative",
        ConversationMode::Summarizer => "summarizer",
        ConversationMode::SafetyReview => "safety-review",
        ConversationMode::ProductManager => "product-manager",
        ConversationMode::Coach => "coach",
    }
}

pub fn parse_mode(input: &str) -> Option<ConversationMode> {
    let normalized = input.to_lowercase().replace('_', "-");
    available_modes()
        .into_iter()
        .find(|row| row.name == normalized)
        .map(|row| row.mode)
}

fn default_format(mode: &ConversationMode) -> Vec<String> {
    match mode {
        ConversationMode::Debate => vec![
            "Topic".to_string(),
            "Side A".to_string(),
            "Side B".to_string(),
            "Counterarguments".to_string(),
            "Balanced verdict".to_string(),
        ],
        ConversationMode::Teacher => vec![
            "Explanation".to_string(),
            "Example".to_string(),
            "Mini exercise".to_string(),
            "Recap".to_string(),
        ],
        ConversationMode::Socratic => vec![
            "Current question".to_string(),
            "Why this question".to_string(),
            "Hint".to_string(),
        ],
        _ => vec![
            "Summary".to_string(),
            "Details".to_string(),
            "Next step".to_string(),
        ],
    }
}
