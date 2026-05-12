use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    core::Synapse,
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEfficiencyStats {
    pub route_id: String,
    pub from: String,
    pub to: String,
    pub times_used: u64,
    pub successes: u64,
    pub failures: u64,
    pub average_runtime_ms: f32,
    pub average_energy: f32,
    pub average_score: f32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub efficiency_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteEfficiencyIndex {
    pub routes: BTreeMap<String, RouteEfficiencyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RouteEfficiencyOverview {
    pub route_count: usize,
    pub average_efficiency: f32,
    pub top_routes: Vec<String>,
    pub least_efficient_routes: Vec<String>,
    pub high_failure_routes: Vec<String>,
}

pub fn route_efficiency_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("route_efficiency.json")
}

pub fn load_route_efficiency(store: &DiskStore) -> Result<RouteEfficiencyIndex> {
    let path = route_efficiency_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(RouteEfficiencyIndex::default())
    }
}

pub fn save_route_efficiency(store: &DiskStore, index: &RouteEfficiencyIndex) -> Result<()> {
    save_json(&route_efficiency_path(store), index)
}

pub fn update_route_efficiency_from_synapses(
    store: &DiskStore,
    synapses: &[Synapse],
    runtime_ms: u64,
    energy: f32,
    success: bool,
    final_score: f32,
) -> Result<usize> {
    let mut index = load_route_efficiency(store)?;
    let mut updated = 0;
    for synapse in synapses.iter().take(64) {
        update_one(
            &mut index,
            &synapse.id,
            &synapse.from,
            &synapse.to,
            runtime_ms,
            energy,
            success,
            final_score,
        );
        updated += 1;
    }
    save_route_efficiency(store, &index)?;
    Ok(updated)
}

pub fn update_named_route_efficiency(
    store: &DiskStore,
    route_id: &str,
    from: &str,
    to: &str,
    runtime_ms: u64,
    energy: f32,
    success: bool,
    final_score: f32,
) -> Result<()> {
    let mut index = load_route_efficiency(store)?;
    update_one(
        &mut index,
        route_id,
        from,
        to,
        runtime_ms,
        energy,
        success,
        final_score,
    );
    save_route_efficiency(store, &index)
}

pub fn route_efficiency_bonus(store: &DiskStore, route_id: &str) -> f32 {
    load_route_efficiency(store)
        .ok()
        .and_then(|index| index.routes.get(route_id).cloned())
        .map(|stats| (stats.efficiency_score - 0.5) * 0.08)
        .unwrap_or(0.0)
}

pub fn route_efficiency_overview(store: &DiskStore) -> Result<RouteEfficiencyOverview> {
    let index = load_route_efficiency(store)?;
    let mut routes = index.routes.values().cloned().collect::<Vec<_>>();
    let average_efficiency = if routes.is_empty() {
        0.0
    } else {
        routes
            .iter()
            .map(|route| route.efficiency_score)
            .sum::<f32>()
            / routes.len() as f32
    };
    routes.sort_by(|a, b| b.efficiency_score.total_cmp(&a.efficiency_score));
    let top_routes = routes.iter().take(10).map(format_route).collect::<Vec<_>>();
    let least_efficient_routes = routes
        .iter()
        .rev()
        .take(10)
        .map(format_route)
        .collect::<Vec<_>>();
    let high_failure_routes = routes
        .iter()
        .filter(|route| route.failures > route.successes)
        .take(10)
        .map(format_route)
        .collect::<Vec<_>>();
    Ok(RouteEfficiencyOverview {
        route_count: routes.len(),
        average_efficiency,
        top_routes,
        least_efficient_routes,
        high_failure_routes,
    })
}

fn update_one(
    index: &mut RouteEfficiencyIndex,
    route_id: &str,
    from: &str,
    to: &str,
    runtime_ms: u64,
    energy: f32,
    success: bool,
    final_score: f32,
) {
    let stats = index
        .routes
        .entry(route_id.to_string())
        .or_insert(RouteEfficiencyStats {
            route_id: route_id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            times_used: 0,
            successes: 0,
            failures: 0,
            average_runtime_ms: 0.0,
            average_energy: 0.0,
            average_score: 0.0,
            last_used_at: None,
            efficiency_score: 0.0,
        });
    stats.times_used += 1;
    stats.successes += u64::from(success);
    stats.failures += u64::from(!success);
    let n = stats.times_used as f32;
    stats.average_runtime_ms = rolling_average(stats.average_runtime_ms, runtime_ms as f32, n);
    stats.average_energy = rolling_average(stats.average_energy, energy, n);
    stats.average_score = rolling_average(stats.average_score, final_score, n);
    stats.last_used_at = Some(Utc::now());
    stats.efficiency_score = score(stats);
}

fn rolling_average(previous: f32, next: f32, n: f32) -> f32 {
    if n <= 1.0 {
        next
    } else {
        ((previous * (n - 1.0)) + next) / n
    }
}

fn score(stats: &RouteEfficiencyStats) -> f32 {
    let success_rate = stats.successes as f32 / stats.times_used.max(1) as f32;
    let normalized_low_energy = 1.0 / (1.0 + stats.average_energy);
    let normalized_low_runtime = 1.0 / (1.0 + stats.average_runtime_ms / 1000.0);
    (success_rate * 0.40
        + stats.average_score * 0.25
        + normalized_low_energy * 0.20
        + normalized_low_runtime * 0.15)
        .clamp(0.0, 1.0)
}

fn format_route(route: &RouteEfficiencyStats) -> String {
    format!(
        "{}: {} -> {} efficiency {:.2} success {}/{} energy {:.1} runtime {:.0}ms",
        route.route_id,
        route.from,
        route.to,
        route.efficiency_score,
        route.successes,
        route.times_used,
        route.average_energy,
        route.average_runtime_ms
    )
}
