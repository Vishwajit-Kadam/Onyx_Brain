use crate::{
    core::TaskType,
    experts::{Expert, ExpertContext, ExpertResult},
};

pub struct ReasoningExpert;

impl Expert for ReasoningExpert {
    fn name(&self) -> &'static str {
        "ReasoningExpert"
    }

    fn can_handle(&self, task: &crate::core::Task) -> f32 {
        if matches!(
            task.task_type,
            TaskType::Planning | TaskType::Code | TaskType::Reasoning
        ) {
            0.8
        } else {
            0.2
        }
    }

    fn estimate_cost(&self, _task: &crate::core::Task) -> f32 {
        0.3
    }

    fn run(&self, context: &ExpertContext) -> ExpertResult {
        ExpertResult {
            expert_name: self.name().to_string(),
            summary: format!(
                "Steps selected from {} active neurons.",
                context.active_neurons.len()
            ),
            suggested_actions: vec![
                "create project directory".to_string(),
                "write Rust files".to_string(),
                "run safe check".to_string(),
            ],
            success: true,
            estimated_cost: 0.3,
        }
    }
}
