use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::core::ids::{MemoryId, NeuronId, SynapseId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SynapseType {
    Excitatory,
    Inhibitory,
    MemoryPointer,
    ExpertPointer,
    ToolPointer,
    GoalPointer,
    Shortcut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    pub id: SynapseId,
    pub from: NeuronId,
    pub to: NeuronId,
    pub synapse_type: SynapseType,
    pub weight: f32,
    pub confidence: f32,
    pub success_score: f32,
    pub failure_score: f32,
    pub energy_cost: f32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub memory_ref: Option<MemoryId>,
    pub expert_ref: Option<String>,
    pub metadata: Map<String, Value>,
}

impl Synapse {
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        synapse_type: SynapseType,
        weight: f32,
    ) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            synapse_type,
            weight,
            confidence: 0.5,
            success_score: 0.0,
            failure_score: 0.0,
            energy_cost: 0.05,
            last_used_at: None,
            usage_count: 0,
            memory_ref: None,
            expert_ref: None,
            metadata: Map::new(),
        }
    }
}
