use crate::{
    core::TaskType,
    experts::{Expert, ExpertContext, ExpertResult},
};

pub struct ToolUseExpert;

impl Expert for ToolUseExpert {
    fn name(&self) -> &'static str {
        "ToolUseExpert"
    }

    fn can_handle(&self, task: &crate::core::Task) -> f32 {
        if matches!(
            task.task_type,
            TaskType::Code | TaskType::FileOperation | TaskType::ToolUse
        ) {
            0.7
        } else {
            0.05
        }
    }

    fn estimate_cost(&self, _task: &crate::core::Task) -> f32 {
        0.25
    }

    fn run(&self, _context: &ExpertContext) -> ExpertResult {
        ExpertResult {
            expert_name: self.name().to_string(),
            summary: "Selected sandbox-only tools.".to_string(),
            suggested_actions: vec!["use_safe_filesystem".to_string()],
            success: true,
            estimated_cost: 0.1,
            confidence_score: 0.8,
        }
    }
}
