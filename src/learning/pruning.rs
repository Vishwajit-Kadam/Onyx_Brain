use crate::core::Synapse;

pub fn should_prune(synapse: &Synapse) -> bool {
    (synapse.usage_count == 0 && synapse.confidence < 0.1)
        || synapse.failure_score > synapse.success_score + 3.0
        || synapse.weight < -0.8
}
