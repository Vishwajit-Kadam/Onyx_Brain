use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};
use uuid::Uuid;

use crate::{
    agency::{ParsedGoal, ProjectState},
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCacheEntry {
    pub cache_id: String,
    pub normalized_goal_signature: String,
    pub intent: String,
    pub task_type: String,
    pub features: Vec<String>,
    pub plan_steps: Vec<String>,
    pub expected_files: Vec<String>,
    pub expected_tools: Vec<String>,
    pub success_count: u64,
    pub failure_count: u64,
    pub average_runtime_ms: f32,
    pub average_energy: f32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCacheMatch {
    pub cache_id: String,
    pub similarity_score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanCacheIndex {
    pub entries: BTreeMap<String, PlanCacheSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCacheSummary {
    pub cache_id: String,
    pub normalized_goal_signature: String,
    pub success_count: u64,
    pub failure_count: u64,
    pub average_runtime_ms: f32,
    pub average_energy: f32,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlanCacheOverview {
    pub entries: usize,
    pub top_cached_plans: Vec<String>,
    pub estimated_runtime_saved: f32,
    pub cache_hit_rate: f32,
    pub failed_cached_plans: usize,
}

pub fn plans_dir(store: &DiskStore) -> std::path::PathBuf {
    store.paths.data.join("cache").join("plans")
}

pub fn plan_cache_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("plan_cache_index.json")
}

pub fn plan_cache_path(store: &DiskStore, cache_id: &str) -> std::path::PathBuf {
    plans_dir(store).join(format!("{cache_id}.json"))
}

pub fn load_plan_cache_index(store: &DiskStore) -> Result<PlanCacheIndex> {
    let path = plan_cache_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(PlanCacheIndex::default())
    }
}

pub fn save_plan_cache_entry(store: &DiskStore, entry: &PlanCacheEntry) -> Result<()> {
    fs::create_dir_all(plans_dir(store))?;
    save_json(&plan_cache_path(store, &entry.cache_id), entry)?;
    let mut index = load_plan_cache_index(store)?;
    index.entries.insert(
        entry.cache_id.clone(),
        PlanCacheSummary {
            cache_id: entry.cache_id.clone(),
            normalized_goal_signature: entry.normalized_goal_signature.clone(),
            success_count: entry.success_count,
            failure_count: entry.failure_count,
            average_runtime_ms: entry.average_runtime_ms,
            average_energy: entry.average_energy,
            last_used_at: entry.last_used_at,
        },
    );
    save_json(&plan_cache_index_path(store), &index)
}

pub fn load_plan_cache_entry(store: &DiskStore, cache_id: &str) -> Result<PlanCacheEntry> {
    load_json(&plan_cache_path(store, cache_id))
}

pub fn find_cached_plan(store: &DiskStore, parsed: &ParsedGoal) -> Result<Option<PlanCacheMatch>> {
    let signature = goal_signature(parsed);
    let index = load_plan_cache_index(store)?;
    let mut best: Option<PlanCacheMatch> = None;
    for summary in index.entries.values() {
        let similarity = signature_similarity(&signature, &summary.normalized_goal_signature);
        if similarity >= 0.68
            && best
                .as_ref()
                .is_none_or(|current| similarity > current.similarity_score)
        {
            best = Some(PlanCacheMatch {
                cache_id: summary.cache_id.clone(),
                similarity_score: similarity,
                reason: "normalized goal signature matched cached successful plan".to_string(),
            });
        }
    }
    Ok(best)
}

pub fn store_successful_plan(
    store: &DiskStore,
    parsed: &ParsedGoal,
    state: &ProjectState,
    plan_steps: Vec<String>,
    runtime_ms: u64,
    energy: f32,
) -> Result<PlanCacheEntry> {
    let signature = goal_signature(parsed);
    let existing = load_plan_cache_index(store)?
        .entries
        .values()
        .find(|summary| summary.normalized_goal_signature == signature)
        .cloned();
    let now = Utc::now();
    let mut entry = if let Some(summary) = existing {
        load_plan_cache_entry(store, &summary.cache_id)?
    } else {
        PlanCacheEntry {
            cache_id: format!("plan_{}", Uuid::new_v4()),
            normalized_goal_signature: signature,
            intent: format!("{:?}", parsed.intent),
            task_type: "Code".to_string(),
            features: parsed.requested_features.clone(),
            plan_steps,
            expected_files: Vec::new(),
            expected_tools: vec!["CodeEditorTool".to_string(), "TerminalTool".to_string()],
            success_count: 0,
            failure_count: 0,
            average_runtime_ms: 0.0,
            average_energy: 0.0,
            last_used_at: None,
            created_at: now,
            updated_at: now,
        }
    };
    if state.status == "Completed" {
        entry.success_count += 1;
    } else {
        entry.failure_count += 1;
    }
    let count = entry.success_count + entry.failure_count;
    entry.average_runtime_ms =
        rolling_average(entry.average_runtime_ms, runtime_ms as f32, count.max(1));
    entry.average_energy = rolling_average(entry.average_energy, energy, count.max(1));
    entry.expected_files = state
        .files_created
        .iter()
        .chain(state.files_modified.iter())
        .cloned()
        .collect();
    entry.last_used_at = Some(now);
    entry.updated_at = now;
    save_plan_cache_entry(store, &entry)?;
    Ok(entry)
}

pub fn mark_cache_used(
    store: &DiskStore,
    cache_id: &str,
    success: bool,
    runtime_ms: u64,
    energy: f32,
) -> Result<()> {
    let mut entry = load_plan_cache_entry(store, cache_id)?;
    entry.success_count += u64::from(success);
    entry.failure_count += u64::from(!success);
    let count = entry.success_count + entry.failure_count;
    entry.average_runtime_ms = rolling_average(entry.average_runtime_ms, runtime_ms as f32, count);
    entry.average_energy = rolling_average(entry.average_energy, energy, count);
    entry.last_used_at = Some(Utc::now());
    entry.updated_at = Utc::now();
    save_plan_cache_entry(store, &entry)
}

pub fn plan_cache_overview(store: &DiskStore) -> Result<PlanCacheOverview> {
    let index = load_plan_cache_index(store)?;
    let mut rows = index.entries.values().cloned().collect::<Vec<_>>();
    rows.sort_by(|a, b| b.success_count.cmp(&a.success_count));
    let total_hits: u64 = rows
        .iter()
        .map(|row| row.success_count + row.failure_count)
        .sum();
    let successes: u64 = rows.iter().map(|row| row.success_count).sum();
    let cache_hit_rate = if total_hits == 0 {
        0.0
    } else {
        successes as f32 / total_hits as f32
    };
    Ok(PlanCacheOverview {
        entries: rows.len(),
        top_cached_plans: rows
            .iter()
            .take(10)
            .map(|row| {
                format!(
                    "{} success {} failure {} runtime {:.0}ms energy {:.1}",
                    row.normalized_goal_signature,
                    row.success_count,
                    row.failure_count,
                    row.average_runtime_ms,
                    row.average_energy
                )
            })
            .collect(),
        estimated_runtime_saved: rows
            .iter()
            .map(|row| row.success_count as f32 * row.average_runtime_ms * 0.15)
            .sum(),
        cache_hit_rate,
        failed_cached_plans: rows.iter().filter(|row| row.failure_count > 0).count(),
    })
}

pub fn goal_signature(parsed: &ParsedGoal) -> String {
    let mut features = parsed
        .requested_features
        .iter()
        .map(|feature| normalize(feature))
        .collect::<Vec<_>>();
    features.sort();
    features.dedup();
    format!("{:?}:rust_cli:{}", parsed.intent, features.join(","))
}

fn signature_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }
    let a_parts = a
        .split([':', ','])
        .collect::<std::collections::BTreeSet<_>>();
    let b_parts = b
        .split([':', ','])
        .collect::<std::collections::BTreeSet<_>>();
    if a_parts.is_empty() {
        return 0.0;
    }
    let overlap = a_parts.intersection(&b_parts).count();
    overlap as f32 / a_parts.len() as f32
}

fn rolling_average(previous: f32, next: f32, count: u64) -> f32 {
    if count <= 1 {
        next
    } else {
        ((previous * (count - 1) as f32) + next) / count as f32
    }
}

fn normalize(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
