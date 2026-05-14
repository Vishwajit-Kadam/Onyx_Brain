use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ConversationIntent {
    Greeting,
    Question,
    Explanation,
    DebatePrompt,
    PlanningRequest,
    CritiqueRequest,
    DebugRequest,
    SummaryRequest,
    CreativeRequest,
    SafetyQuestion,
    #[default]
    Unknown,
}

pub fn detect_intent(input: &str) -> ConversationIntent {
    let lower = input.to_lowercase();
    let words = lower
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    if ["hello", "hi", "hey"]
        .iter()
        .any(|term| words.contains(term))
    {
        ConversationIntent::Greeting
    } else if lower.contains("debate") || lower.contains("should ") {
        ConversationIntent::DebatePrompt
    } else if lower.contains("explain") || lower.contains("teach") {
        ConversationIntent::Explanation
    } else if lower.contains("plan") || lower.contains("roadmap") {
        ConversationIntent::PlanningRequest
    } else if ["review", "critic", "critique"]
        .iter()
        .any(|term| lower.contains(term))
    {
        ConversationIntent::CritiqueRequest
    } else if ["error", "failed", "unresolved import", "panic", "compile"]
        .iter()
        .any(|term| lower.contains(term))
    {
        ConversationIntent::DebugRequest
    } else if lower.contains("summarize") || lower.contains("summary") {
        ConversationIntent::SummaryRequest
    } else if lower.contains("research") || lower.contains("outline") {
        ConversationIntent::Question
    } else if lower.contains("brainstorm") || lower.contains("ideas") {
        ConversationIntent::CreativeRequest
    } else if lower.contains("safe") || lower.contains("risk") {
        ConversationIntent::SafetyQuestion
    } else if lower.ends_with('?') {
        ConversationIntent::Question
    } else {
        ConversationIntent::Unknown
    }
}

pub fn extract_topic(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    for marker in ["about ", "for ", "on ", "understand "] {
        if let Some(start) = lower.find(marker) {
            let topic = input[start + marker.len()..]
                .trim_matches(|ch: char| ch == '"' || ch == '.' || ch == '?')
                .trim();
            if !topic.is_empty() {
                return Some(title_case(topic));
            }
        }
    }
    let cleaned = input
        .trim_matches(|ch: char| ch == '"' || ch == '.' || ch == '?')
        .trim();
    (!cleaned.is_empty()).then(|| title_case(cleaned))
}

pub fn extract_constraints(input: &str) -> Vec<String> {
    let lower = input.to_lowercase();
    let mut constraints = Vec::new();
    if lower.contains("beginner") || lower.contains("simple") {
        constraints.push("keep explanation beginner-friendly".to_string());
    }
    if lower.contains("safe") {
        constraints.push("emphasize safety boundaries".to_string());
    }
    if lower.contains("citation") || lower.contains("research") {
        constraints.push("use citation placeholders; do not fake sources".to_string());
    }
    constraints
}

pub fn extract_requested_format(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    if lower.contains("table") {
        Some("table".to_string())
    } else if lower.contains("checklist") {
        Some("checklist".to_string())
    } else if lower.contains("outline") {
        Some("outline".to_string())
    } else {
        None
    }
}

fn title_case(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
