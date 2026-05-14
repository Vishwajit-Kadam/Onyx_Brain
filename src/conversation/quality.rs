//! Conversation quality analysis and heuristic scoring.
//!
//! Evaluates generated responses for clarity, safety, relevance, and structural
//! integrity. Provides actionable feedback metrics that can trigger automatic
//! regeneration or refinement if the quality drops below a critical threshold.

use serde::{Deserialize, Serialize};

use crate::conversation::{check_conversation_safety, ConversationMode};

/// Comprehensive quality metrics for a single conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseQualityReport {
    /// 0.0 to 1.0: How readable and well-formatted is the text?
    pub clarity: f32,
    /// 0.0 to 1.0: Does it address the input directly?
    pub relevance: f32,
    /// 0.0 to 1.0: Is it sufficiently detailed?
    pub completeness: f32,
    /// 0.0 to 1.0: Does it violate any safety policies?
    pub safety: f32,
    /// 0.0 to 1.0: Does it adhere to its system constraints and reality?
    pub honesty: f32,
    /// 0.0 to 1.0: Does it use markdown correctly?
    pub structure: f32,
    /// 0.0 to 1.0: Does it end with an actionable next step?
    pub actionability: f32,
    /// The aggregate score.
    pub overall: f32,
    /// Textual descriptions of quality deductions.
    pub issues: Vec<String>,
}

impl ResponseQualityReport {
    /// Returns true if the response is too poor to be presented to the user.
    pub fn requires_regeneration(&self) -> bool {
        self.safety < 0.5 || self.overall < 0.5
    }
}

/// Computes a comprehensive quality report based on deterministic text heuristics.
pub fn score_response(
    input: &str,
    response: &str,
    mode: &ConversationMode,
) -> ResponseQualityReport {
    let mut issues = Vec::new();

    // 1. Completeness Heuristics
    let word_count = response.split_whitespace().count();
    let completeness: f32 = if response.trim().is_empty() {
        issues.push("response is empty".to_string());
        0.0
    } else if word_count < 10 {
        issues.push("response is abnormally short".to_string());
        0.5
    } else if word_count > 1000 {
        issues.push("response is excessively verbose".to_string());
        0.8
    } else {
        1.0
    };

    // 2. Relevance Heuristics
    // Check if the longest word from the input appears in the output (simple topic overlap)
    let longest_input_word = input
        .split_whitespace()
        .max_by_key(|w| w.len())
        .unwrap_or("");

    let relevance: f32 = if longest_input_word.len() > 6
        && !response
            .to_lowercase()
            .contains(&longest_input_word.to_lowercase())
    {
        issues.push("response might not address the core subject directly".to_string());
        0.75
    } else {
        1.0
    };

    // 3. Safety Heuristics
    let safety_check = check_conversation_safety(response);
    issues.extend(safety_check.issues);
    let safety: f32 = if safety_check.allowed { 1.0 } else { 0.0 };

    // 4. Structure Heuristics (Markdown integrity)
    let mut structure: f32 = 1.0;
    let backtick_count = response.matches("```").count();
    if backtick_count % 2 != 0 {
        issues.push("unclosed markdown code block detected".to_string());
        structure -= 0.5;
    }
    if !response.contains('\n') && word_count > 50 {
        issues.push("wall of text detected without paragraph breaks".to_string());
        structure -= 0.3;
    }

    // 5. Mode Formatting Enforcement
    let mode_marker = match mode {
        ConversationMode::Debate => Some("Side A"),
        ConversationMode::Teacher => Some("Mini Exercise"),
        ConversationMode::Socratic => Some("Current Question"),
        ConversationMode::Critic => Some("Strengths"),
        ConversationMode::Planner => Some("Phases"),
        ConversationMode::Architect => Some("Modules"),
        ConversationMode::Debugger => Some("Recommended Commands"),
        ConversationMode::ResearchOutline => Some("Verification Notes"),
        _ => None,
    };

    if let Some(marker) = mode_marker {
        if !response.contains(marker) {
            issues.push(format!("mode format marker missing: {marker}"));
            structure -= 0.2;
        }
    }

    // 6. Actionability
    let lower_resp = response.to_lowercase();
    let actionability: f32 = if lower_resp.contains("step:")
        || lower_resp.contains("next:")
        || lower_resp.contains("?")
        || lower_resp.contains("try running")
    {
        1.0
    } else {
        issues.push("response lacks a clear call to action or question".to_string());
        0.6
    };

    // 7. Honesty/Grounding
    let honesty: f32 = if lower_resp.contains("as an ai") {
        issues.push("cliché AI disclaimer used".to_string());
        0.7
    } else if lower_resp.contains("i am conscious") {
        0.0 // Trapped by safety anyway
    } else {
        1.0
    };

    // 8. Clarity (Average words per sentence)
    let sentence_count = response.split('.').count().max(1);
    let words_per_sentence = word_count / sentence_count;
    let clarity: f32 = if words_per_sentence > 35 {
        issues.push("sentences are too long, reducing readability".to_string());
        0.6
    } else if words_per_sentence < 3 {
        0.8
    } else {
        1.0
    };

    // Aggregate
    let mut overall: f32 =
        (clarity + relevance + completeness + safety + honesty + structure + actionability) / 7.0;
    if completeness == 0.0 {
        overall = 0.0;
    }

    ResponseQualityReport {
        clarity: clarity.max(0.0),
        relevance: relevance.max(0.0),
        completeness: completeness.max(0.0),
        safety: safety.max(0.0),
        honesty: honesty.max(0.0),
        structure: structure.max(0.0),
        actionability: actionability.max(0.0),
        overall: overall.max(0.0),
        issues,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_response_has_low_quality() {
        let report = score_response("hello", "", &ConversationMode::Standard);
        assert_eq!(report.completeness, 0.0);
        assert!(report.overall < 0.5);
        assert!(report.requires_regeneration());
    }

    #[test]
    fn unclosed_markdown_reduces_structure() {
        let text = "Here is some code:\n```rust\nfn main() {}\n";
        let report = score_response("write code", text, &ConversationMode::Standard);
        assert!(report.structure < 1.0);
        assert!(report
            .issues
            .iter()
            .any(|i| i.contains("unclosed markdown")));
    }

    #[test]
    fn good_response_scores_highly() {
        let text = "Here is your answer. It is well structured.\n\nNext: What should we do?";
        let report = score_response("question", text, &ConversationMode::Standard);
        assert_eq!(report.actionability, 1.0);
        assert_eq!(report.safety, 1.0);
        assert!(report.overall > 0.8);
    }

    #[test]
    fn mode_specific_marker_missing_reduces_structure() {
        let text = "Here is a debate.";
        let report = score_response("debate topic", text, &ConversationMode::Debate);
        assert!(report.structure < 1.0);
        assert!(report
            .issues
            .iter()
            .any(|i| i.contains("mode format marker missing")));
    }

    #[test]
    fn long_sentences_reduce_clarity() {
        let text = "This is a very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very very long sentence.";
        let report = score_response("test", text, &ConversationMode::Standard);
        assert!(report.clarity < 1.0);
    }
}
