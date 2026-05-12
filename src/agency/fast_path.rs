use serde::{Deserialize, Serialize};

use crate::{
    agency::{IntentKind, ParsedGoal, PlanCacheMatch},
    learning::HabitMatch,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FastPathDecision {
    pub used_fast_path: bool,
    pub reason: String,
    pub confidence: f32,
    pub skipped_steps: Vec<String>,
    pub preserved_steps: Vec<String>,
}

pub fn decide_fast_path(
    parsed: &ParsedGoal,
    habits: &[HabitMatch],
    cache: Option<&PlanCacheMatch>,
    previous_failure: bool,
) -> FastPathDecision {
    let habit_confidence = habits
        .iter()
        .map(|habit| habit.confidence * habit.relevance_score)
        .fold(0.0_f32, f32::max);
    let cache_confidence = cache.map(|cache| cache.similarity_score).unwrap_or(0.0);
    let confidence = habit_confidence.max(cache_confidence);
    let supported = matches!(
        parsed.intent,
        IntentKind::CreateProject | IntentKind::ModifyProject
    ) && parsed.requested_features.iter().all(|feature| {
        let feature = feature.to_ascii_lowercase();
        matches!(
            feature.as_str(),
            "add"
                | "subtract"
                | "multiply"
                | "divide"
                | "tests"
                | "readme"
                | "cli"
                | "function"
                | "module"
        )
    });

    if confidence >= 0.85 && supported && !previous_failure {
        FastPathDecision {
            used_fast_path: true,
            reason: "high-confidence habit or cached plan matched a supported safe workflow"
                .to_string(),
            confidence,
            skipped_steps: vec![
                "verbose planning narration".to_string(),
                "redundant procedural memory fan-out".to_string(),
                "repeated project structure analysis".to_string(),
            ],
            preserved_steps: vec![
                "sandbox path checks".to_string(),
                "project state updates".to_string(),
                "cargo validation when policy requires it".to_string(),
                "route trace".to_string(),
                "final report".to_string(),
            ],
        }
    } else {
        FastPathDecision {
            used_fast_path: false,
            reason: if previous_failure {
                "previous similar failure prevents fast path".to_string()
            } else if !supported {
                "task is not a supported deterministic fast-path workflow".to_string()
            } else {
                "no high-confidence habit or cache match".to_string()
            },
            confidence,
            skipped_steps: Vec::new(),
            preserved_steps: vec![
                "full deterministic project workflow".to_string(),
                "all safety checks".to_string(),
            ],
        }
    }
}
