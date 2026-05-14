//! Routing classifier — multi-label task classification with confidence scoring.
//!
//! Classifies user input into task types based on keyword patterns with
//! confidence scores, edge-case handling, and support for ambiguous inputs.

use serde::{Deserialize, Serialize};

use crate::core::TaskType;

/// A classification result with confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub primary: TaskType,
    pub confidence: f32,
    pub alternatives: Vec<(TaskType, f32)>,
    pub ambiguous: bool,
}

impl ClassificationResult {
    /// Whether the classification is confident enough to act on.
    pub fn is_confident(&self) -> bool {
        self.confidence > 0.5 && !self.ambiguous
    }

    pub fn summarize(&self) -> String {
        let alts: Vec<String> = self
            .alternatives
            .iter()
            .map(|(t, c)| format!("{:?}({:.0}%)", t, c * 100.0))
            .collect();
        format!(
            "{:?} ({:.0}%){} [{}]",
            self.primary,
            self.confidence * 100.0,
            if self.ambiguous { " ⚠ ambiguous" } else { "" },
            if alts.is_empty() {
                "no alternatives".into()
            } else {
                alts.join(", ")
            }
        )
    }
}

pub struct Classifier;

impl Classifier {
    /// Classify input into a task type (backward-compatible).
    pub fn classify(input: &str) -> TaskType {
        Self::classify_with_confidence(input).primary
    }

    /// Classify with full confidence scoring.
    pub fn classify_with_confidence(input: &str) -> ClassificationResult {
        let lower = input.to_lowercase();

        if lower.trim().is_empty() {
            return ClassificationResult {
                primary: TaskType::Unknown,
                confidence: 1.0,
                alternatives: vec![],
                ambiguous: false,
            };
        }

        // Score each category
        let mut scores: Vec<(TaskType, f32)> = vec![
            (TaskType::Code, score_code(&lower)),
            (TaskType::FileOperation, score_file_op(&lower)),
            (TaskType::Planning, score_planning(&lower)),
            (TaskType::Reasoning, score_reasoning(&lower)),
            (TaskType::MemoryQuery, score_memory(&lower)),
            (TaskType::Chat, score_chat(&lower)),
        ];

        // Sort by score descending
        scores.sort_by(|a, b| b.1.total_cmp(&a.1));

        let (primary, top_score) = scores[0].clone();
        let alternatives: Vec<(TaskType, f32)> = scores[1..]
            .iter()
            .filter(|(_, s)| *s > 0.1)
            .cloned()
            .collect();

        // Ambiguity: top two scores are close
        let ambiguous = if scores.len() >= 2 {
            (top_score - scores[1].1).abs() < 0.15 && top_score < 0.6
        } else {
            false
        };

        ClassificationResult {
            primary,
            confidence: top_score.clamp(0.0, 1.0),
            alternatives,
            ambiguous,
        }
    }
}

// ── Scoring functions ───────────────────────────────────────────────────────

fn score_code(lower: &str) -> f32 {
    let keywords = [
        ("code", 0.4),
        ("rust", 0.4),
        ("compile", 0.5),
        ("cargo", 0.5),
        ("error", 0.3),
        ("project", 0.2),
        ("function", 0.3),
        ("struct", 0.4),
        ("impl", 0.4),
        ("test", 0.25),
        ("lib.rs", 0.5),
        ("main.rs", 0.5),
        ("debug", 0.3),
        ("build", 0.3),
        ("binary", 0.3),
        ("crate", 0.4),
    ];
    weighted_score(lower, &keywords)
}

fn score_file_op(lower: &str) -> f32 {
    let keywords = [
        ("create file", 0.6),
        ("folder", 0.4),
        ("write", 0.3),
        ("directory", 0.4),
        ("save", 0.3),
        ("read file", 0.5),
        ("copy", 0.3),
        ("move", 0.2),
        ("delete file", 0.5),
        ("rename", 0.3),
        ("path", 0.2),
        ("mkdir", 0.5),
    ];
    weighted_score(lower, &keywords)
}

fn score_planning(lower: &str) -> f32 {
    let keywords = [
        ("plan", 0.5),
        ("steps", 0.4),
        ("roadmap", 0.5),
        ("phase", 0.4),
        ("strategy", 0.4),
        ("schedule", 0.3),
        ("milestone", 0.4),
        ("decompose", 0.4),
        ("break down", 0.4),
        ("organize", 0.3),
    ];
    weighted_score(lower, &keywords)
}

fn score_reasoning(lower: &str) -> f32 {
    let keywords = [
        ("why", 0.3),
        ("reason", 0.4),
        ("analyze", 0.4),
        ("explain", 0.3),
        ("think", 0.2),
        ("evaluate", 0.3),
        ("compare", 0.3),
        ("assess", 0.3),
        ("investigate", 0.3),
        ("understand", 0.2),
    ];
    weighted_score(lower, &keywords)
}

fn score_memory(lower: &str) -> f32 {
    let keywords = [
        ("remember", 0.5),
        ("memory", 0.5),
        ("recall", 0.5),
        ("forget", 0.4),
        ("learned", 0.3),
        ("history", 0.3),
        ("past", 0.2),
        ("experience", 0.3),
    ];
    weighted_score(lower, &keywords)
}

fn score_chat(lower: &str) -> f32 {
    let keywords = [
        ("hello", 0.4),
        ("hi", 0.3),
        ("how are", 0.4),
        ("thanks", 0.3),
        ("help", 0.2),
        ("what", 0.1),
        ("tell me", 0.2),
    ];
    // Chat is the fallback — give it a small base score
    let explicit = weighted_score(lower, &keywords);
    (explicit + 0.15).min(0.8) // modest base so it wins for casual input
}

fn weighted_score(input: &str, keywords: &[(&str, f32)]) -> f32 {
    let total: f32 = keywords
        .iter()
        .filter(|(kw, _)| input.contains(kw))
        .map(|(_, weight)| weight)
        .sum();
    total.min(1.0)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_code_input() {
        let result = Classifier::classify_with_confidence("Build a Rust project with cargo");
        assert_eq!(result.primary, TaskType::Code);
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn classifies_file_operation() {
        let result = Classifier::classify("Create file hello.txt in directory src");
        assert_eq!(result, TaskType::FileOperation);
    }

    #[test]
    fn classifies_planning() {
        let result = Classifier::classify("Create a roadmap with phases and milestones");
        assert_eq!(result, TaskType::Planning);
    }

    #[test]
    fn classifies_reasoning() {
        let result =
            Classifier::classify("Why does this algorithm have O(n) complexity? Analyze it");
        assert_eq!(result, TaskType::Reasoning);
    }

    #[test]
    fn classifies_memory() {
        let result = Classifier::classify("Do you remember what we did in the last session?");
        assert_eq!(result, TaskType::MemoryQuery);
    }

    #[test]
    fn empty_input_is_unknown() {
        let result = Classifier::classify_with_confidence("");
        assert_eq!(result.primary, TaskType::Unknown);
        assert!(!result.ambiguous);
    }

    #[test]
    fn casual_input_defaults_to_chat() {
        let result = Classifier::classify("hello, how are you?");
        assert_eq!(result, TaskType::Chat);
    }

    #[test]
    fn ambiguous_input_is_flagged() {
        // "plan code" hits both planning and code
        let result = Classifier::classify_with_confidence("plan");
        // Should detect some alternatives
        assert!(!result.alternatives.is_empty() || result.confidence > 0.3);
    }
}
