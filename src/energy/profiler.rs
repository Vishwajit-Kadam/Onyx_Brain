use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnergyReport {
    pub active_neuron_count: usize,
    pub loaded_synapse_count: usize,
    pub memory_items_loaded: usize,
    pub expert_count: usize,
    pub tool_action_count: usize,
    pub runtime_ms: u128,
    pub estimated_cost_units: u64,
}

pub struct Profiler {
    started_at: Instant,
}

impl Profiler {
    pub fn start() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub fn finish(
        self,
        active_neuron_count: usize,
        loaded_synapse_count: usize,
        memory_items_loaded: usize,
        expert_count: usize,
        tool_action_count: usize,
    ) -> EnergyReport {
        let runtime_ms = self.started_at.elapsed().as_millis();
        let estimated_cost_units = active_neuron_count as u64
            + loaded_synapse_count as u64 * 2
            + memory_items_loaded as u64 * 3
            + expert_count as u64 * 5
            + tool_action_count as u64 * 8
            + runtime_ms as u64 / 10;
        EnergyReport {
            active_neuron_count,
            loaded_synapse_count,
            memory_items_loaded,
            expert_count,
            tool_action_count,
            runtime_ms,
            estimated_cost_units,
        }
    }
}
