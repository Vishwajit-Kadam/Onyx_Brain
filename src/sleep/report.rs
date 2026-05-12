use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub logs_seen: usize,
    pub strengthened_routes: usize,
    pub pruned_synapses: usize,
    pub shortcut_synapses_created: usize,
    pub report_path: String,
}
