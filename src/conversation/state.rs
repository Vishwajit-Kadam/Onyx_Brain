use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::conversation::ConversationMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub session_id: String,
    pub current_topic: Option<String>,
    pub goals: Vec<String>,
    pub open_questions: Vec<String>,
    pub assumptions: Vec<String>,
    pub user_preferences: Vec<String>,
    pub mode: ConversationMode,
    pub turn_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConversationState {
    pub fn new(session_id: &str, mode: ConversationMode) -> Self {
        let now = Utc::now();
        Self {
            session_id: session_id.to_string(),
            current_topic: None,
            goals: Vec::new(),
            open_questions: Vec::new(),
            assumptions: vec![
                "Conversation is deterministic and local; no web access by default.".to_string(),
            ],
            user_preferences: Vec::new(),
            mode,
            turn_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}
