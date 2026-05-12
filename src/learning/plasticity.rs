use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    core::{Synapse, SynapseType},
    storage::DiskStore,
    utils::time::now,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningReport {
    pub strengthened: usize,
    pub weakened: usize,
    pub new_synapses: usize,
}

pub fn update_routes(
    store: &DiskStore,
    synapses: &[Synapse],
    active_neuron_ids: &[String],
    success: bool,
) -> Result<LearningReport> {
    let mut strengthened = 0;
    let mut weakened = 0;
    for synapse in synapses {
        let mut updated = synapse.clone();
        updated.last_used_at = Some(now());
        updated.usage_count += 1;
        if success {
            updated.success_score += 1.0;
            updated.weight = (updated.weight + 0.1).clamp(-1.0, 1.0);
            updated.confidence = (updated.confidence + 0.01).clamp(0.0, 1.0);
            strengthened += 1;
        } else {
            updated.failure_score += 1.0;
            updated.weight = (updated.weight - 0.1).clamp(-1.0, 1.0);
            updated.confidence = (updated.confidence - 0.01).clamp(0.0, 1.0);
            weakened += 1;
        }
        store.save_synapse(&updated)?;
    }
    let new_synapses = if success {
        hebbian_shortcuts(store, active_neuron_ids, 4)?
    } else {
        0
    };
    Ok(LearningReport {
        strengthened,
        weakened,
        new_synapses,
    })
}

fn hebbian_shortcuts(
    store: &DiskStore,
    active_neuron_ids: &[String],
    limit: usize,
) -> Result<usize> {
    let mut created = 0;
    for pair in active_neuron_ids.windows(2) {
        if created >= limit {
            break;
        }
        let from = &pair[0];
        let to = &pair[1];
        if from == to {
            continue;
        }
        let id = format!("shortcut_{from}_{to}");
        if store.synapse_path(&id).exists() {
            continue;
        }
        let mut synapse = Synapse::new(&id, from, to, SynapseType::Shortcut, 0.2);
        synapse.confidence = 0.2;
        synapse.success_score = 1.0;
        store.save_synapse(&synapse)?;
        let mut outgoing = store.read_outgoing_synapse_ids(from)?;
        outgoing.push(id);
        outgoing.sort();
        outgoing.dedup();
        store.write_outgoing_synapse_ids(from, &outgoing)?;
        created += 1;
    }
    Ok(created)
}
