use crate::{
    core::TaskType,
    experts::{Expert, ExpertContext, ExpertResult},
};

pub struct LanguageExpert;

impl Expert for LanguageExpert {
    fn name(&self) -> &'static str {
        "LanguageExpert"
    }

    fn can_handle(&self, task: &crate::core::Task) -> f32 {
        if task.task_type == TaskType::Chat {
            0.9
        } else {
            0.25
        }
    }

    fn estimate_cost(&self, _task: &crate::core::Task) -> f32 {
        0.2
    }

    fn run(&self, context: &ExpertContext) -> ExpertResult {
        ExpertResult {
            expert_name: self.name().to_string(),
            summary: format!("Prepared a concise response for: {}", context.task.input),
            suggested_actions: Vec::new(),
            success: true,
            estimated_cost: 0.1,
            confidence_score: 0.8,
        }
    }
}
