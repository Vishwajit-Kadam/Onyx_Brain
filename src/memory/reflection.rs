use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::storage::{save_json, DiskStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionMemory {
    pub id: String,
    pub session_id: String,
    pub goal_type: String,
    pub successful_patterns: Vec<String>,
    pub recurring_issues: Vec<String>,
    pub useful_recipes: Vec<String>,
    pub failed_assumptions: Vec<String>,
    pub next_time_improvements: Vec<String>,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

pub fn save_reflection(
    store: &DiskStore,
    session_id: &str,
    goal_type: &str,
    useful_recipes: Vec<String>,
    recurring_issues: Vec<String>,
) -> Result<ReflectionMemory> {
    let item = ReflectionMemory {
        id: format!("reflection_{}", Uuid::new_v4()),
        session_id: session_id.to_string(),
        goal_type: goal_type.to_string(),
        successful_patterns: vec![
            "dependency-aware planning".to_string(),
            "quality review before export".to_string(),
        ],
        recurring_issues,
        useful_recipes,
        failed_assumptions: Vec::new(),
        next_time_improvements: vec![
            "prefer complete artifact packs from the first pass".to_string()
        ],
        confidence: 0.8,
        created_at: Utc::now(),
    };
    let dir = store.paths.data.join("reflections");
    fs::create_dir_all(&dir)?;
    save_json(&dir.join(format!("{}.json", item.id)), &item)?;
    Ok(item)
}

pub fn recent_reflections(store: &DiskStore) -> Result<Vec<ReflectionMemory>> {
    let dir = store.paths.data.join("reflections");
    fs::create_dir_all(&dir)?;
    let mut reflections: Vec<ReflectionMemory> = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Ok(item) = crate::storage::load_json(&path) {
                reflections.push(item);
            }
        }
    }
    reflections.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(reflections)
}
