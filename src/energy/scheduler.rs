//! Energy scheduler — decides whether tools can run based on the current
//! energy budget, priority weighting, and action tracking.
//!
//! Extends the simple boolean check with priority-aware scheduling,
//! budget tracking, and decision reporting.

use crate::energy::EnergyBudget;
use serde::{Deserialize, Serialize};

/// A scheduling decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    pub allowed: bool,
    pub reason: String,
    pub remaining_actions: usize,
    pub budget_utilization: f32,
}

/// Priority level for an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl ActionPriority {
    /// High-priority actions are allowed even when budget is tight.
    fn budget_threshold(&self) -> f32 {
        match self {
            ActionPriority::Critical => 0.95, // almost always allowed
            ActionPriority::High => 0.85,
            ActionPriority::Normal => 0.75,
            ActionPriority::Low => 0.5,
        }
    }
}

/// Check if a tool action can run (backward-compatible simple check).
pub fn can_run_tool(current_actions: usize, budget: &EnergyBudget) -> bool {
    current_actions < budget.max_tool_actions
}

/// Make a priority-aware scheduling decision.
pub fn schedule_action(
    current_actions: usize,
    budget: &EnergyBudget,
    priority: ActionPriority,
) -> SchedulingDecision {
    let max = budget.max_tool_actions;
    if max == 0 {
        return SchedulingDecision {
            allowed: false,
            reason: "Budget is zero — no actions allowed".into(),
            remaining_actions: 0,
            budget_utilization: 1.0,
        };
    }

    let utilization = current_actions as f32 / max as f32;
    let remaining = max.saturating_sub(current_actions);

    if current_actions >= max {
        // Budget exhausted — only critical actions pass
        if priority == ActionPriority::Critical && current_actions < max + 2 {
            return SchedulingDecision {
                allowed: true,
                reason: "Critical action override — exceeding budget by emergency allowance".into(),
                remaining_actions: 0,
                budget_utilization: utilization,
            };
        }
        return SchedulingDecision {
            allowed: false,
            reason: format!(
                "Budget exhausted: {}/{} actions used, priority {:?} insufficient",
                current_actions, max, priority
            ),
            remaining_actions: 0,
            budget_utilization: utilization,
        };
    }

    let threshold = priority.budget_threshold();
    if utilization < threshold {
        SchedulingDecision {
            allowed: true,
            reason: format!(
                "Allowed: {}/{} used ({:.0}%), priority {:?}",
                current_actions,
                max,
                utilization * 100.0,
                priority
            ),
            remaining_actions: remaining,
            budget_utilization: utilization,
        }
    } else {
        SchedulingDecision {
            allowed: false,
            reason: format!(
                "Budget too tight for {:?} priority: {:.0}% utilized (threshold: {:.0}%)",
                priority,
                utilization * 100.0,
                threshold * 100.0
            ),
            remaining_actions: remaining,
            budget_utilization: utilization,
        }
    }
}

/// Report on current budget status.
pub fn budget_status_report(current_actions: usize, budget: &EnergyBudget) -> String {
    let max = budget.max_tool_actions;
    let remaining = max.saturating_sub(current_actions);
    let utilization = if max > 0 {
        current_actions as f32 / max as f32
    } else {
        1.0
    };
    format!(
        "Energy budget: {}/{} actions used ({:.0}% utilized), {} remaining",
        current_actions,
        max,
        utilization * 100.0,
        remaining
    )
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy::EnergyBudget;

    fn test_budget(max: usize) -> EnergyBudget {
        EnergyBudget {
            max_tool_actions: max,
            ..Default::default()
        }
    }

    #[test]
    fn can_run_tool_basic() {
        let budget = test_budget(10);
        assert!(can_run_tool(5, &budget));
        assert!(!can_run_tool(10, &budget));
    }

    #[test]
    fn schedule_normal_within_budget() {
        let decision = schedule_action(3, &test_budget(10), ActionPriority::Normal);
        assert!(decision.allowed);
        assert_eq!(decision.remaining_actions, 7);
    }

    #[test]
    fn schedule_low_rejected_when_tight() {
        let decision = schedule_action(6, &test_budget(10), ActionPriority::Low);
        // 60% utilization > 50% threshold for Low priority
        assert!(!decision.allowed);
    }

    #[test]
    fn critical_overrides_exhausted_budget() {
        let decision = schedule_action(10, &test_budget(10), ActionPriority::Critical);
        assert!(decision.allowed);
    }

    #[test]
    fn zero_budget_denies_everything() {
        let decision = schedule_action(0, &test_budget(0), ActionPriority::Critical);
        assert!(!decision.allowed);
    }

    #[test]
    fn budget_report_is_readable() {
        let report = budget_status_report(3, &test_budget(10));
        assert!(report.contains("3/10"));
        assert!(report.contains("7 remaining"));
    }
}
