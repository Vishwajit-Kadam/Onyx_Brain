//! Episodic memory — stores sequences of events tied to a specific session or experience.
//!
//! Unlike semantic memories (facts) or procedural memories (how-to steps), episodic
//! memories capture *what happened* in temporal order. They decay over time unless
//! reinforced by access or marked as significant.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::storage::{load_json, save_json, DiskStore};

pub use crate::memory::MemoryItem;

/// A single event within an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub tags: Vec<String>,
    /// Emotional valence: -1.0 (negative) to 1.0 (positive). Neutral is 0.0.
    pub valence: f32,
}

/// A full episode — a coherent sequence of events from a single session or experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub episode_id: String,
    pub session_id: String,
    pub title: String,
    pub events: Vec<EpisodicEvent>,
    pub significance: f32,
    pub access_count: u64,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Episode {
    /// Create a new episode for a session.
    pub fn new(session_id: &str, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            episode_id: format!("episode_{}", Uuid::new_v4()),
            session_id: session_id.to_string(),
            title: title.into(),
            events: Vec::new(),
            significance: 0.5,
            access_count: 0,
            created_at: now,
            last_accessed: now,
        }
    }

    /// Record a new event in this episode.
    pub fn record_event(
        &mut self,
        description: impl Into<String>,
        tags: Vec<String>,
        valence: f32,
    ) {
        self.events.push(EpisodicEvent {
            event_id: format!("evt_{}", Uuid::new_v4()),
            timestamp: Utc::now(),
            description: description.into(),
            tags,
            valence: valence.clamp(-1.0, 1.0),
        });
    }

    /// Compute a decay-adjusted significance score.
    /// Episodes lose significance over time unless accessed frequently.
    pub fn decayed_significance(&self) -> f32 {
        let age_days = Utc::now()
            .signed_duration_since(self.last_accessed)
            .num_days()
            .max(0) as f32;
        let decay = (-age_days / 30.0).exp(); // half-life ~21 days
        let access_boost = (self.access_count as f32 / 5.0).min(1.0) * 0.2;
        (self.significance * decay + access_boost).clamp(0.0, 1.0)
    }

    /// Mark this episode as accessed, bumping its recency and count.
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Validate that the episode has minimal required structure.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        if self.title.trim().is_empty() {
            issues.push("Episode title is empty".to_string());
        }
        if self.session_id.trim().is_empty() {
            issues.push("Episode session_id is empty".to_string());
        }
        if self.significance < 0.0 || self.significance > 1.0 {
            issues.push(format!(
                "Significance {} is out of [0.0, 1.0] range",
                self.significance
            ));
        }
        for evt in &self.events {
            if evt.description.trim().is_empty() {
                issues.push(format!("Event {} has empty description", evt.event_id));
            }
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    /// Produce a concise text summary of the episode.
    pub fn summarize(&self) -> String {
        let event_count = self.events.len();
        let avg_valence = if event_count > 0 {
            self.events.iter().map(|e| e.valence).sum::<f32>() / event_count as f32
        } else {
            0.0
        };
        let valence_label = if avg_valence > 0.3 {
            "positive"
        } else if avg_valence < -0.3 {
            "negative"
        } else {
            "neutral"
        };
        format!(
            "Episode '{}': {} events, significance {:.2}, overall {} tone, accessed {} times",
            self.title,
            event_count,
            self.decayed_significance(),
            valence_label,
            self.access_count
        )
    }
}

// ── Persistence helpers ─────────────────────────────────────────────────────

/// Save an episode to the episodic memory directory.
pub fn save_episode(store: &DiskStore, episode: &Episode) -> Result<()> {
    let dir = store.paths.data.join("episodic");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join(format!("{}.json", episode.episode_id)), episode)
}

/// Load a specific episode by ID.
pub fn load_episode(store: &DiskStore, episode_id: &str) -> Result<Episode> {
    let path = store
        .paths
        .data
        .join("episodic")
        .join(format!("{episode_id}.json"));
    load_json(&path)
}

/// Load all episodes, optionally filtering by session_id.
pub fn load_episodes(store: &DiskStore, session_filter: Option<&str>) -> Result<Vec<Episode>> {
    let dir = store.paths.data.join("episodic");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut episodes = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(ep) = load_json::<Episode>(&path) {
                if let Some(sid) = session_filter {
                    if ep.session_id != sid {
                        continue;
                    }
                }
                episodes.push(ep);
            }
        }
    }
    episodes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(episodes)
}

/// Return the most significant recent episodes, sorted by decayed significance.
pub fn most_significant_episodes(store: &DiskStore, limit: usize) -> Result<Vec<Episode>> {
    let mut episodes = load_episodes(store, None)?;
    episodes.sort_by(|a, b| {
        b.decayed_significance()
            .total_cmp(&a.decayed_significance())
    });
    episodes.truncate(limit);
    Ok(episodes)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_episode_has_valid_defaults() {
        let ep = Episode::new("session_1", "Test episode");
        assert_eq!(ep.session_id, "session_1");
        assert_eq!(ep.events.len(), 0);
        assert!((ep.significance - 0.5).abs() < f32::EPSILON);
        assert!(ep.validate().is_ok());
    }

    #[test]
    fn record_event_adds_to_sequence() {
        let mut ep = Episode::new("s1", "Events test");
        ep.record_event("goal started", vec!["start".into()], 0.5);
        ep.record_event("task failed", vec!["fail".into()], -0.8);
        ep.record_event("recovery succeeded", vec!["recovery".into()], 0.7);
        assert_eq!(ep.events.len(), 3);
        assert!(ep.events[1].valence < 0.0);
    }

    #[test]
    fn valence_is_clamped() {
        let mut ep = Episode::new("s1", "Clamp test");
        ep.record_event("extreme", vec![], 5.0);
        assert!((ep.events[0].valence - 1.0).abs() < f32::EPSILON);
        ep.record_event("extreme negative", vec![], -10.0);
        assert!((ep.events[1].valence - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn validate_rejects_empty_title() {
        let ep = Episode::new("s1", "");
        assert!(ep.validate().is_err());
    }

    #[test]
    fn decayed_significance_decreases_over_time() {
        let mut ep = Episode::new("s1", "Decay test");
        ep.significance = 0.9;
        // Freshly created — should be close to base significance
        let fresh = ep.decayed_significance();
        assert!(fresh > 0.8);
    }

    #[test]
    fn mark_accessed_bumps_count() {
        let mut ep = Episode::new("s1", "Access test");
        assert_eq!(ep.access_count, 0);
        ep.mark_accessed();
        ep.mark_accessed();
        assert_eq!(ep.access_count, 2);
    }

    #[test]
    fn summarize_produces_readable_output() {
        let mut ep = Episode::new("s1", "Summary test");
        ep.record_event("started", vec![], 0.5);
        ep.record_event("completed", vec![], 0.8);
        let summary = ep.summarize();
        assert!(summary.contains("Summary test"));
        assert!(summary.contains("2 events"));
        assert!(summary.contains("positive"));
    }
}
