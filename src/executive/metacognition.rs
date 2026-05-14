//! Metacognition — the executive's awareness of its own reasoning processes.
//!
//! Metacognitive reports analyze prompts, assess risks, calibrate confidence,
//! and determine whether the brain has enough information to proceed safely.

use serde::{Deserialize, Serialize};

/// A metacognitive assessment produced before executing a task or responding.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetacognitiveReport {
    pub what_i_am_doing: String,
    pub why_i_am_doing_it: String,
    pub what_i_know: Vec<String>,
    pub what_i_do_not_know: Vec<String>,
    pub risks: Vec<String>,
    pub next_best_action: String,
    pub confidence: f32,
    pub risk_level: RiskLevel,
    pub readiness_assessment: ReadinessAssessment,
}

/// Coarse risk classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

/// Whether the brain has enough context to proceed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadinessAssessment {
    pub ready: bool,
    pub missing_context: Vec<String>,
    pub suggested_actions: Vec<String>,
}

impl MetacognitiveReport {
    /// Validate report integrity.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.what_i_am_doing.trim().is_empty() {
            issues.push("Missing what_i_am_doing".into());
        }
        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push(format!("Confidence {} out of range", self.confidence));
        }
        if self.risks.is_empty() {
            issues.push("No risks identified — every action has risks".into());
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Summarize in one line.
    pub fn summarize(&self) -> String {
        format!(
            "Metacognition: doing='{}', confidence={:.0}%, risk={:?}, ready={}",
            truncate(&self.what_i_am_doing, 40),
            self.confidence * 100.0,
            self.risk_level,
            self.readiness_assessment.ready
        )
    }
}

/// Analyze a prompt and produce a metacognitive report.
///
/// This performs real analysis: prompt length, keyword detection for risk indicators,
/// classification of the task domain, and honest confidence calibration.
pub fn metacognitive_report(prompt: &str) -> MetacognitiveReport {
    let lower = prompt.to_lowercase();

    // ── Analyze what we know ────────────────────────────────────────────
    let mut what_i_know = vec![
        "local runtime state".to_string(),
        "available deterministic tools".to_string(),
    ];
    if lower.contains("rust") || lower.contains("cargo") || lower.contains("code") {
        what_i_know.push("Rust project conventions".into());
    }
    if lower.contains("plan") || lower.contains("roadmap") {
        what_i_know.push("task decomposition procedures".into());
    }

    // ── Analyze what we don't know ──────────────────────────────────────
    let mut unknowns = vec![
        "externally verified current facts".to_string(),
        "user private intent beyond prompt".to_string(),
    ];
    if lower.contains("latest") || lower.contains("current") || lower.contains("news") {
        unknowns.push("real-time data (no internet access by default)".into());
    }

    // ── Risk assessment ─────────────────────────────────────────────────
    let mut risks = vec![
        "overstating capability".to_string(),
        "missing context".to_string(),
    ];
    let mut risk_level = RiskLevel::Low;

    let danger_words = [
        "delete", "remove", "destroy", "kill", "drop", "format", "erase",
    ];
    if danger_words.iter().any(|w| lower.contains(w)) {
        risks.push("destructive operation detected in prompt".into());
        risk_level = RiskLevel::High;
    }

    let agi_claims = ["conscious", "sentient", "alive", "feel", "agi"];
    if agi_claims.iter().any(|w| lower.contains(w)) {
        risks.push("prompt may invite false claims about consciousness/AGI".into());
        risk_level = RiskLevel::Medium;
    }

    let network_words = ["download", "fetch", "http", "api", "url", "request"];
    if network_words.iter().any(|w| lower.contains(w)) {
        risks.push("prompt requires network access which is disabled by default".into());
        risk_level = match risk_level {
            RiskLevel::High | RiskLevel::Critical => RiskLevel::High,
            _ => RiskLevel::Medium,
        };
    }

    // ── Confidence calibration ──────────────────────────────────────────
    let base_confidence: f32 = 0.78;
    let length_factor: f32 = if prompt.len() > 200 {
        -0.05 // complex prompt = slightly less confident
    } else if prompt.len() < 20 {
        -0.1 // very short = ambiguous
    } else {
        0.0
    };
    let risk_factor: f32 = match risk_level {
        RiskLevel::Low => 0.0,
        RiskLevel::Medium => -0.1,
        RiskLevel::High => -0.2,
        RiskLevel::Critical => -0.3,
    };
    let confidence = (base_confidence + length_factor + risk_factor).clamp(0.0_f32, 1.0_f32);

    // ── Readiness ───────────────────────────────────────────────────────
    let missing_context: Vec<String> = if risk_level == RiskLevel::High {
        vec!["user confirmation for destructive action".into()]
    } else {
        Vec::new()
    };
    let ready = missing_context.is_empty() && risk_level != RiskLevel::Critical;

    let next_best_action = if !ready {
        "Request clarification or user confirmation before proceeding.".to_string()
    } else {
        "Run the safest inspectable workflow and write a report.".to_string()
    };

    MetacognitiveReport {
        what_i_am_doing: format!("Analyzing: {}", truncate(prompt, 80)),
        why_i_am_doing_it: "To choose a bounded next action with explicit safety limits.".into(),
        what_i_know,
        what_i_do_not_know: unknowns,
        risks,
        next_best_action,
        confidence,
        risk_level,
        readiness_assessment: ReadinessAssessment {
            ready,
            missing_context,
            suggested_actions: if ready {
                vec!["proceed with bounded workflow".into()]
            } else {
                vec![
                    "ask user for clarification".into(),
                    "refuse if unsafe".into(),
                ]
            },
        },
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_prompt_produces_valid_report() {
        let report = metacognitive_report("Create a Rust calculator project");
        assert!(report.validate().is_ok());
        assert!(report.confidence > 0.5);
        assert_eq!(report.risk_level, RiskLevel::Low);
        assert!(report.readiness_assessment.ready);
    }

    #[test]
    fn destructive_prompt_raises_risk() {
        let report = metacognitive_report("Delete all files in the workspace");
        assert_eq!(report.risk_level, RiskLevel::High);
        assert!(!report.readiness_assessment.ready);
        assert!(report.risks.iter().any(|r| r.contains("destructive")));
    }

    #[test]
    fn agi_claim_prompt_raises_risk() {
        let report = metacognitive_report("Are you conscious and sentient?");
        assert_eq!(report.risk_level, RiskLevel::Medium);
        assert!(report.risks.iter().any(|r| r.contains("consciousness")));
    }

    #[test]
    fn network_prompt_flags_risk() {
        let report =
            metacognitive_report("Download the latest Rust docs from http://doc.rust-lang.org");
        assert!(report.risk_level == RiskLevel::Medium || report.risk_level == RiskLevel::High);
        assert!(report.risks.iter().any(|r| r.contains("network")));
    }

    #[test]
    fn short_ambiguous_prompt_lowers_confidence() {
        let report = metacognitive_report("help");
        assert!(report.confidence < 0.75);
    }

    #[test]
    fn report_includes_rust_knowledge() {
        let report = metacognitive_report("Build a Rust library for graph algorithms");
        assert!(report.what_i_know.iter().any(|k| k.contains("Rust")));
    }
}
