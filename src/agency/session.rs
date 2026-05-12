use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
    Interrupted,
    Resumable,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    pub session_id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub goal_ids: Vec<String>,
    pub project_ids: Vec<String>,
    pub journal_entries: Vec<String>,
    pub checkpoints: Vec<String>,
    pub total_runtime_ms: u64,
    pub total_energy: f32,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionSummary {
    pub session_id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionIndex {
    #[serde(default)]
    pub sessions: Vec<WorkSessionSummary>,
}

pub fn session_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("session_index.json")
}

pub fn session_path(store: &DiskStore, session_id: &str) -> std::path::PathBuf {
    store.paths.sessions.join(format!("{session_id}.json"))
}

pub fn load_session_index(store: &DiskStore) -> Result<SessionIndex> {
    let path = session_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(SessionIndex::default())
    }
}

pub fn save_session(store: &DiskStore, session: &WorkSession) -> Result<()> {
    save_json(&session_path(store, &session.session_id), session)?;
    let mut index = load_session_index(store)?;
    index
        .sessions
        .retain(|row| row.session_id != session.session_id);
    index.sessions.push(WorkSessionSummary {
        session_id: session.session_id.clone(),
        title: session.title.clone(),
        started_at: session.started_at,
        ended_at: session.ended_at,
        status: session.status.clone(),
    });
    index
        .sessions
        .sort_by(|a, b| b.started_at.cmp(&a.started_at));
    if index.sessions.len() > 256 {
        index.sessions.truncate(256);
    }
    save_json(&session_index_path(store), &index)
}

pub fn load_session(store: &DiskStore, selector: &str) -> Result<WorkSession> {
    let session_id = resolve_session_id(store, selector)?;
    load_json(&session_path(store, &session_id))
}

pub fn resolve_session_id(store: &DiskStore, selector: &str) -> Result<String> {
    if selector.eq_ignore_ascii_case("latest") {
        load_session_index(store)?
            .sessions
            .first()
            .map(|row| row.session_id.clone())
            .ok_or_else(|| anyhow::anyhow!("no sessions found"))
    } else {
        Ok(selector.to_string())
    }
}

pub fn session_start(store: &DiskStore, title: impl Into<String>) -> Result<WorkSession> {
    let now = Utc::now();
    let session = WorkSession {
        session_id: format!("session_{}_{}", timestamp_slug(), Uuid::new_v4()),
        title: title.into(),
        started_at: now,
        ended_at: None,
        status: SessionStatus::Active,
        goal_ids: Vec::new(),
        project_ids: Vec::new(),
        journal_entries: Vec::new(),
        checkpoints: Vec::new(),
        total_runtime_ms: 0,
        total_energy: 0.0,
        summary: String::new(),
    };
    save_session(store, &session)?;
    Ok(session)
}

pub fn get_or_start_session(store: &DiskStore, title: &str) -> Result<WorkSession> {
    if let Some(active) = load_session_index(store)?
        .sessions
        .into_iter()
        .find(|row| row.status == SessionStatus::Active)
    {
        load_json(&session_path(store, &active.session_id))
    } else {
        session_start(store, title)
    }
}

pub fn session_end(store: &DiskStore, selector: &str) -> Result<WorkSession> {
    let mut session = load_session(store, selector)?;
    session.status = SessionStatus::Completed;
    session.ended_at = Some(Utc::now());
    if session.summary.is_empty() {
        session.summary = "Session completed.".to_string();
    }
    save_session(store, &session)?;
    Ok(session)
}

pub fn session_resume(store: &DiskStore, selector: &str) -> Result<WorkSession> {
    let mut session = load_session(store, selector)?;
    session.status = SessionStatus::Active;
    session
        .checkpoints
        .push("Session resumed without duplicating completed tasks.".to_string());
    save_session(store, &session)?;
    Ok(session)
}

pub fn sessions(store: &DiskStore) -> Result<Vec<WorkSessionSummary>> {
    Ok(load_session_index(store)?.sessions)
}

pub fn active_session_count(store: &DiskStore) -> Result<usize> {
    Ok(load_session_index(store)?
        .sessions
        .iter()
        .filter(|row| row.status == SessionStatus::Active)
        .count())
}

pub fn latest_session_id(store: &DiskStore) -> Result<Option<String>> {
    Ok(load_session_index(store)?
        .sessions
        .first()
        .map(|row| row.session_id.clone()))
}
