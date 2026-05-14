//! Reasoning Expert for logical breakdown and step-by-step planning.
//!
//! Evaluates tasks that require multi-step execution, logical decomposition,
//! or dependency resolution. It translates an abstract input into a concrete
//! list of discrete, actionable steps.

use crate::{
    core::{Task, TaskType},
    experts::{Expert, ExpertContext, ExpertResult},
};

pub struct ReasoningExpert;

impl ReasoningExpert {
    /// Applies heuristics to break down a prompt into logical steps.
    fn decompose_logic(input: &str) -> Vec<String> {
        let mut steps = Vec::new();
        let lower = input.to_lowercase();

        // Initial setup step
        if lower.contains("create") || lower.contains("build") || lower.contains("new") {
            steps.push("Initialize workspace and empty targets".to_string());
        } else if lower.contains("modify") || lower.contains("update") || lower.contains("refactor")
        {
            steps.push("Analyze existing codebase structure".to_string());
        }

        // Core implementation steps
        if lower.contains("function") || lower.contains("method") {
            steps.push("Define function signatures and types".to_string());
            steps.push("Implement core function logic".to_string());
        }

        // Validation steps
        if lower.contains("test") || lower.contains("verify") {
            steps.push("Write unit tests for new behavior".to_string());
            steps.push("Run test suite and verify coverage".to_string());
        } else {
            // Default sanity check
            steps.push("Compile and check for syntax errors".to_string());
        }

        // Fallback for simple prompts
        if steps.is_empty() {
            steps.push("Execute requested operation".to_string());
            steps.push("Verify outcome".to_string());
        }

        steps
    }

    /// Evaluates if the task has logical dependencies.
    fn has_dependencies(input: &str) -> bool {
        let lower = input.to_lowercase();
        lower.contains("after") || lower.contains("then") || lower.contains("depends")
    }
}

impl Expert for ReasoningExpert {
    fn name(&self) -> &'static str {
        "ReasoningExpert"
    }

    fn can_handle(&self, task: &Task) -> f32 {
        let mut score: f32 = 0.2; // Base baseline

        if matches!(task.task_type, TaskType::Planning | TaskType::Reasoning) {
            score += 0.6;
        } else if matches!(task.task_type, TaskType::Code) {
            score += 0.3; // Code often requires reasoning
        }

        let word_count = task.input.split_whitespace().count();
        if word_count > 10 {
            score += 0.1; // Longer prompts usually require more reasoning
        }

        if Self::has_dependencies(&task.input) {
            score += 0.2;
        }

        score.clamp(0.0, 1.0)
    }

    fn estimate_cost(&self, task: &Task) -> f32 {
        // Reasoning cost scales with the length and complexity of the prompt
        let base = 0.3;
        let complexity_bonus = if Self::has_dependencies(&task.input) {
            0.2
        } else {
            0.0
        };
        base + complexity_bonus
    }

    fn run(&self, context: &ExpertContext) -> ExpertResult {
        let steps = Self::decompose_logic(&context.task.input);

        let mut summary = format!("Decomposed task into {} logical steps.", steps.len());

        if Self::has_dependencies(&context.task.input) {
            summary.push_str(" Identified sequential dependencies.");
        }

        if !context.memories.is_empty() {
            summary.push_str(&format!(
                " Referenced {} prior memory contexts.",
                context.memories.len()
            ));
        }

        // Convert string steps to action identifiers
        let mut actions = Vec::new();
        for step in steps {
            if step.contains("test") {
                actions.push("run_tests".to_string());
            } else if step.contains("compile") || step.contains("check") {
                actions.push("run_check".to_string());
            } else if step.contains("Initialize") {
                actions.push("setup_workspace".to_string());
            } else {
                actions.push("edit_files".to_string());
            }
        }

        // Deduplicate actions sequentially
        actions.dedup();

        let confidence = if context.task.input.split_whitespace().count() > 5 {
            0.85
        } else {
            0.6
        };

        ExpertResult {
            expert_name: self.name().to_string(),
            summary,
            suggested_actions: actions,
            success: true,
            estimated_cost: self.estimate_cost(&context.task),
            confidence_score: confidence,
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expert_name_is_reasoning_expert() {
        assert_eq!(ReasoningExpert.name(), "ReasoningExpert");
    }

    #[test]
    fn can_handle_scores_planning_higher() {
        let expert = ReasoningExpert;

        let plan_task = Task::new("plan the architecture".into(), TaskType::Planning);
        assert!(expert.can_handle(&plan_task) > 0.7);

        let chat_task = Task::new("hello".into(), TaskType::Chat);
        assert!(expert.can_handle(&chat_task) < 0.3);
    }

    #[test]
    fn decompose_logic_extracts_correct_steps() {
        let steps = ReasoningExpert::decompose_logic("create a function and test it");
        assert!(steps.iter().any(|s| s.contains("Initialize")));
        assert!(steps.iter().any(|s| s.contains("function signatures")));
        assert!(steps.iter().any(|s| s.contains("unit tests")));
    }

    #[test]
    fn run_generates_action_plan() {
        let expert = ReasoningExpert;
        let task = Task::new(
            "create a calculator function and then test it thoroughly".into(),
            TaskType::Reasoning,
        );

        let ctx = ExpertContext {
            task,
            memories: vec![],
            active_neurons: vec![],
        };

        let result = expert.run(&ctx);
        assert!(result.success);
        assert!(result.summary.contains("Decomposed"));
        assert!(result.summary.contains("sequential dependencies")); // because of "then"

        assert!(result
            .suggested_actions
            .contains(&"setup_workspace".to_string()));
        assert!(result.suggested_actions.contains(&"edit_files".to_string()));
        assert!(result.suggested_actions.contains(&"run_tests".to_string()));
        assert!(result.confidence_score > 0.8);
    }
}
