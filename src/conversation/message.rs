use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConversationRole {
    User,
    Onyx,
    System,
    Tool,
    Reviewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: String,
    pub role: ConversationRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

pub fn message(role: ConversationRole, content: impl Into<String>) -> ConversationMessage {
    ConversationMessage {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        role,
        content: content.into(),
        created_at: Utc::now(),
        metadata: serde_json::json!({}),
    }
}
