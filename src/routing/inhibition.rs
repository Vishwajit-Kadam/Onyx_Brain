use crate::core::{Synapse, SynapseType};

pub fn inhibitory_amount(synapse: &Synapse) -> f32 {
    if synapse.synapse_type == SynapseType::Inhibitory {
        synapse.weight.abs() * synapse.confidence
    } else {
        0.0
    }
}
