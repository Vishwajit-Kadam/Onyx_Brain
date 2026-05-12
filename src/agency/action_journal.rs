use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use uuid::Uuid;

use crate::{
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionType {
    CreateFile,
    ModifyFile,
    DeleteFile,
    CreateDirectory,
    RunCommand,
    UpdateMemory,
    UpdateSynapse,
    UpdateGoal,
    UpdateProjectState,
    CreateSnapshot,
    RestoreSnapshot,
    Maintenance,
    Benchmark,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionStatus {
    Planned,
    Running,
    Completed,
    Failed,
    RolledBack,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionJournalEntry {
    pub id: String,
    pub session_id: String,
    pub goal_id: Option<String>,
    pub project_id: Option<String>,
    pub action_type: ActionType,
    pub target_path: Option<String>,
    pub command: Option<String>,
    pub before_state_ref: Option<String>,
    pub after_state_ref: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ActionStatus,
    pub error: Option<String>,
    pub rollback_available: bool,
    pub rollback_ref: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionJournalSummary {
    pub id: String,
    pub session_id: String,
    pub goal_id: Option<String>,
    pub project_id: Option<String>,
    pub action_type: ActionType,
    pub target_path: Option<String>,
    pub command: Option<String>,
    pub status: ActionStatus,
    pub rollback_available: bool,
    pub rollback_ref: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionJournalIndex {
    #[serde(default)]
    pub entries: Vec<ActionJournalSummary>,
}

pub fn action_journal_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("action_journal_index.json")
}

pub fn journal_entry_path(
    store: &DiskStore,
    session_id: &str,
    entry_id: &str,
) -> std::path::PathBuf {
    store
        .paths
        .journal
        .join(session_id)
        .join(format!("{entry_id}.json"))
}

pub fn load_action_journal_index(store: &DiskStore) -> Result<ActionJournalIndex> {
    let path = action_journal_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(ActionJournalIndex::default())
    }
}

pub fn save_journal_entry(store: &DiskStore, entry: &ActionJournalEntry) -> Result<()> {
    fs::create_dir_all(store.paths.journal.join(&entry.session_id))?;
    save_json(
        &journal_entry_path(store, &entry.session_id, &entry.id),
        entry,
    )?;
    let mut index = load_action_journal_index(store)?;
    index.entries.retain(|row| row.id != entry.id);
    index.entries.push(ActionJournalSummary {
        id: entry.id.clone(),
        session_id: entry.session_id.clone(),
        goal_id: entry.goal_id.clone(),
        project_id: entry.project_id.clone(),
        action_type: entry.action_type.clone(),
        target_path: entry.target_path.clone(),
        command: entry.command.clone(),
        status: entry.status.clone(),
        rollback_available: entry.rollback_available,
        rollback_ref: entry.rollback_ref.clone(),
        created_at: entry.started_at,
    });
    index
        .entries
        .sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if index.entries.len() > 512 {
        index.entries.truncate(512);
    }
    save_json(&action_journal_index_path(store), &index)
}

pub fn start_journal_entry(
    store: &DiskStore,
    session_id: &str,
    action_type: ActionType,
    goal_id: Option<String>,
    project_id: Option<String>,
    target_path: Option<String>,
    command: Option<String>,
    before_state_ref: Option<String>,
    metadata: Value,
) -> Result<ActionJournalEntry> {
    let entry = ActionJournalEntry {
        id: format!("journal_{}_{}", timestamp_slug(), Uuid::new_v4()),
        session_id: session_id.to_string(),
        goal_id,
        project_id,
        action_type,
        target_path,
        command,
        before_state_ref,
        after_state_ref: None,
        started_at: Utc::now(),
        completed_at: None,
        status: ActionStatus::Running,
        error: None,
        rollback_available: false,
        rollback_ref: None,
        metadata,
    };
    save_journal_entry(store, &entry)?;
    Ok(entry)
}

pub fn complete_journal_entry(
    store: &DiskStore,
    entry: &mut ActionJournalEntry,
    after_state_ref: Option<String>,
    rollback_ref: Option<String>,
) -> Result<()> {
    entry.completed_at = Some(Utc::now());
    entry.status = ActionStatus::Completed;
    entry.after_state_ref = after_state_ref;
    entry.rollback_available = rollback_ref.is_some();
    entry.rollback_ref = rollback_ref;
    save_journal_entry(store, entry)
}

pub fn fail_journal_entry(
    store: &DiskStore,
    entry: &mut ActionJournalEntry,
    error: impl Into<String>,
) -> Result<()> {
    entry.completed_at = Some(Utc::now());
    entry.status = ActionStatus::Failed;
    entry.error = Some(error.into());
    save_journal_entry(store, entry)
}

pub fn mark_journal_rolled_back(store: &DiskStore, summary: &ActionJournalSummary) -> Result<()> {
    let path = journal_entry_path(store, &summary.session_id, &summary.id);
    let mut entry: ActionJournalEntry = load_json(&path)?;
    entry.status = ActionStatus::RolledBack;
    entry.completed_at = Some(Utc::now());
    save_journal_entry(store, &entry)
}

pub fn latest_journal_entries(
    store: &DiskStore,
    limit: usize,
    session_filter: Option<&str>,
) -> Result<Vec<ActionJournalSummary>> {
    let index = load_action_journal_index(store)?;
    let latest_session = if session_filter == Some("latest") {
        index.entries.first().map(|row| row.session_id.clone())
    } else {
        session_filter.map(ToOwned::to_owned)
    };
    Ok(index
        .entries
        .into_iter()
        .filter(|row| {
            latest_session
                .as_ref()
                .is_none_or(|session| &row.session_id == session)
        })
        .take(limit)
        .collect())
}

pub fn journal_count(store: &DiskStore) -> Result<usize> {
    Ok(load_action_journal_index(store)?.entries.len())
}

pub fn quick_journal(
    store: &DiskStore,
    session_id: &str,
    action_type: ActionType,
    project_id: Option<String>,
    target: Option<String>,
    command: Option<String>,
    rollback_ref: Option<String>,
) -> Result<String> {
    let mut entry = start_journal_entry(
        store,
        session_id,
        action_type,
        None,
        project_id,
        target,
        command,
        None,
        json!({}),
    )?;
    complete_journal_entry(store, &mut entry, None, rollback_ref)?;
    Ok(entry.id)
}
