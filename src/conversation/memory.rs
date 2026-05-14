use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    conversation::{load_messages, ConversationRole, ConversationState},
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMemory {
    pub id: String,
    pub session_id: String,
    pub topic: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub user_preferences: Vec<String>,
    pub open_questions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub importance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMemorySummary {
    pub id: String,
    pub session_id: String,
    pub topic: String,
    pub summary: String,
    pub importance: f32,
    pub updated_at: DateTime<Utc>,
}

pub fn save_conversation_memory(
    store: &DiskStore,
    state: &ConversationState,
) -> Result<ConversationMemory> {
    fs::create_dir_all(&store.paths.conversation_memory)?;
    let messages = load_messages(store, &state.session_id)?;
    let user_lines = messages
        .iter()
        .filter(|msg| msg.role == ConversationRole::User)
        .map(|msg| msg.content.clone())
        .collect::<Vec<_>>();
    let topic = state
        .current_topic
        .clone()
        .or_else(|| user_lines.last().cloned())
        .unwrap_or_else(|| "Conversation".to_string());
    let now = Utc::now();
    let memory = ConversationMemory {
        id: format!("conversation_memory_{}", uuid::Uuid::new_v4()),
        session_id: state.session_id.clone(),
        topic,
        summary: summarize_lines(&user_lines),
        key_points: user_lines.iter().take(5).cloned().collect(),
        user_preferences: state.user_preferences.clone(),
        open_questions: state.open_questions.clone(),
        created_at: now,
        updated_at: now,
        importance: 0.7,
    };
    save_json(
        &store
            .paths
            .conversation_memory
            .join(format!("{}.json", memory.id)),
        &memory,
    )?;
    Ok(memory)
}

pub fn recent_conversation_memory(store: &DiskStore) -> Result<Vec<ConversationMemorySummary>> {
    fs::create_dir_all(&store.paths.conversation_memory)?;
    let mut rows = Vec::new();
    for entry in fs::read_dir(&store.paths.conversation_memory)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Ok(memory) = load_json::<ConversationMemory>(&path) {
                rows.push(ConversationMemorySummary {
                    id: memory.id,
                    session_id: memory.session_id,
                    topic: memory.topic,
                    summary: memory.summary,
                    importance: memory.importance,
                    updated_at: memory.updated_at,
                });
            }
        }
    }
    rows.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    rows.truncate(25);
    Ok(rows)
}

fn summarize_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        "No user messages recorded yet.".to_string()
    } else {
        format!(
            "Conversation covered: {}",
            lines
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}
