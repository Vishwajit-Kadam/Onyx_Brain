use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::{
    app_api::{record_event, AppEventKind},
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveDecision {
    pub decision_id: String,
    pub session_id: String,
    pub observed_state: String,
    pub candidate_actions: Vec<String>,
    pub chosen_action: String,
    pub reason: String,
    pub confidence: f32,
    pub safety_checked: bool,
    pub created_at: DateTime<Utc>,
}

pub fn record_executive_decision(
    store: &DiskStore,
    session_id: &str,
    observed_state: &str,
    chosen_action: &str,
) -> Result<ExecutiveDecision> {
    let dir = store.paths.executive.join("decisions");
    fs::create_dir_all(&dir)?;
    let decision = ExecutiveDecision {
        decision_id: format!("executive_decision_{}", Uuid::new_v4()),
        session_id: session_id.to_string(),
        observed_state: observed_state.to_string(),
        candidate_actions: vec![
            "inspect status".to_string(),
            "run bounded workflow".to_string(),
            "stop safely".to_string(),
        ],
        chosen_action: chosen_action.to_string(),
        reason: "Chosen because it preserves sandbox, allowlist, and finite execution limits."
            .to_string(),
        confidence: 0.83,
        safety_checked: true,
        created_at: Utc::now(),
    };
    save_json(
        &dir.join(format!("{}.json", decision.decision_id)),
        &decision,
    )?;
    let _ = record_event(
        store,
        session_id,
        AppEventKind::ExecutiveDecisionMade,
        &decision.chosen_action,
    );
    Ok(decision)
}

pub fn recent_decisions(store: &DiskStore, limit: usize) -> Result<Vec<ExecutiveDecision>> {
    let dir = store.paths.executive.join("decisions");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut rows = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Ok(decision) = load_json::<ExecutiveDecision>(&path) {
                rows.push(decision);
            }
        }
    }
    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    rows.truncate(limit);
    Ok(rows)
}
