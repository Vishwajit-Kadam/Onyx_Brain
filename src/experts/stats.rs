use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::storage::DiskStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertStats {
    pub name: String,
    pub total_runs: u64,
    pub successes: u64,
    pub failures: u64,
    pub average_energy_cost: f32,
    pub average_runtime_ms: f32,
    pub confidence: f32,
}

impl ExpertStats {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            total_runs: 0,
            successes: 0,
            failures: 0,
            average_energy_cost: 0.3,
            average_runtime_ms: 0.0,
            confidence: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExpertStatsIndex(pub BTreeMap<String, ExpertStats>);

pub fn ensure_default_expert_stats(store: &DiskStore, names: &[&str]) -> Result<ExpertStatsIndex> {
    let mut index = store.expert_stats_index()?;
    for name in names {
        index
            .0
            .entry((*name).to_string())
            .or_insert_with(|| ExpertStats::new(*name));
    }
    store.save_expert_stats_index(&index)?;
    Ok(index)
}

pub fn update_expert_stats(
    store: &DiskStore,
    expert_names: &[String],
    success: bool,
    energy_cost: f32,
    runtime_ms: f32,
) -> Result<ExpertStatsIndex> {
    let mut index = store.expert_stats_index()?;
    for name in expert_names {
        let stats = index
            .0
            .entry(name.clone())
            .or_insert_with(|| ExpertStats::new(name));
        stats.total_runs += 1;
        if success {
            stats.successes += 1;
        } else {
            stats.failures += 1;
        }
        let runs = stats.total_runs as f32;
        stats.average_energy_cost =
            ((stats.average_energy_cost * (runs - 1.0)) + energy_cost) / runs;
        stats.average_runtime_ms = ((stats.average_runtime_ms * (runs - 1.0)) + runtime_ms) / runs;
        let success_rate = stats.successes as f32 / stats.total_runs.max(1) as f32;
        let failure_penalty = stats.failures as f32 * 0.02;
        stats.confidence = (0.4 + success_rate * 0.6 - failure_penalty).clamp(0.0, 1.0);
    }
    store.save_expert_stats_index(&index)?;
    Ok(index)
}
