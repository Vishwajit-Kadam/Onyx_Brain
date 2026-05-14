//! Base abstractions and dispatch logic for Specialized Experts.
//!
//! The Onyx Brain relies on a mixture of experts (MoE) pattern. Instead of one monolithic
//! process, specific cognitive domains (e.g., Code, Language, Reasoning, ToolUse) are
//! handled by specialized expert structs implementing the `Expert` trait.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{core::Task, memory::MemoryItem};

/// The execution environment provided to an expert during its turn.
#[derive(Debug, Clone)]
pub struct ExpertContext {
    pub task: Task,
    pub memories: Vec<MemoryItem>,
    pub active_neurons: Vec<String>,
}

/// The structured output produced by an expert after evaluating a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertResult {
    pub expert_name: String,
    pub summary: String,
    pub suggested_actions: Vec<String>,
    pub success: bool,
    pub estimated_cost: f32,
    pub confidence_score: f32,
}

/// The core trait that all specialized cognitive experts must implement.
pub trait Expert: Send + Sync {
    /// Returns the unique string identifier for this expert.
    fn name(&self) -> &'static str;

    /// Evaluates how well this expert can handle the given task (0.0 to 1.0).
    fn can_handle(&self, task: &Task) -> f32;

    /// Estimates the abstract energy/compute cost of running this expert.
    fn estimate_cost(&self, task: &Task) -> f32;

    /// Executes the expert's specific logic on the given context.
    fn run(&self, context: &ExpertContext) -> ExpertResult;
}

/// Manages a registry of experts and dispatches tasks to the most capable one(s).
pub struct ExpertRegistry {
    experts: HashMap<&'static str, Box<dyn Expert>>,
}

impl Default for ExpertRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpertRegistry {
    pub fn new() -> Self {
        Self {
            experts: HashMap::new(),
        }
    }

    /// Registers a new expert into the system.
    pub fn register(&mut self, expert: Box<dyn Expert>) {
        self.experts.insert(expert.name(), expert);
    }

    /// Finds the best expert for a given task, based on the `can_handle` score.
    pub fn find_best_expert(&self, task: &Task) -> Option<&dyn Expert> {
        self.experts
            .values()
            .max_by(|a, b| {
                a.can_handle(task)
                    .partial_cmp(&b.can_handle(task))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|bx| bx.as_ref())
    }

    /// Selects multiple experts that meet a minimum confidence threshold.
    pub fn find_experts_above_threshold(&self, task: &Task, threshold: f32) -> Vec<&dyn Expert> {
        let mut candidates: Vec<&dyn Expert> = self
            .experts
            .values()
            .filter(|e| e.can_handle(task) >= threshold)
            .map(|bx| bx.as_ref())
            .collect();

        // Sort by capability descending
        candidates.sort_by(|a, b| {
            b.can_handle(task)
                .partial_cmp(&a.can_handle(task))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Task, TaskType};

    struct DummyExpertA;
    impl Expert for DummyExpertA {
        fn name(&self) -> &'static str {
            "DummyA"
        }
        fn can_handle(&self, task: &Task) -> f32 {
            if task.task_type == TaskType::Code {
                0.9
            } else {
                0.1
            }
        }
        fn estimate_cost(&self, _task: &Task) -> f32 {
            1.0
        }
        fn run(&self, _ctx: &ExpertContext) -> ExpertResult {
            unimplemented!()
        }
    }

    struct DummyExpertB;
    impl Expert for DummyExpertB {
        fn name(&self) -> &'static str {
            "DummyB"
        }
        fn can_handle(&self, task: &Task) -> f32 {
            if task.task_type == TaskType::Chat {
                0.8
            } else {
                0.2
            }
        }
        fn estimate_cost(&self, _task: &Task) -> f32 {
            1.0
        }
        fn run(&self, _ctx: &ExpertContext) -> ExpertResult {
            unimplemented!()
        }
    }

    #[test]
    fn registry_finds_best_expert() {
        let mut registry = ExpertRegistry::new();
        registry.register(Box::new(DummyExpertA));
        registry.register(Box::new(DummyExpertB));

        let task_code = Task::new("write code".into(), TaskType::Code);

        let best = registry.find_best_expert(&task_code).unwrap();
        assert_eq!(best.name(), "DummyA");

        let task_chat = Task::new("hello".into(), TaskType::Chat);

        let best2 = registry.find_best_expert(&task_chat).unwrap();
        assert_eq!(best2.name(), "DummyB");
    }

    #[test]
    fn registry_filters_by_threshold() {
        let mut registry = ExpertRegistry::new();
        registry.register(Box::new(DummyExpertA));
        registry.register(Box::new(DummyExpertB));

        let task_code = Task::new("write code".into(), TaskType::Code);

        // Only DummyA should pass the 0.5 threshold for Code
        let experts = registry.find_experts_above_threshold(&task_code, 0.5);
        assert_eq!(experts.len(), 1);
        assert_eq!(experts[0].name(), "DummyA");

        // Both should pass a 0.05 threshold
        let experts_low = registry.find_experts_above_threshold(&task_code, 0.05);
        assert_eq!(experts_low.len(), 2);
    }
}
