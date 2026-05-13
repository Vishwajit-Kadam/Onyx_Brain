use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBudget {
    pub max_warnings: usize,
    pub max_auto_repaired_issues: usize,
    pub max_remaining_minor_issues: usize,
    pub zero_critical_required: bool,
}

impl Default for ErrorBudget {
    fn default() -> Self {
        Self {
            max_warnings: 5,
            max_auto_repaired_issues: 10,
            max_remaining_minor_issues: 3,
            zero_critical_required: true,
        }
    }
}

pub fn status_from_error_budget(
    budget: &ErrorBudget,
    warnings: usize,
    repaired: usize,
    remaining_minor: usize,
    critical: usize,
) -> String {
    if budget.zero_critical_required && critical > 0 {
        "SafetyStopped".to_string()
    } else if warnings > budget.max_warnings
        || repaired > budget.max_auto_repaired_issues
        || remaining_minor > budget.max_remaining_minor_issues
    {
        "CompletedWithWarnings".to_string()
    } else {
        "Completed".to_string()
    }
}
