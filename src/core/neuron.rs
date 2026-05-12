use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::core::ids::NeuronId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NeuronKind {
    Concept,
    Skill,
    Expert,
    Memory,
    Tool,
    Goal,
    TaskType,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualNeuron {
    pub id: NeuronId,
    pub label: String,
    pub kind: NeuronKind,
    pub threshold: f32,
    pub base_activation: f32,
    pub last_activated_at: Option<DateTime<Utc>>,
    pub activation_count: u64,
    pub metadata: Map<String, Value>,
}

impl VirtualNeuron {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: NeuronKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            threshold: 0.5,
            base_activation: 0.1,
            last_activated_at: None,
            activation_count: 0,
            metadata: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveNeuron {
    pub id: NeuronId,
    pub activation: f32,
    pub threshold: f32,
    pub reasons: Vec<String>,
    pub loaded_synapse_count: usize,
    pub estimated_energy_cost: f32,
}
