use serde::{Deserialize, Serialize};

use crate::conversation::{
    architecture_response, critique_response, debate_analysis, debugger_response, detect_intent,
    extract_constraints, extract_requested_format, extract_topic, planning_response,
    render_architecture, render_critique, render_debate, render_debugger, render_plan,
    render_research_outline, render_socratic, render_teaching, research_outline_response,
    socratic_response, teaching_response, ConversationIntent, ConversationMode,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePlan {
    pub mode: ConversationMode,
    pub intent: ConversationIntent,
    pub sections: Vec<ResponseSection>,
    pub tone: ResponseTone,
    pub needs_disclaimer: bool,
    pub needs_followup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSection {
    pub title: String,
    pub bullets: Vec<String>,
    pub paragraphs: Vec<String>,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseTone {
    Friendly,
    Technical,
    Simple,
    Direct,
    Encouraging,
    Neutral,
}

pub fn generate_response_plan(mode: ConversationMode, input: &str) -> ResponsePlan {
    let intent = detect_intent(input);
    let tone = if input.to_lowercase().contains("beginner") {
        ResponseTone::Simple
    } else if matches!(
        mode,
        ConversationMode::Architect | ConversationMode::Debugger
    ) {
        ResponseTone::Technical
    } else {
        ResponseTone::Friendly
    };
    ResponsePlan {
        mode,
        intent: intent.clone(),
        sections: vec![ResponseSection {
            title: "Summary".to_string(),
            bullets: extract_constraints(input),
            paragraphs: extract_requested_format(input).into_iter().collect(),
            priority: 1,
        }],
        tone,
        needs_disclaimer: input.to_lowercase().contains("research")
            || input.to_lowercase().contains("citation"),
        needs_followup: matches!(intent, ConversationIntent::Question),
    }
}

pub fn render_response(plan: &ResponsePlan, input: &str) -> String {
    let topic = extract_topic(input).unwrap_or_else(|| "This Topic".to_string());
    let body = match plan.mode {
        ConversationMode::Debate => render_debate(&debate_analysis(&topic)),
        ConversationMode::Teacher => render_teaching(&teaching_response(&topic, input)),
        ConversationMode::Socratic => render_socratic(&socratic_response(&topic)),
        ConversationMode::Critic => render_critique(&critique_response(&topic)),
        ConversationMode::Planner | ConversationMode::ProductManager => {
            render_plan(&planning_response(&topic))
        }
        ConversationMode::Architect => render_architecture(&architecture_response(&topic)),
        ConversationMode::Debugger => render_debugger(&debugger_response(input)),
        ConversationMode::ResearchOutline => {
            render_research_outline(&research_outline_response(&topic))
        }
        ConversationMode::Creative => creative_response(&topic),
        ConversationMode::Summarizer => summarizer_response(input),
        ConversationMode::SafetyReview => safety_review_response(input),
        ConversationMode::Coach => coach_response(&topic),
        ConversationMode::Standard => standard_response(&topic, input, &plan.intent),
    };
    if plan.needs_disclaimer && !body.contains("Verification Notes") {
        format!("{body}\n\n## Verification Notes\nNo web access was used by default. Treat factual claims as placeholders until externally verified.\n")
    } else {
        body
    }
}

fn standard_response(topic: &str, input: &str, intent: &ConversationIntent) -> String {
    if matches!(intent, ConversationIntent::Greeting) {
        "# Standard Mode\n\nHello. I’m Onyx Brain’s deterministic conversation layer. I can explain, debate, teach, critique, plan, outline research, and help debug safely. I do not claim consciousness, AGI, sentience, or LLM capability.\n\n## Useful Commands\n- `mode teacher \"...\"`\n- `mode debate \"...\"`\n- `mode planner \"...\"`\n- `mode debugger \"...\"`\n".to_string()
    } else {
        format!("# Standard Mode\n\n## Summary\n{topic}\n\n## Response\nI can help structure this locally and safely. Based on your prompt, the useful path is to clarify the goal, identify constraints, choose a mode, and produce a checked response.\n\n## Safety Note\nNo web access or external verification is used by default, and this layer is deterministic rather than an LLM.\n\nPrompt considered: {input}\n")
    }
}

fn creative_response(topic: &str) -> String {
    format!("# Creative Mode\n\n## Ideas for {topic}\n- Make a compact explainer.\n- Turn it into a checklist.\n- Create a debate map.\n- Build a beginner exercise.\n\n## Boundary\nCreative output stays local, deterministic, and safety-framed.\n")
}

fn summarizer_response(input: &str) -> String {
    format!("# Summarizer Mode\n\n## Key Points\n- {}\n\n## Action Items\n- Identify the main claim.\n- Extract unresolved questions.\n- Decide the next safe step.\n", input.lines().next().unwrap_or(input))
}

fn safety_review_response(input: &str) -> String {
    format!("# Safety Review Mode\n\n## Risks Checked\n- Sandbox bypass\n- Unrestricted shell\n- Fake citations\n- Secret extraction\n- Unsupported AGI/consciousness claims\n\n## Result\nNo action should bypass safety boundaries. Review the prompt carefully before connecting tools.\n\nPrompt: {input}\n")
}

fn coach_response(topic: &str) -> String {
    format!("# Coach Mode\n\n## Focus\n{topic}\n\n## Advice\n- Shrink the task to one next action.\n- Preserve safety boundaries.\n- Check progress after each step.\n\n## Next Step\nWrite one sentence describing the outcome you want.\n")
}
