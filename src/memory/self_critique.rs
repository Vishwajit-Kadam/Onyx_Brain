use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::{
    memory::{MemoryItem, MemoryType},
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfCritiqueMemory {
    pub id: String,
    pub session_id: String,
    pub issue_pattern: String,
    pub fix_applied: String,
    pub success: bool,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

pub fn save_self_critique(
    store: &DiskStore,
    session_id: &str,
    issue_pattern: &str,
    fix_applied: &str,
    success: bool,
) -> Result<SelfCritiqueMemory> {
    let item = SelfCritiqueMemory {
        id: format!("critique_{}", Uuid::new_v4()),
        session_id: session_id.to_string(),
        issue_pattern: issue_pattern.to_string(),
        fix_applied: fix_applied.to_string(),
        success,
        confidence: if success { 0.8 } else { 0.4 },
        created_at: Utc::now(),
    };
    let dir = store.paths.data.join("self_critique");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join(format!("{}.json", item.id)), &item)?;
    let mut memory = MemoryItem::new(
        format!("memory_{}", item.id),
        MemoryType::Procedural,
        format!("Self critique: {}", item.issue_pattern),
        format!(
            "Issue: {}. Fix: {}. Success: {}",
            item.issue_pattern, item.fix_applied, item.success
        ),
        vec![
            "self_critique".to_string(),
            "autonomy".to_string(),
            "revision".to_string(),
        ],
        Vec::new(),
    );
    memory.importance = 0.55;
    store.save_memory(&memory)?;
    Ok(item)
}
