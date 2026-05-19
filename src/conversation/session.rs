//! Conversation session management — tracks message history, token metrics,
//! and turn-taking limits for deterministic conversation persistence.
//!
//! Provides atomic operations for appending turns, truncating over-length
//! histories, and maintaining an index of active sessions.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{
    conversation::{
        message, ConversationMessage, ConversationMode, ConversationRole, ConversationState,
    },
    storage::{load_json_with_recovery, save_json, save_json_with_backup, DiskStore},
    utils::time::timestamp_slug,
};

/// Maximum number of turns allowed in a single session before truncation is required.
pub const MAX_SESSION_TURNS: u64 = 100;

/// Hard limit on index size to prevent the `conversation_index.json` from unbounded growth.
pub const MAX_INDEX_ENTRIES: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSessionSummary {
    pub session_id: String,
    pub mode: ConversationMode,
    pub topic: Option<String>,
    pub message_count: usize,
    /// Token estimate — defaults to 0 for indexes saved before this field existed.
    #[serde(default)]
    pub estimated_tokens: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationIndex {
    #[serde(default)]
    pub sessions: Vec<ConversationSessionSummary>,
}

pub fn conversation_index_path(store: &DiskStore) -> PathBuf {
    store.paths.indexes.join("conversation_index.json")
}

pub fn load_conversation_index(store: &DiskStore) -> Result<ConversationIndex> {
    let path = conversation_index_path(store);
    if !path.exists() {
        return Ok(ConversationIndex::default());
    }

    // Try the normal deserialization path (now tolerant of missing fields
    // thanks to #[serde(default)] on estimated_tokens).
    let index: ConversationIndex = match load_json_with_recovery(&path) {
        Ok(idx) => idx,
        Err(_) => {
            // The file is corrupt beyond schema-level fixes. Archive it and
            // start fresh so that conversation commands keep working.
            eprintln!(
                "Warning: Conversation index was corrupt and has been archived. A fresh index was created."
            );
            let archive_dir = store.paths.recovery.join("corrupt_archive");
            let _ = fs::create_dir_all(&archive_dir);
            let archive_name = format!("{}_conversation_index.json", timestamp_slug());
            let _ = fs::rename(&path, archive_dir.join(&archive_name));
            let fresh = ConversationIndex::default();
            let _ = save_json(&path, &fresh);
            return Ok(fresh);
        }
    };

    // Schema migration: fill estimated_tokens for entries from older schema
    // (they deserialize with the default value of 0).
    let needs_migration = index
        .sessions
        .iter()
        .any(|s| s.estimated_tokens == 0 && s.message_count > 0);

    if needs_migration {
        let mut migrated = index;
        for session in &mut migrated.sessions {
            if session.estimated_tokens == 0 && session.message_count > 0 {
                // Heuristic: ~25 tokens per message on average when we cannot
                // load the actual messages. This is a safe low estimate.
                session.estimated_tokens = session.message_count * 25;
            }
        }
        eprintln!("Conversation index was upgraded (added estimated_tokens to older entries).");
        let _ = save_json_with_backup(&path, &migrated);
        Ok(migrated)
    } else {
        Ok(index)
    }
}

pub fn save_conversation_index(store: &DiskStore, index: &ConversationIndex) -> Result<()> {
    save_json_with_backup(&conversation_index_path(store), index)
}

pub fn start_conversation(store: &DiskStore, mode: ConversationMode) -> Result<ConversationState> {
    store.ensure_layout()?;
    let session_id = format!("conversation_{}_{}", timestamp_slug(), uuid::Uuid::new_v4());
    let state = ConversationState::new(&session_id, mode.clone());

    fs::create_dir_all(conversation_dir(store, &session_id))
        .with_context(|| format!("creating conversation dir {}", session_id))?;

    save_state(store, &state)?;
    save_messages(store, &session_id, &[])?;
    update_conversation_index(store, &state, 0, 0)?;

    Ok(state)
}

pub fn conversation_dir(store: &DiskStore, session_id: &str) -> PathBuf {
    store.paths.conversations.join(session_id)
}

pub fn messages_path(store: &DiskStore, session_id: &str) -> PathBuf {
    conversation_dir(store, session_id).join("messages.json")
}

pub fn state_path(store: &DiskStore, session_id: &str) -> PathBuf {
    conversation_dir(store, session_id).join("state.json")
}

pub fn load_state(store: &DiskStore, session_id: &str) -> Result<ConversationState> {
    load_json_with_recovery(&state_path(store, session_id))
}

pub fn save_state(store: &DiskStore, state: &ConversationState) -> Result<()> {
    fs::create_dir_all(conversation_dir(store, &state.session_id))?;
    save_json(&state_path(store, &state.session_id), state)
}

pub fn load_messages(store: &DiskStore, session_id: &str) -> Result<Vec<ConversationMessage>> {
    let path = messages_path(store, session_id);
    if path.exists() {
        load_json_with_recovery(&path)
    } else {
        Ok(Vec::new())
    }
}

pub fn save_messages(
    store: &DiskStore,
    session_id: &str,
    messages: &[ConversationMessage],
) -> Result<()> {
    fs::create_dir_all(conversation_dir(store, session_id))?;
    // We use atomic write with backup for message history since it's precious user data.
    save_json_with_backup(&messages_path(store, session_id), &messages.to_vec())
}

pub fn latest_conversation_id(store: &DiskStore) -> Option<String> {
    load_conversation_index(store)
        .ok()
        .and_then(|index| index.sessions.first().map(|row| row.session_id.clone()))
}

/// Estimate tokens roughly as 1 token per 4 bytes of text.
fn estimate_tokens(messages: &[ConversationMessage]) -> usize {
    messages.iter().map(|m| m.content.len() / 4).sum()
}

pub fn update_conversation_index(
    store: &DiskStore,
    state: &ConversationState,
    message_count: usize,
    estimated_tokens: usize,
) -> Result<()> {
    let mut index = load_conversation_index(store)?;
    index
        .sessions
        .retain(|row| row.session_id != state.session_id);

    index.sessions.push(ConversationSessionSummary {
        session_id: state.session_id.clone(),
        mode: state.mode.clone(),
        topic: state.current_topic.clone(),
        message_count,
        estimated_tokens,
        created_at: state.created_at,
        updated_at: state.updated_at,
    });

    index
        .sessions
        .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    if index.sessions.len() > MAX_INDEX_ENTRIES {
        index.sessions.truncate(MAX_INDEX_ENTRIES);
    }

    save_conversation_index(store, &index)
}

/// Appends a user-turn and system-turn to the session, automatically truncating
/// if the session exceeds `MAX_SESSION_TURNS`.
pub fn append_turn(
    store: &DiskStore,
    state: &mut ConversationState,
    user_input: &str,
    onyx_output: &str,
    quality: &serde_json::Value,
) -> Result<Vec<ConversationMessage>> {
    let mut messages = load_messages(store, &state.session_id)?;

    // Add new turns
    messages.push(message(ConversationRole::User, user_input));
    let mut response = message(ConversationRole::Onyx, onyx_output);
    response.metadata = quality.clone();
    messages.push(response);

    // Enforce context bounds
    if state.turn_count >= MAX_SESSION_TURNS && messages.len() > 10 {
        // Keep the first message (usually system prompt/context) and the latest N messages
        let tail_start = messages.len().saturating_sub(20);
        let mut truncated = vec![messages[0].clone()];
        truncated.extend_from_slice(&messages[tail_start..]);
        messages = truncated;
    }

    state.turn_count += 1;
    state.updated_at = Utc::now();

    let tokens = estimate_tokens(&messages);

    save_messages(store, &state.session_id, &messages)?;
    save_state(store, state)?;
    update_conversation_index(store, state, messages.len(), tokens)?;

    Ok(messages)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> DiskStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp = std::env::temp_dir().join(format!("onyx_session_test_{}", id));
        let _ = fs::remove_dir_all(&temp);
        let store = DiskStore::new(&temp);
        store.ensure_layout().unwrap();
        store
    }

    #[test]
    fn start_conversation_initializes_empty_session() {
        let store = test_store();
        let state = start_conversation(&store, ConversationMode::Standard).unwrap();
        assert_eq!(state.turn_count, 0);

        let msgs = load_messages(&store, &state.session_id).unwrap();
        assert!(msgs.is_empty());

        let index = load_conversation_index(&store).unwrap();
        assert_eq!(index.sessions.len(), 1);
        assert_eq!(index.sessions[0].session_id, state.session_id);
    }

    #[test]
    fn append_turn_adds_messages_and_updates_index() {
        let store = test_store();
        let mut state = start_conversation(&store, ConversationMode::Standard).unwrap();

        append_turn(
            &store,
            &mut state,
            "hello",
            "hi there",
            &serde_json::json!({}),
        )
        .unwrap();

        let msgs = load_messages(&store, &state.session_id).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, ConversationRole::User);
        assert_eq!(msgs[1].role, ConversationRole::Onyx);
        assert_eq!(state.turn_count, 1);

        let index = load_conversation_index(&store).unwrap();
        assert_eq!(index.sessions[0].message_count, 2);
        assert!(index.sessions[0].estimated_tokens > 0);
    }

    #[test]
    fn latest_conversation_id_returns_most_recently_updated() {
        let store = test_store();
        let state1 = start_conversation(&store, ConversationMode::Standard).unwrap();
        // Artificial delay not strictly needed because indexing sorts by updated_at,
        // but let's update state2 to make it definitively latest.
        let mut state2 = start_conversation(&store, ConversationMode::Standard).unwrap();
        append_turn(&store, &mut state2, "ping", "pong", &serde_json::json!({})).unwrap();

        let latest = latest_conversation_id(&store).unwrap();
        assert_eq!(latest, state2.session_id);
        assert_ne!(latest, state1.session_id);
    }

    #[test]
    fn estimate_tokens_is_approximate() {
        let msgs = vec![
            message(ConversationRole::User, "1234"),     // 1 token
            message(ConversationRole::Onyx, "12345678"), // 2 tokens
        ];
        let tokens = estimate_tokens(&msgs);
        assert_eq!(tokens, 3);
    }

    #[test]
    fn truncate_index_at_max_entries() {
        let store = test_store();
        let state = start_conversation(&store, ConversationMode::Standard).unwrap();
        let mut index = ConversationIndex::default();
        for i in 0..300 {
            index.sessions.push(ConversationSessionSummary {
                session_id: format!("sess_{}", i),
                mode: ConversationMode::Standard,
                topic: None,
                message_count: 2,
                estimated_tokens: 10,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
        save_conversation_index(&store, &index).unwrap();
        // Force an update to trigger truncation
        update_conversation_index(&store, &state, 2, 10).unwrap();
        let loaded = load_conversation_index(&store).unwrap();
        assert_eq!(loaded.sessions.len(), MAX_INDEX_ENTRIES);
    }

    /// Old conversation_index.json without estimated_tokens loads successfully.
    #[test]
    fn old_index_without_estimated_tokens_loads() {
        let store = test_store();
        let path = conversation_index_path(&store);
        // Write an old-schema index that lacks estimated_tokens entirely.
        let old_json = serde_json::json!({
            "sessions": [
                {
                    "session_id": "old_session_1",
                    "mode": "Standard",
                    "topic": "Hello World",
                    "message_count": 4,
                    "created_at": "2026-05-01T00:00:00Z",
                    "updated_at": "2026-05-01T00:01:00Z"
                },
                {
                    "session_id": "old_session_2",
                    "mode": "Debate",
                    "topic": "Open Source",
                    "message_count": 6,
                    "created_at": "2026-05-02T00:00:00Z",
                    "updated_at": "2026-05-02T00:01:00Z"
                }
            ]
        });
        crate::storage::save_json(&path, &old_json).unwrap();

        let index = load_conversation_index(&store).unwrap();
        assert_eq!(index.sessions.len(), 2);
    }

    /// Migration fills estimated_tokens from message_count heuristic.
    #[test]
    fn migration_fills_estimated_tokens() {
        let store = test_store();
        let path = conversation_index_path(&store);
        let old_json = serde_json::json!({
            "sessions": [
                {
                    "session_id": "migrate_1",
                    "mode": "Standard",
                    "topic": "Test",
                    "message_count": 4,
                    "created_at": "2026-05-01T00:00:00Z",
                    "updated_at": "2026-05-01T00:01:00Z"
                }
            ]
        });
        crate::storage::save_json(&path, &old_json).unwrap();

        let index = load_conversation_index(&store).unwrap();
        // message_count 4 × 25 tokens heuristic = 100
        assert_eq!(index.sessions[0].estimated_tokens, 100);
    }

    /// Migrated index saves back to disk successfully.
    #[test]
    fn migrated_index_saves_successfully() {
        let store = test_store();
        let path = conversation_index_path(&store);
        let old_json = serde_json::json!({
            "sessions": [
                {
                    "session_id": "save_test",
                    "mode": "Teacher",
                    "topic": "Basics",
                    "message_count": 2,
                    "created_at": "2026-05-01T00:00:00Z",
                    "updated_at": "2026-05-01T00:01:00Z"
                }
            ]
        });
        crate::storage::save_json(&path, &old_json).unwrap();

        // First load triggers migration + save
        let _ = load_conversation_index(&store).unwrap();

        // Second load should succeed and contain the migrated value
        let reloaded = load_conversation_index(&store).unwrap();
        assert_eq!(reloaded.sessions[0].estimated_tokens, 50); // 2 × 25
        assert_eq!(reloaded.sessions.len(), 1);
    }

    /// Corrupt conversation index is archived and rebuilt.
    #[test]
    fn corrupt_index_archived_and_rebuilt() {
        let store = test_store();
        let path = conversation_index_path(&store);
        // Write invalid JSON
        fs::write(&path, "this is not valid json {{{").unwrap();

        let index = load_conversation_index(&store).unwrap();
        assert!(index.sessions.is_empty(), "rebuilt index should be empty");

        // The corrupt file should have been archived
        let archive_dir = store.paths.recovery.join("corrupt_archive");
        assert!(
            archive_dir.exists(),
            "corrupt_archive directory should exist"
        );
        let archived: Vec<_> = fs::read_dir(&archive_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            !archived.is_empty(),
            "at least one archived file should exist"
        );
    }

    /// Chat command does not crash when old index exists.
    #[test]
    fn chat_command_survives_old_index() {
        let store = test_store();
        let path = conversation_index_path(&store);
        let old_json = serde_json::json!({
            "sessions": [{
                "session_id": "chat_old",
                "mode": "Standard",
                "topic": "Hello",
                "message_count": 2,
                "created_at": "2026-05-01T00:00:00Z",
                "updated_at": "2026-05-01T00:01:00Z"
            }]
        });
        crate::storage::save_json(&path, &old_json).unwrap();

        // Starting a new conversation should succeed despite the old index
        let state = start_conversation(&store, ConversationMode::Standard).unwrap();
        assert_eq!(state.turn_count, 0);

        let index = load_conversation_index(&store).unwrap();
        assert!(index.sessions.len() >= 2); // old entry + new one
    }

    /// Mode debate does not crash when old index exists.
    #[test]
    fn mode_debate_survives_old_index() {
        let store = test_store();
        let path = conversation_index_path(&store);
        let old_json = serde_json::json!({
            "sessions": [{
                "session_id": "debate_old",
                "mode": "Debate",
                "topic": "Test",
                "message_count": 2,
                "created_at": "2026-05-01T00:00:00Z",
                "updated_at": "2026-05-01T00:01:00Z"
            }]
        });
        crate::storage::save_json(&path, &old_json).unwrap();

        let state = start_conversation(&store, ConversationMode::Debate).unwrap();
        assert_eq!(state.turn_count, 0);
        assert_eq!(state.mode, ConversationMode::Debate);
    }
}
