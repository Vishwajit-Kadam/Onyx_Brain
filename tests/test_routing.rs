use onyx_brain::{
    core::ActiveNeuron,
    energy::EnergyBudget,
    routing::{activation_budget::enforce_sparse_budget, inhibition::inhibitory_amount},
};

#[test]
fn inhibitory_synapse_reduces_activation() {
    let mut synapse = onyx_brain::core::Synapse::new(
        "s1",
        "a",
        "b",
        onyx_brain::core::SynapseType::Inhibitory,
        -0.7,
    );
    synapse.confidence = 0.5;
    assert!((inhibitory_amount(&synapse) - 0.35).abs() < f32::EPSILON);
}

#[test]
fn router_never_exceeds_sparse_budget() {
    let neurons = (0..10)
        .map(|idx| ActiveNeuron {
            id: format!("n{idx}"),
            activation: idx as f32,
            threshold: 0.1,
            reasons: Vec::new(),
            loaded_synapse_count: 0,
            estimated_energy_cost: 0.0,
        })
        .collect();
    let budget = EnergyBudget {
        max_active_neurons: 3,
        ..EnergyBudget::default()
    };
    let active = enforce_sparse_budget(neurons, &budget);
    assert_eq!(active.len(), 3);
    assert_eq!(active[0].id, "n9");
}
