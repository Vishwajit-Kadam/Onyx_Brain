use onyx_brain::{
    core::TaskType,
    energy::{EnergyBudgetManager, Profiler},
};

#[test]
fn code_tasks_get_miniature_default_budget() {
    let budget = EnergyBudgetManager::budget_for(&TaskType::Code);
    assert_eq!(budget.max_active_neurons, 32);
    assert_eq!(budget.max_active_synapses_per_neuron, 16);
    assert_eq!(budget.max_experts, 2);
}

#[test]
fn profiler_reports_estimated_cost() {
    let report = Profiler::start().finish(2, 3, 1, 1, 1);
    assert!(report.estimated_cost_units >= 24);
}
