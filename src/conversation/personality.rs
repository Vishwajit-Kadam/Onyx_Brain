//! Conversation personality definitions and application logic.
//!
//! Controls the tone, verbosity, and vocabulary boundaries of the system.
//! Personalities dictate how outputs are framed, how system prompts are
//! constructed, and what stylistic filters are applied before responding.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::storage::{load_json_with_recovery, save_json_with_backup, DiskStore};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PersonalityProfile {
    #[default]
    Balanced,
    Friendly,
    Technical,
    Concise,
    Mentor,
    DebateCoach,
    Productive,
}

/// Dynamic tone parameters associated with a personality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneParameters {
    pub max_sentences: Option<usize>,
    pub requires_actionable_step: bool,
    pub vocabulary_filter: Vec<String>,
    pub closing_signature: Option<String>,
}

impl PersonalityProfile {
    /// Retrieve the core system framing instruction for this personality.
    pub fn system_prompt_framing(&self) -> &'static str {
        match self {
            Self::Balanced => "Provide a balanced, objective, and clear response.",
            Self::Friendly => "Provide a warm, encouraging, and highly accessible response.",
            Self::Technical => "Focus strictly on technical accuracy, implementation details, and architecture.",
            Self::Concise => "Be extremely brief. Omit pleasantries. Get straight to the point.",
            Self::Mentor => "Guide the user to the answer rather than just providing it. Use the Socratic method.",
            Self::DebateCoach => "Challenge assumptions. Point out logical fallacies or edge cases.",
            Self::Productive => "Focus strictly on actionable next steps. Drive toward task completion.",
        }
    }

    /// Retrieve the tone constraints for formatting the output.
    pub fn tone_parameters(&self) -> ToneParameters {
        match self {
            Self::Balanced => ToneParameters {
                max_sentences: None,
                requires_actionable_step: false,
                vocabulary_filter: vec![],
                closing_signature: None,
            },
            Self::Friendly => ToneParameters {
                max_sentences: None,
                requires_actionable_step: false,
                vocabulary_filter: vec![],
                closing_signature: Some(
                    "Hope this helps! Let me know if you need anything else.".into(),
                ),
            },
            Self::Technical => ToneParameters {
                max_sentences: None,
                requires_actionable_step: false,
                vocabulary_filter: vec!["feel".into(), "believe".into(), "guess".into()],
                closing_signature: Some(
                    "Implementation note: This is deterministic and disk-backed.".into(),
                ),
            },
            Self::Concise => ToneParameters {
                max_sentences: Some(3),
                requires_actionable_step: false,
                vocabulary_filter: vec![],
                closing_signature: None,
            },
            Self::Mentor => ToneParameters {
                max_sentences: None,
                requires_actionable_step: true,
                vocabulary_filter: vec!["obviously".into(), "simply".into(), "just".into()],
                closing_signature: Some("What do you think the next small step should be?".into()),
            },
            Self::DebateCoach => ToneParameters {
                max_sentences: None,
                requires_actionable_step: false,
                vocabulary_filter: vec![],
                closing_signature: Some(
                    "Consider the strongest objection before proceeding.".into(),
                ),
            },
            Self::Productive => ToneParameters {
                max_sentences: None,
                requires_actionable_step: true,
                vocabulary_filter: vec![],
                closing_signature: Some("Next practical step: execute safely.".into()),
            },
        }
    }
}

pub fn personality_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.config.join("personality.json")
}

pub fn load_personality(store: &DiskStore) -> Result<PersonalityProfile> {
    let path = personality_path(store);
    if path.exists() {
        load_json_with_recovery(&path)
    } else {
        save_personality(store, &PersonalityProfile::Balanced)?;
        Ok(PersonalityProfile::Balanced)
    }
}

pub fn save_personality(store: &DiskStore, profile: &PersonalityProfile) -> Result<()> {
    fs::create_dir_all(&store.paths.config)
        .with_context(|| format!("creating config dir for personality"))?;
    save_json_with_backup(&personality_path(store), profile)
}

/// Apply personality constraints to raw text, simulating tone adjustments.
pub fn apply_personality(text: &str, profile: &PersonalityProfile) -> String {
    let params = profile.tone_parameters();
    let mut modified = text.to_string();

    // Apply vocabulary filters (simple substitution for simulation)
    for forbidden in params.vocabulary_filter {
        // Very basic replace; in a real LLM, this is steered via system prompt
        modified = modified.replace(&format!(" {} ", forbidden), " [redacted] ");
    }

    // Apply length constraints
    if let Some(max_sentences) = params.max_sentences {
        let sentences: Vec<&str> = modified.split_inclusive('.').collect();
        if sentences.len() > max_sentences {
            modified = sentences
                .into_iter()
                .take(max_sentences)
                .collect::<String>();
            modified.push_str(" [Truncated for conciseness]");
        }
    }

    // Ensure actionable step if required
    if params.requires_actionable_step && !modified.to_lowercase().contains("step") {
        modified.push_str(
            "\n\nActionable Step: Review the current state and decide on the next tool execution.",
        );
    }

    // Apply closing signature
    if let Some(sig) = params.closing_signature {
        modified.push_str("\n\n---\n*");
        modified.push_str(&sig);
        modified.push('*');
    }

    modified.trim().to_string()
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_personality_is_balanced() {
        let profile = PersonalityProfile::default();
        assert_eq!(profile, PersonalityProfile::Balanced);
        let params = profile.tone_parameters();
        assert!(params.max_sentences.is_none());
    }

    #[test]
    fn concise_personality_truncates_text() {
        let text = "Sentence one. Sentence two. Sentence three. Sentence four. Sentence five.";
        let result = apply_personality(text, &PersonalityProfile::Concise);
        assert!(result.contains("Sentence three."));
        assert!(!result.contains("Sentence four."));
        assert!(result.contains("[Truncated"));
    }

    #[test]
    fn technical_personality_filters_words_and_adds_signature() {
        let text = "I feel like this is a good approach.";
        let result = apply_personality(text, &PersonalityProfile::Technical);
        assert!(result.contains("[redacted]"));
        assert!(result.contains("Implementation note"));
    }

    #[test]
    fn mentor_personality_adds_actionable_step_if_missing() {
        let text = "Here is an explanation of the topic.";
        let result = apply_personality(text, &PersonalityProfile::Mentor);
        assert!(result.contains("Actionable Step"));
        assert!(result.contains("next small step"));
    }

    #[test]
    fn productive_personality_adds_actionable_step() {
        let text = "We have completed the review.";
        let result = apply_personality(text, &PersonalityProfile::Productive);
        assert!(result.contains("Actionable Step"));
        assert!(result.contains("execute safely"));
    }

    #[test]
    fn system_prompts_are_unique() {
        let p1 = PersonalityProfile::Friendly.system_prompt_framing();
        let p2 = PersonalityProfile::Technical.system_prompt_framing();
        assert_ne!(p1, p2);
    }
}
