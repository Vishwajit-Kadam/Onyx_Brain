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
    if path.exists() {
        load_json_with_recovery(&path)
    } else {
        Ok(ConversationIndex::default())
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
}
