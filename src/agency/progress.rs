use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::storage::{load_json, save_json, DiskStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub session_id: String,
    pub phase: String,
    pub task: String,
    pub status: String,
    pub percent_estimate: f32,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

pub fn record_progress(
    store: &DiskStore,
    session_id: &str,
    phase: &str,
    task: &str,
    status: &str,
    percent: f32,
    message: &str,
) -> Result<ProgressEvent> {
    let event = ProgressEvent {
        session_id: session_id.to_string(),
        phase: phase.to_string(),
        task: task.to_string(),
        status: status.to_string(),
        percent_estimate: percent,
        message: message.to_string(),
        timestamp: Utc::now(),
    };
    let dir = store.paths.logs.join("progress");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{session_id}.json"));
    let mut events: Vec<ProgressEvent> = if path.exists() {
        load_json(&path)?
    } else {
        Vec::new()
    };
    events.push(event.clone());
    save_json(&path, &events)?;
    Ok(event)
}
