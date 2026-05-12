use crate::{core::ActiveNeuron, energy::EnergyBudget};

pub fn enforce_sparse_budget(
    mut neurons: Vec<ActiveNeuron>,
    budget: &EnergyBudget,
) -> Vec<ActiveNeuron> {
    neurons.sort_by(|a, b| b.activation.total_cmp(&a.activation));
    neurons.truncate(budget.max_active_neurons);
    neurons
}
