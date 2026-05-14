//! Executive continuity — linking sessions, detecting gaps, and rebuilding state.
//!
//! The continuity model tracks the executive layer's awareness of its own session
//! history: how many sessions it has seen, what happened in recent ones, and whether
//! there are unexplained gaps that require recovery.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::storage::{load_json, save_json, DiskStore};

/// Continuity model persisted between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuityModel {
    pub recent_sessions_seen: usize,
    pub continuity_note: String,
    pub session_chain: Vec<SessionLink>,
    pub last_session_status: Option<String>,
    pub gap_detected: bool,
    pub gap_reason: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ContinuityModel {
    fn default() -> Self {
        Self {
            recent_sessions_seen: 0,
            continuity_note: "No sessions observed yet.".into(),
            session_chain: Vec::new(),
            last_session_status: None,
            gap_detected: false,
            gap_reason: None,
            updated_at: Utc::now(),
        }
    }
}

/// A link to a historical session for continuity tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLink {
    pub session_id: String,
    pub title: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl ContinuityModel {
    /// Register that a new session has been observed.
    pub fn register_session(
        &mut self,
        session_id: &str,
        title: &str,
        status: &str,
        started_at: DateTime<Utc>,
        ended_at: Option<DateTime<Utc>>,
    ) {
        // Check for gaps: if last session ended but there's a large time gap
        if let Some(last) = self.session_chain.last() {
            if let Some(last_end) = last.ended_at {
                let gap_hours = started_at.signed_duration_since(last_end).num_hours();
                if gap_hours > 24 {
                    self.gap_detected = true;
                    self.gap_reason = Some(format!(
                        "{}h gap between session '{}' and '{}'",
                        gap_hours, last.title, title
                    ));
                }
            }
        }

        self.session_chain.push(SessionLink {
            session_id: session_id.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            started_at,
            ended_at,
        });
        self.recent_sessions_seen = self.session_chain.len();
        self.last_session_status = Some(status.to_string());
        self.continuity_note = format!(
            "Observed {} sessions. Last: '{}' ({})",
            self.recent_sessions_seen, title, status
        );
        self.updated_at = Utc::now();

        // Keep only the last 50 sessions to bound memory
        if self.session_chain.len() > 50 {
            self.session_chain = self.session_chain.split_off(self.session_chain.len() - 50);
        }
    }

    /// Resolve a detected gap (after recovery or investigation).
    pub fn resolve_gap(&mut self, resolution: &str) {
        self.gap_detected = false;
        self.gap_reason = Some(format!("Resolved: {}", resolution));
        self.updated_at = Utc::now();
    }

    /// Validate that the continuity model is consistent.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        // Session chain should be in chronological order
        for window in self.session_chain.windows(2) {
            if window[1].started_at < window[0].started_at {
                issues.push(format!(
                    "Session '{}' started before '{}' but appears later in chain",
                    window[1].title, window[0].title
                ));
            }
        }
        if self.gap_detected && self.gap_reason.is_none() {
            issues.push("Gap detected but no reason provided".into());
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Produce a concise summary.
    pub fn summarize(&self) -> String {
        let gap_note = if self.gap_detected {
            format!(
                " ⚠ Gap: {}",
                self.gap_reason.as_deref().unwrap_or("unknown")
            )
        } else {
            String::new()
        };
        format!(
            "Continuity: {} sessions tracked. Last status: {}.{}",
            self.recent_sessions_seen,
            self.last_session_status.as_deref().unwrap_or("none"),
            gap_note
        )
    }
}

// ── Persistence ─────────────────────────────────────────────────────────────

const CONTINUITY_FILE: &str = "continuity_model.json";

pub fn save_continuity(store: &DiskStore, model: &ContinuityModel) -> Result<()> {
    let dir = &store.paths.executive;
    fs::create_dir_all(dir)?;
    save_json(&dir.join(CONTINUITY_FILE), model)
}

pub fn load_continuity(store: &DiskStore) -> Result<ContinuityModel> {
    let path = store.paths.executive.join(CONTINUITY_FILE);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(ContinuityModel::default())
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_is_valid() {
        let model = ContinuityModel::default();
        assert!(model.validate().is_ok());
        assert!(!model.gap_detected);
    }

    #[test]
    fn register_session_updates_state() {
        let mut model = ContinuityModel::default();
        model.register_session(
            "s1",
            "First session",
            "Completed",
            Utc::now(),
            Some(Utc::now()),
        );
        assert_eq!(model.recent_sessions_seen, 1);
        assert_eq!(model.last_session_status.as_deref(), Some("Completed"));
    }

    #[test]
    fn gap_detection_works() {
        let mut model = ContinuityModel::default();
        let t1 = Utc::now() - chrono::Duration::hours(48);
        let t1_end = t1 + chrono::Duration::hours(1);
        model.register_session("s1", "Old session", "Completed", t1, Some(t1_end));
        model.register_session("s2", "New session", "Active", Utc::now(), None);
        assert!(model.gap_detected);
    }

    #[test]
    fn resolve_gap_clears_flag() {
        let mut model = ContinuityModel::default();
        model.gap_detected = true;
        model.gap_reason = Some("48h gap".into());
        model.resolve_gap("user was on vacation");
        assert!(!model.gap_detected);
    }

    #[test]
    fn session_chain_is_bounded() {
        let mut model = ContinuityModel::default();
        for i in 0..60 {
            model.register_session(
                &format!("s{i}"),
                &format!("Session {i}"),
                "Completed",
                Utc::now(),
                Some(Utc::now()),
            );
        }
        assert!(model.session_chain.len() <= 50);
    }
}
