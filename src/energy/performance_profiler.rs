use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::{
    agency::FastPathDecision,
    energy::AdaptiveBudgetDecision,
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeBreakdown {
    pub total_runtime_ms: u64,
    pub brain_runtime_ms: u64,
    pub routing_runtime_ms: u64,
    pub memory_runtime_ms: u64,
    pub planning_runtime_ms: u64,
    pub tool_runtime_ms: u64,
    pub cargo_check_runtime_ms: Option<u64>,
    pub cargo_test_runtime_ms: Option<u64>,
    pub filesystem_runtime_ms: u64,
    pub reporting_runtime_ms: u64,
    pub maintenance_runtime_ms: u64,
    pub unknown_runtime_ms: u64,
}

impl RuntimeBreakdown {
    pub fn new(total_runtime_ms: u64) -> Self {
        Self {
            total_runtime_ms,
            unknown_runtime_ms: total_runtime_ms,
            ..Self::default()
        }
    }

    pub fn finalize_unknown(mut self) -> Self {
        let known = self.brain_runtime_ms
            + self.routing_runtime_ms
            + self.memory_runtime_ms
            + self.planning_runtime_ms
            + self.tool_runtime_ms
            + self.reporting_runtime_ms
            + self.maintenance_runtime_ms;
        self.unknown_runtime_ms = self.total_runtime_ms.saturating_sub(known);
        self
    }

    pub fn cargo_runtime_ms(&self) -> u64 {
        self.cargo_check_runtime_ms.unwrap_or(0) + self.cargo_test_runtime_ms.unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    pub id: String,
    pub command_name: String,
    pub task_type: String,
    pub project_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub runtime_ms: u64,
    pub estimated_energy: f32,
    pub active_neurons: usize,
    pub loaded_synapses: usize,
    pub memories_loaded: usize,
    pub skills_reused: usize,
    pub tool_actions: usize,
    pub cargo_check_runtime_ms: Option<u64>,
    pub cargo_test_runtime_ms: Option<u64>,
    pub success: bool,
    pub final_score: f32,
    #[serde(default)]
    pub adaptive_budget: Option<AdaptiveBudgetDecision>,
    #[serde(default)]
    pub habits_used: usize,
    #[serde(default)]
    pub cache_hits: usize,
    #[serde(default)]
    pub runtime_breakdown: RuntimeBreakdown,
    #[serde(default)]
    pub habit_created: bool,
    #[serde(default)]
    pub habit_strengthened: bool,
    #[serde(default)]
    pub habit_id: Option<String>,
    #[serde(default)]
    pub fast_path_decision: Option<FastPathDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfileSummary {
    pub id: String,
    pub command_name: String,
    pub task_type: String,
    pub project_name: Option<String>,
    pub runtime_ms: u64,
    pub estimated_energy: f32,
    pub success: bool,
    pub final_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceIndex {
    pub profiles: Vec<PerformanceProfileSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceOverview {
    pub profile_count: usize,
    pub average_runtime_last_5: f32,
    pub average_energy_last_5: f32,
    pub slowest_command_types: Vec<String>,
    pub estimated_budget_savings: f32,
    #[serde(default)]
    pub average_brain_runtime_last_5: f32,
    #[serde(default)]
    pub average_tool_runtime_last_5: f32,
    #[serde(default)]
    pub average_cargo_runtime_last_5: f32,
}

pub fn new_profile_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn profile_dir(store: &DiskStore) -> std::path::PathBuf {
    store.paths.logs.join("performance_profiles")
}

pub fn performance_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("performance_index.json")
}

pub fn load_performance_index(store: &DiskStore) -> Result<PerformanceIndex> {
    let path = performance_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        rebuild_performance_index(store)
    }
}

pub fn save_performance_profile(store: &DiskStore, profile: &PerformanceProfile) -> Result<()> {
    fs::create_dir_all(profile_dir(store))?;
    let path = profile_dir(store).join(format!("{}.json", profile.id));
    save_json(&path, profile)?;
    let mut index = load_performance_index(store)?;
    index.profiles.push(PerformanceProfileSummary {
        id: profile.id.clone(),
        command_name: profile.command_name.clone(),
        task_type: profile.task_type.clone(),
        project_name: profile.project_name.clone(),
        runtime_ms: profile.runtime_ms,
        estimated_energy: profile.estimated_energy,
        success: profile.success,
        final_score: profile.final_score,
        created_at: profile.ended_at,
    });
    index
        .profiles
        .sort_by(|a, b| a.created_at.cmp(&b.created_at));
    if index.profiles.len() > 256 {
        index.profiles = index.profiles.split_off(index.profiles.len() - 256);
    }
    save_json(&performance_index_path(store), &index)
}

pub fn rebuild_performance_index(store: &DiskStore) -> Result<PerformanceIndex> {
    let mut index = PerformanceIndex::default();
    let dir = profile_dir(store);
    if dir.exists() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Ok(profile) = load_json::<PerformanceProfile>(&path) else {
                continue;
            };
            index.profiles.push(PerformanceProfileSummary {
                id: profile.id,
                command_name: profile.command_name,
                task_type: profile.task_type,
                project_name: profile.project_name,
                runtime_ms: profile.runtime_ms,
                estimated_energy: profile.estimated_energy,
                success: profile.success,
                final_score: profile.final_score,
                created_at: profile.ended_at,
            });
        }
    }
    index
        .profiles
        .sort_by(|a, b| a.created_at.cmp(&b.created_at));
    save_json(&performance_index_path(store), &index)?;
    Ok(index)
}

pub fn performance_overview(store: &DiskStore) -> Result<PerformanceOverview> {
    let index = load_performance_index(store)?;
    let recent = index.profiles.iter().rev().take(5).collect::<Vec<_>>();
    let average_runtime_last_5 = average(recent.iter().map(|profile| profile.runtime_ms as f32));
    let average_energy_last_5 = average(recent.iter().map(|profile| profile.estimated_energy));
    let mut full_recent = Vec::new();
    for summary in &recent {
        if let Ok(profile) = load_json::<PerformanceProfile>(
            &profile_dir(store).join(format!("{}.json", summary.id)),
        ) {
            full_recent.push(profile);
        }
    }
    let average_brain_runtime_last_5 = average(
        full_recent
            .iter()
            .map(|profile| profile.runtime_breakdown.brain_runtime_ms as f32),
    );
    let average_tool_runtime_last_5 = average(
        full_recent
            .iter()
            .map(|profile| profile.runtime_breakdown.tool_runtime_ms as f32),
    );
    let average_cargo_runtime_last_5 = average(
        full_recent
            .iter()
            .map(|profile| profile.runtime_breakdown.cargo_runtime_ms() as f32),
    );
    let mut by_command = std::collections::BTreeMap::<String, (u64, u64)>::new();
    for profile in &index.profiles {
        let entry = by_command
            .entry(profile.command_name.clone())
            .or_insert((0, 0));
        entry.0 += profile.runtime_ms;
        entry.1 += 1;
    }
    let mut slowest = by_command
        .into_iter()
        .map(|(command, (runtime, count))| {
            (
                runtime as f32 / count.max(1) as f32,
                format!("{command}: {:.0} ms", runtime as f32 / count.max(1) as f32),
            )
        })
        .collect::<Vec<_>>();
    slowest.sort_by(|a, b| b.0.total_cmp(&a.0));
    let savings = index
        .profiles
        .iter()
        .rev()
        .take(16)
        .filter(|profile| profile.success)
        .count() as f32
        * 0.02;
    Ok(PerformanceOverview {
        profile_count: index.profiles.len(),
        average_runtime_last_5,
        average_energy_last_5,
        slowest_command_types: slowest.into_iter().take(5).map(|(_, row)| row).collect(),
        estimated_budget_savings: savings.clamp(0.0, 0.3),
        average_brain_runtime_last_5,
        average_tool_runtime_last_5,
        average_cargo_runtime_last_5,
    })
}

fn average(values: impl Iterator<Item = f32>) -> f32 {
    let mut total = 0.0;
    let mut count = 0_u32;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}
