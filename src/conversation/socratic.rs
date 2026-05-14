use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocraticResponse {
    pub topic: String,
    pub current_question: String,
    pub why_this_question: String,
    pub hint: String,
    pub possible_next_steps: Vec<String>,
}

pub fn socratic_response(topic: &str) -> SocraticResponse {
    SocraticResponse {
        topic: topic.to_string(),
        current_question: format!("What problem do you think {topic} is trying to solve?"),
        why_this_question: "Starting with the problem makes the design easier to reason about.".to_string(),
        hint: "Look for what changes when the system has too much state, too many choices, or unclear memory boundaries.".to_string(),
        possible_next_steps: vec![
            "Answer the question in one sentence.".to_string(),
            "Name one example from Onyx Brain.".to_string(),
        ],
    }
}

pub fn render_socratic(response: &SocraticResponse) -> String {
    format!(
        "# Socratic Mode\n\nTopic: {}\n\n## Current Question\n{}\n\n## Why This Question\n{}\n\n## Hint\n{}\n\n## Possible Next Steps\n{}\n",
        response.topic,
        response.current_question,
        response.why_this_question,
        response.hint,
        response
            .possible_next_steps
            .iter()
            .map(|row| format!("- {row}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
