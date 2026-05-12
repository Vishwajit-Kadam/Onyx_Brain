use crate::energy::EnergyBudget;

pub fn can_run_tool(current_actions: usize, budget: &EnergyBudget) -> bool {
    current_actions < budget.max_tool_actions
}
