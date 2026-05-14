use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppEventKind {
    ChatMessageCreated,
    GoalStarted,
    TaskStarted,
    TaskCompleted,
    ArtifactCreated,
    ValidationCompleted,
    RepairPerformed,
    SnapshotCreated,
    RollbackPerformed,
    DoctorCompleted,
    CreativeProjectCreated,
    ExecutiveDecisionMade,
    SafetyStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEvent {
    pub event_id: String,
    pub session_id: String,
    pub kind: AppEventKind,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppEventLog {
    pub events: Vec<AppEvent>,
}

pub fn record_event(
    store: &DiskStore,
    session_id: &str,
    kind: AppEventKind,
    message: impl Into<String>,
) -> Result<AppEvent> {
    fs::create_dir_all(&store.paths.events)?;
    let path = event_log_path(store, session_id);
    let mut log = if path.exists() {
        load_json::<AppEventLog>(&path)?
    } else {
        AppEventLog::default()
    };
    let event = AppEvent {
        event_id: format!("event_{}", timestamp_slug()),
        session_id: session_id.to_string(),
        kind,
        message: message.into(),
        created_at: Utc::now(),
    };
    log.events.push(event.clone());
    save_json(&path, &log)?;
    Ok(event)
}

pub fn load_events(store: &DiskStore, session_id: &str) -> Result<AppEventLog> {
    let path = event_log_path(store, session_id);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(AppEventLog::default())
    }
}

fn event_log_path(store: &DiskStore, session_id: &str) -> std::path::PathBuf {
    store.paths.events.join(format!("{session_id}.json"))
}
