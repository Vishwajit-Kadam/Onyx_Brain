use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{
    agency::IntentKind,
    core::Priority,
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalStatus {
    Active,
    Completed,
    Failed,
    Blocked,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalMemoryItem {
    pub goal_id: String,
    pub title: String,
    pub original_prompt: String,
    pub parsed_intent: IntentKind,
    pub project_name: Option<String>,
    pub status: GoalStatus,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub linked_project_id: Option<String>,
    pub linked_memories: Vec<String>,
    pub linked_skills: Vec<String>,
    pub success_score: f32,
    pub energy_spent: u64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoalIndex(pub BTreeMap<String, String>);

pub fn goals_dir(store: &DiskStore) -> PathBuf {
    store.paths.data.join("goals")
}

pub fn goal_path(store: &DiskStore, goal_id: &str) -> PathBuf {
    goals_dir(store).join(format!("{goal_id}.json"))
}

pub fn save_goal(store: &DiskStore, goal: &GoalMemoryItem) -> Result<()> {
    fs::create_dir_all(goals_dir(store))?;
    save_json(&goal_path(store, &goal.goal_id), goal)?;
    let mut index = load_goal_index(store)?;
    index.0.insert(goal.goal_id.clone(), goal.title.clone());
    save_json(&store.paths.indexes.join("goal_index.json"), &index)
}

pub fn load_goal(store: &DiskStore, goal_id: &str) -> Result<GoalMemoryItem> {
    load_json(&goal_path(store, goal_id))
}

pub fn load_goal_index(store: &DiskStore) -> Result<GoalIndex> {
    let path = store.paths.indexes.join("goal_index.json");
    if path.exists() {
        load_json(&path)
    } else {
        Ok(GoalIndex::default())
    }
}

pub fn list_goals(store: &DiskStore) -> Result<Vec<GoalMemoryItem>> {
    let dir = goals_dir(store);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut goals: Vec<GoalMemoryItem> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            goals.push(load_json(&path)?);
        }
    }
    goals.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(goals)
}
