//! Core abstractions for edges (connections) in the cognitive graph.
//!
//! A `Synapse` connects two `VirtualNeuron`s. It stores the connection weight,
//! learning scores (success vs failure tracking), and energy cost. Synapses
//! are updated via Hebbian-like mechanisms during or after cognitive cycles.

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

/// A weighted edge between two conceptual nodes.
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
    /// Create a new synapse with default plasticity parameters.
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
            weight: weight.clamp(0.0, 1.0),
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

    /// Mark the synapse as used during a traversal, increasing its count and usage timestamp.
    pub fn mark_used(&mut self) {
        self.usage_count = self.usage_count.saturating_add(1);
        self.last_used_at = Some(Utc::now());
    }

    /// Adjust the synapse weight and success score based on a positive outcome.
    /// Simulates Hebbian reinforcement (fire together, wire together).
    pub fn reinforce(&mut self, reward_magnitude: f32) {
        self.success_score += reward_magnitude;
        self.weight = (self.weight + (reward_magnitude * 0.1)).min(1.0);
        self.confidence = (self.confidence + 0.05).min(1.0);
    }

    /// Adjust the synapse weight and failure score based on a negative outcome.
    /// Simulates Anti-Hebbian decay (punishment).
    pub fn penalize(&mut self, penalty_magnitude: f32) {
        self.failure_score += penalty_magnitude;
        self.weight = (self.weight - (penalty_magnitude * 0.1)).max(0.0);
        self.confidence = (self.confidence - 0.02).max(0.1);
    }

    /// Calculate the effective signal strength transmitted across this synapse.
    /// Inhibitory synapses return negative strengths.
    pub fn transmit_signal(&self, source_activation: f32) -> f32 {
        let raw_signal = source_activation * self.weight;
        match self.synapse_type {
            SynapseType::Inhibitory => -raw_signal,
            _ => raw_signal,
        }
    }

    /// Decay the synapse weight naturally over time if not used.
    pub fn apply_decay(&mut self, decay_factor: f32) {
        // Only decay if weight is above a minimum floor
        if self.weight > 0.01 {
            self.weight *= decay_factor;
            self.confidence *= decay_factor.sqrt(); // Confidence decays slower than weight
        }
    }

    /// Estimate the energy cost of traversing this synapse based on weight and type.
    pub fn dynamic_energy_cost(&self) -> f32 {
        // Higher weight implies wider channel (less resistance), so lower base energy.
        // Shortcuts are highly optimized.
        let base = self.energy_cost;
        let efficiency = self.weight * 0.02;
        let cost = if self.synapse_type == SynapseType::Shortcut {
            base * 0.5
        } else {
            base - efficiency
        };
        cost.max(0.01)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_synapse_clamps_weight() {
        let s = Synapse::new("s1", "n1", "n2", SynapseType::Excitatory, 1.5);
        assert_eq!(s.weight, 1.0);
        let s2 = Synapse::new("s2", "n1", "n2", SynapseType::Excitatory, -0.5);
        assert_eq!(s2.weight, 0.0);
    }

    #[test]
    fn mark_used_updates_metrics() {
        let mut s = Synapse::new("s1", "n1", "n2", SynapseType::Excitatory, 0.5);
        s.mark_used();
        assert_eq!(s.usage_count, 1);
        assert!(s.last_used_at.is_some());
    }

    #[test]
    fn reinforce_increases_weight_and_confidence() {
        let mut s = Synapse::new("s1", "n1", "n2", SynapseType::Excitatory, 0.5);
        s.confidence = 0.5;
        s.reinforce(1.0);
        assert_eq!(s.success_score, 1.0);
        assert!(s.weight > 0.5);
        assert!(s.confidence > 0.5);
    }

    #[test]
    fn penalize_decreases_weight_and_confidence() {
        let mut s = Synapse::new("s1", "n1", "n2", SynapseType::Excitatory, 0.5);
        s.confidence = 0.5;
        s.penalize(1.0);
        assert_eq!(s.failure_score, 1.0);
        assert!(s.weight < 0.5);
        assert!(s.confidence < 0.5);
    }

    #[test]
    fn transmit_signal_is_negative_for_inhibitory() {
        let s_ex = Synapse::new("s1", "n1", "n2", SynapseType::Excitatory, 0.8);
        assert!(s_ex.transmit_signal(1.0) > 0.0);

        let s_in = Synapse::new("s2", "n1", "n2", SynapseType::Inhibitory, 0.8);
        assert!(s_in.transmit_signal(1.0) < 0.0);
    }

    #[test]
    fn apply_decay_reduces_weight_gradually() {
        let mut s = Synapse::new("s1", "n1", "n2", SynapseType::Excitatory, 0.8);
        s.confidence = 0.8;
        s.apply_decay(0.9);
        assert!((s.weight - 0.72).abs() < f32::EPSILON);
        assert!(s.confidence > 0.72); // Confidence decays slower (sqrt(0.9) approx 0.948)
    }

    #[test]
    fn dynamic_energy_cost_is_lower_for_shortcuts() {
        let s_norm = Synapse::new("s1", "n1", "n2", SynapseType::Excitatory, 0.5);
        let s_short = Synapse::new("s2", "n1", "n2", SynapseType::Shortcut, 0.5);
        assert!(s_short.dynamic_energy_cost() < s_norm.dynamic_energy_cost());
    }
}
