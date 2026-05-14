use serde::{Deserialize, Serialize};

use crate::conversation::{mode_name, ConversationMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPattern {
    pub id: String,
    pub name: String,
    pub trigger_keywords: Vec<String>,
    pub mode: ConversationMode,
    pub template_sections: Vec<String>,
    pub examples: Vec<String>,
}

pub fn prompt_library() -> Vec<PromptPattern> {
    vec![
        pattern(
            "debate",
            "Debate",
            ConversationMode::Debate,
            &["debate", "should"],
            &["topic", "two positions", "verdict"],
        ),
        pattern(
            "teacher",
            "Teacher",
            ConversationMode::Teacher,
            &["explain", "teach", "beginner"],
            &["topic", "level", "exercise"],
        ),
        pattern(
            "planner",
            "Planner",
            ConversationMode::Planner,
            &["plan", "roadmap"],
            &["goal", "phases", "risks"],
        ),
        pattern(
            "critic",
            "Critic",
            ConversationMode::Critic,
            &["review", "critique"],
            &["subject", "strengths", "weaknesses"],
        ),
        pattern(
            "debugger",
            "Debugger",
            ConversationMode::Debugger,
            &["error", "failed", "panic"],
            &["error text", "safe checks"],
        ),
        pattern(
            "architect",
            "Architect",
            ConversationMode::Architect,
            &["design", "architecture"],
            &["goal", "modules", "tradeoffs"],
        ),
        pattern(
            "research",
            "Research Outline",
            ConversationMode::ResearchOutline,
            &["research", "outline"],
            &["topic", "questions", "verification notes"],
        ),
        pattern(
            "artifact-pack",
            "Artifact Pack",
            ConversationMode::Planner,
            &["learning pack", "launch kit"],
            &["deliverables", "validation", "export"],
        ),
    ]
}

fn pattern(
    id: &str,
    name: &str,
    mode: ConversationMode,
    triggers: &[&str],
    sections: &[&str],
) -> PromptPattern {
    PromptPattern {
        id: id.to_string(),
        name: name.to_string(),
        trigger_keywords: triggers.iter().map(|row| row.to_string()).collect(),
        mode: mode.clone(),
        template_sections: sections.iter().map(|row| row.to_string()).collect(),
        examples: vec![format!(
            "cargo run -- mode {} \"{} about Onyx Brain\"",
            mode_name(&mode),
            triggers.first().copied().unwrap_or("ask")
        )],
    }
}
