//! Core abstractions for nodes in the cognitive graph.
//!
//! A `VirtualNeuron` represents a discrete semantic node, which could be
//! a skill, a memory, a tool, or an expert. It maintains base activation
//! levels, dynamic firing thresholds, and usage metrics for hebbian learning.

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

/// A node in the semantic/cognitive graph.
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
    /// Create a new neuron with sensible default thresholds.
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

    /// Update the activation metrics, bumping the usage counter and timestamp.
    pub fn mark_activated(&mut self) {
        self.activation_count = self.activation_count.saturating_add(1);
        self.last_activated_at = Some(Utc::now());
        // Temporarily boost base activation (plasticity)
        self.base_activation = (self.base_activation + 0.05).min(0.9);
    }

    /// Simulate natural decay of base activation over time to prune unused neurons.
    pub fn apply_decay(&mut self, decay_factor: f32) {
        self.base_activation *= decay_factor;
        if self.base_activation < 0.01 {
            self.base_activation = 0.01;
        }
    }

    /// Compute the dynamic threshold based on usage history.
    /// Frequently used neurons become easier to activate (lower threshold).
    pub fn dynamic_threshold(&self) -> f32 {
        let usage_bonus = (self.activation_count as f32 * 0.01).min(0.2);
        (self.threshold - usage_bonus).max(0.1)
    }

    /// Determine if an input signal is strong enough to trigger this neuron.
    pub fn is_triggered(&self, input_signal: f32) -> bool {
        (self.base_activation + input_signal) >= self.dynamic_threshold()
    }

    /// Merges new metadata fields into the neuron, keeping existing ones.
    pub fn merge_metadata(&mut self, new_metadata: Map<String, Value>) {
        for (k, v) in new_metadata {
            self.metadata.insert(k, v);
        }
    }
}

/// Represents the transient state of a neuron that has "fired" during a single cognitive cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveNeuron {
    pub id: NeuronId,
    pub activation: f32,
    pub threshold: f32,
    pub reasons: Vec<String>,
    pub loaded_synapse_count: usize,
    pub estimated_energy_cost: f32,
}

impl ActiveNeuron {
    /// Build an active neuron representation, calculating energy cost based on activation intensity.
    pub fn new(id: NeuronId, activation: f32, threshold: f32, synapse_count: usize) -> Self {
        // Simple heuristic: higher activation + more downstream connections = more "energy"
        let base_cost = 0.5;
        let intensity_cost = (activation - threshold).max(0.0) * 2.0;
        let connectivity_cost = synapse_count as f32 * 0.1;

        Self {
            id,
            activation,
            threshold,
            reasons: Vec::new(),
            loaded_synapse_count: synapse_count,
            estimated_energy_cost: base_cost + intensity_cost + connectivity_cost,
        }
    }

    pub fn add_reason(&mut self, reason: impl Into<String>) {
        self.reasons.push(reason.into());
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_neuron_has_sensible_defaults() {
        let n = VirtualNeuron::new("n1", "Test Neuron", NeuronKind::Concept);
        assert_eq!(n.id, "n1");
        assert_eq!(n.activation_count, 0);
        assert!(n.last_activated_at.is_none());
    }

    #[test]
    fn mark_activated_increases_metrics() {
        let mut n = VirtualNeuron::new("n1", "Test Neuron", NeuronKind::Skill);
        let initial_base = n.base_activation;
        n.mark_activated();
        assert_eq!(n.activation_count, 1);
        assert!(n.last_activated_at.is_some());
        assert!(n.base_activation > initial_base);
    }

    #[test]
    fn decay_reduces_base_activation() {
        let mut n = VirtualNeuron::new("n1", "Test", NeuronKind::Memory);
        n.base_activation = 0.8;
        n.apply_decay(0.5);
        assert!((n.base_activation - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn decay_does_not_go_below_minimum() {
        let mut n = VirtualNeuron::new("n1", "Test", NeuronKind::Memory);
        n.base_activation = 0.015;
        n.apply_decay(0.1);
        assert_eq!(n.base_activation, 0.01);
    }

    #[test]
    fn dynamic_threshold_lowers_with_usage() {
        let mut n = VirtualNeuron::new("n1", "Test", NeuronKind::Tool);
        let t1 = n.dynamic_threshold();
        n.activation_count = 10;
        let t2 = n.dynamic_threshold();
        assert!(t2 < t1);
    }

    #[test]
    fn is_triggered_respects_dynamic_threshold() {
        let n = VirtualNeuron::new("n1", "Test", NeuronKind::Tool);
        // threshold is 0.5, base_activation is 0.1
        // Need input >= 0.4
        assert!(!n.is_triggered(0.3));
        assert!(n.is_triggered(0.4));
    }

    #[test]
    fn merge_metadata_adds_new_keys() {
        let mut n = VirtualNeuron::new("n1", "Test", NeuronKind::Goal);
        n.metadata.insert("a".into(), Value::Bool(true));
        let mut extra = Map::new();
        extra.insert("b".into(), Value::String("hello".into()));
        extra.insert("a".into(), Value::Bool(false)); // Should overwrite

        n.merge_metadata(extra);
        assert_eq!(n.metadata.len(), 2);
        assert_eq!(n.metadata.get("a"), Some(&Value::Bool(false)));
        assert_eq!(n.metadata.get("b"), Some(&Value::String("hello".into())));
    }

    #[test]
    fn active_neuron_calculates_energy() {
        let mut active = ActiveNeuron::new("n1".into(), 0.8, 0.5, 5);
        assert!(active.estimated_energy_cost > 0.5);
        active.add_reason("Fired due to context match.");
        assert_eq!(active.reasons.len(), 1);
    }
}
