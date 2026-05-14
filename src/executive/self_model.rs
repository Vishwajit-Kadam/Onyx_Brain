//! Executive self-model — the brain's representation of its own identity,
//! capabilities, limitations, and safety state.
//!
//! The self-model can be persisted, validated for drift, updated with new
//! capabilities, and loaded on startup to restore executive awareness.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::storage::{load_json, save_json, DiskStore};

/// The brain's model of itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub limitations: Vec<String>,
    pub current_goals: Vec<String>,
    pub current_mode: String,
    pub confidence_state: ConfidenceState,
    pub safety_state: SafetyState,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceState {
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyState {
    pub sandboxed: bool,
    pub network_default: String,
    pub unrestricted_shell: bool,
    pub note: String,
}

impl SelfModel {
    /// Validate the self-model for integrity.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.name.trim().is_empty() {
            issues.push("Self-model name is empty".into());
        }
        if self.version.trim().is_empty() {
            issues.push("Version is empty".into());
        }
        if self.capabilities.is_empty() {
            issues.push("No capabilities listed".into());
        }
        if self.limitations.is_empty() {
            issues.push("No limitations listed — every system has limitations".into());
        }
        if self.confidence_state.score < 0.0 || self.confidence_state.score > 1.0 {
            issues.push(format!(
                "Confidence {} out of [0,1] range",
                self.confidence_state.score
            ));
        }
        // Safety invariant: the brain must always be sandboxed
        if !self.safety_state.sandboxed {
            issues.push("CRITICAL: self-model reports sandbox disabled".into());
        }
        if self.safety_state.unrestricted_shell {
            issues.push("CRITICAL: self-model reports unrestricted shell".into());
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Check if the self-model has drifted from a known-good baseline.
    pub fn drift_report(&self, baseline: &SelfModel) -> Vec<String> {
        let mut drifts = Vec::new();
        if self.version != baseline.version {
            drifts.push(format!(
                "Version changed: {} → {}",
                baseline.version, self.version
            ));
        }
        let new_caps: Vec<_> = self
            .capabilities
            .iter()
            .filter(|c| !baseline.capabilities.contains(c))
            .collect();
        if !new_caps.is_empty() {
            drifts.push(format!("New capabilities: {:?}", new_caps));
        }
        let removed_caps: Vec<_> = baseline
            .capabilities
            .iter()
            .filter(|c| !self.capabilities.contains(c))
            .collect();
        if !removed_caps.is_empty() {
            drifts.push(format!("Removed capabilities: {:?}", removed_caps));
        }
        if self.safety_state.sandboxed != baseline.safety_state.sandboxed {
            drifts.push("CRITICAL: sandbox state changed".into());
        }
        if self.safety_state.unrestricted_shell != baseline.safety_state.unrestricted_shell {
            drifts.push("CRITICAL: unrestricted_shell state changed".into());
        }
        drifts
    }

    /// Add a new capability (if not already present).
    pub fn add_capability(&mut self, cap: impl Into<String>) {
        let c = cap.into();
        if !self.capabilities.contains(&c) {
            self.capabilities.push(c);
            self.last_updated = Utc::now();
        }
    }

    /// Update confidence with a reason.
    pub fn update_confidence(&mut self, score: f32, reason: impl Into<String>) {
        self.confidence_state.score = score.clamp(0.0, 1.0);
        self.confidence_state.reason = reason.into();
        self.last_updated = Utc::now();
    }

    /// Produce a concise summary.
    pub fn summarize(&self) -> String {
        format!(
            "SelfModel '{}' v{}: {} capabilities, {} limitations, confidence {:.0}%, sandbox={}",
            self.name,
            self.version,
            self.capabilities.len(),
            self.limitations.len(),
            self.confidence_state.score * 100.0,
            self.safety_state.sandboxed
        )
    }
}

// ── Persistence ─────────────────────────────────────────────────────────────

const SELF_MODEL_FILE: &str = "self_model.json";

pub fn save_self_model(store: &DiskStore, model: &SelfModel) -> Result<()> {
    let dir = &store.paths.executive;
    fs::create_dir_all(dir)?;
    save_json(&dir.join(SELF_MODEL_FILE), model)
}

pub fn load_self_model(store: &DiskStore) -> Result<Option<SelfModel>> {
    let path = store.paths.executive.join(SELF_MODEL_FILE);
    if path.exists() {
        Ok(Some(load_json(&path)?))
    } else {
        Ok(None)
    }
}

/// Initialize the default self-model for a given version.
pub fn initialize_self_model(version: &str) -> SelfModel {
    SelfModel {
        id: format!("self_model_{}", Uuid::new_v4()),
        name: "Onyx Brain consciousness-inspired self-model".to_string(),
        version: version.to_string(),
        capabilities: vec![
            "bounded autonomous workflow orchestration".to_string(),
            "deterministic conversation modes".to_string(),
            "creative production planning".to_string(),
            "recoverability through journal, snapshots, and doctor checks".to_string(),
        ],
        limitations: vec![
            "not conscious, sentient, AGI, or a real LLM by default".to_string(),
            "no network access by default".to_string(),
            "no unrestricted shell execution".to_string(),
        ],
        current_goals: Vec::new(),
        current_mode: "reflective state".to_string(),
        confidence_state: ConfidenceState {
            score: 0.82,
            reason: "core systems initialized and safety boundaries known".to_string(),
        },
        safety_state: SafetyState {
            sandboxed: true,
            network_default: "disabled by default".to_string(),
            unrestricted_shell: false,
            note: "Safety boundaries cannot be disabled from the GUI.".to_string(),
        },
        last_updated: Utc::now(),
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_self_model_validates() {
        let model = initialize_self_model("0.0.4");
        assert!(model.validate().is_ok());
    }

    #[test]
    fn validate_catches_disabled_sandbox() {
        let mut model = initialize_self_model("0.0.4");
        model.safety_state.sandboxed = false;
        let result = model.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|i| i.contains("sandbox")));
    }

    #[test]
    fn validate_catches_unrestricted_shell() {
        let mut model = initialize_self_model("0.0.4");
        model.safety_state.unrestricted_shell = true;
        let result = model.validate();
        assert!(result.is_err());
    }

    #[test]
    fn drift_detection_finds_version_change() {
        let baseline = initialize_self_model("0.0.3");
        let current = initialize_self_model("0.0.4");
        let drifts = current.drift_report(&baseline);
        assert!(drifts.iter().any(|d| d.contains("Version")));
    }

    #[test]
    fn drift_detection_catches_safety_change() {
        let baseline = initialize_self_model("0.0.4");
        let mut current = initialize_self_model("0.0.4");
        current.safety_state.sandboxed = false;
        let drifts = current.drift_report(&baseline);
        assert!(drifts.iter().any(|d| d.contains("CRITICAL")));
    }

    #[test]
    fn add_capability_is_idempotent() {
        let mut model = initialize_self_model("0.0.4");
        let initial_count = model.capabilities.len();
        model.add_capability("new feature");
        model.add_capability("new feature"); // duplicate
        assert_eq!(model.capabilities.len(), initial_count + 1);
    }

    #[test]
    fn update_confidence_clamps() {
        let mut model = initialize_self_model("0.0.4");
        model.update_confidence(5.0, "impossible confidence");
        assert!((model.confidence_state.score - 1.0).abs() < f32::EPSILON);
    }
}
