//! Hebbian learning — "neurons that fire together wire together."
//!
//! Implements the core Hebbian learning rule for synapse strengthening and
//! weakening based on correlated activation. Provides weight update functions,
//! normalization, and decay for maintaining stable network dynamics.

use serde::{Deserialize, Serialize};

/// Maximum number of new synapses that can be created per task.
pub const MAX_NEW_SYNAPSES_PER_TASK: usize = 4;

/// Default learning rate for Hebbian updates.
pub const DEFAULT_LEARNING_RATE: f32 = 0.05;

/// Minimum synapse weight before pruning.
pub const MIN_WEIGHT: f32 = 0.01;

/// Maximum synapse weight to prevent runaway strengthening.
pub const MAX_WEIGHT: f32 = 1.0;

/// Weight decay factor applied each cycle to prevent unbounded growth.
pub const DECAY_FACTOR: f32 = 0.995;

/// A Hebbian weight update result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HebbianUpdate {
    pub pre_neuron: String,
    pub post_neuron: String,
    pub old_weight: f32,
    pub new_weight: f32,
    pub delta: f32,
    pub pruned: bool,
}

/// Compute the Hebbian weight update for a single synapse.
///
/// The standard Hebbian rule: Δw = η × pre × post
/// where η is the learning rate, pre is the presynaptic activation,
/// and post is the postsynaptic activation.
pub fn hebbian_update(
    current_weight: f32,
    pre_activation: f32,
    post_activation: f32,
    learning_rate: f32,
) -> HebbianUpdate {
    let pre = pre_activation.clamp(0.0, 1.0);
    let post = post_activation.clamp(0.0, 1.0);
    let delta = learning_rate * pre * post;
    let new_weight = (current_weight + delta).clamp(MIN_WEIGHT, MAX_WEIGHT);
    let pruned = new_weight <= MIN_WEIGHT && current_weight > MIN_WEIGHT;
    HebbianUpdate {
        pre_neuron: String::new(),
        post_neuron: String::new(),
        old_weight: current_weight,
        new_weight,
        delta,
        pruned,
    }
}

/// Anti-Hebbian update: weaken a synapse when neurons fire out of sync.
///
/// Used when presynaptic neuron fires but postsynaptic does not.
pub fn anti_hebbian_update(
    current_weight: f32,
    pre_activation: f32,
    learning_rate: f32,
) -> HebbianUpdate {
    let delta = -(learning_rate * pre_activation.clamp(0.0, 1.0) * 0.5);
    let new_weight = (current_weight + delta).clamp(MIN_WEIGHT, MAX_WEIGHT);
    let pruned = new_weight <= MIN_WEIGHT;
    HebbianUpdate {
        pre_neuron: String::new(),
        post_neuron: String::new(),
        old_weight: current_weight,
        new_weight,
        delta,
        pruned,
    }
}

/// Apply weight decay to all weights in a batch, returning updated weights
/// and identifying which synapses should be pruned.
pub fn apply_decay(weights: &[f32]) -> Vec<(f32, bool)> {
    weights
        .iter()
        .map(|w| {
            let decayed = w * DECAY_FACTOR;
            let pruned = decayed < MIN_WEIGHT;
            (if pruned { 0.0 } else { decayed }, pruned)
        })
        .collect()
}

/// Normalize a set of weights so they sum to 1.0, preserving relative magnitudes.
pub fn normalize_weights(weights: &mut [f32]) {
    let sum: f32 = weights.iter().sum();
    if sum > 0.0 {
        for w in weights.iter_mut() {
            *w /= sum;
        }
    }
}

/// Compute the number of new synapses to create for a given activation pattern.
/// Bounded by MAX_NEW_SYNAPSES_PER_TASK.
pub fn new_synapses_to_create(
    active_neurons: usize,
    existing_synapses: usize,
    desired_connectivity: f32,
) -> usize {
    let desired = (active_neurons as f32 * desired_connectivity) as usize;
    let deficit = desired.saturating_sub(existing_synapses);
    deficit.min(MAX_NEW_SYNAPSES_PER_TASK)
}

/// Summarize a batch of Hebbian updates.
pub fn summarize_updates(updates: &[HebbianUpdate]) -> String {
    let strengthened = updates.iter().filter(|u| u.delta > 0.0).count();
    let weakened = updates.iter().filter(|u| u.delta < 0.0).count();
    let pruned = updates.iter().filter(|u| u.pruned).count();
    format!(
        "Hebbian updates: {} strengthened, {} weakened, {} pruned (total: {})",
        strengthened,
        weakened,
        pruned,
        updates.len()
    )
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hebbian_update_strengthens_correlated_neurons() {
        let update = hebbian_update(0.5, 0.8, 0.9, DEFAULT_LEARNING_RATE);
        assert!(update.new_weight > 0.5);
        assert!(update.delta > 0.0);
    }

    #[test]
    fn hebbian_update_clamps_weight() {
        let update = hebbian_update(0.99, 1.0, 1.0, 0.5);
        assert!(update.new_weight <= MAX_WEIGHT);
    }

    #[test]
    fn anti_hebbian_weakens() {
        let update = anti_hebbian_update(0.5, 0.8, DEFAULT_LEARNING_RATE);
        assert!(update.new_weight < 0.5);
        assert!(update.delta < 0.0);
    }

    #[test]
    fn weight_decay_reduces_values() {
        let result = apply_decay(&[0.5, 0.3, 0.1]);
        assert!(result[0].0 < 0.5);
        assert!(result[1].0 < 0.3);
    }

    #[test]
    fn decay_prunes_tiny_weights() {
        let result = apply_decay(&[0.005]);
        assert!(result[0].1); // pruned
    }

    #[test]
    fn normalize_preserves_ratios() {
        let mut weights = vec![2.0, 3.0, 5.0];
        normalize_weights(&mut weights);
        assert!((weights.iter().sum::<f32>() - 1.0).abs() < 0.001);
        assert!(weights[2] > weights[1]);
    }

    #[test]
    fn new_synapses_bounded() {
        let n = new_synapses_to_create(100, 0, 0.5);
        assert!(n <= MAX_NEW_SYNAPSES_PER_TASK);
    }

    #[test]
    fn summarize_is_readable() {
        let updates = vec![
            hebbian_update(0.5, 0.8, 0.9, DEFAULT_LEARNING_RATE),
            anti_hebbian_update(0.5, 0.8, DEFAULT_LEARNING_RATE),
        ];
        let summary = summarize_updates(&updates);
        assert!(summary.contains("1 strengthened"));
        assert!(summary.contains("1 weakened"));
    }
}
