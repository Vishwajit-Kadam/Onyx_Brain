use serde::{Deserialize, Serialize};

use crate::{
    agency::{IntentKind, ParsedGoal},
    core::TaskType,
    learning::HabitMatch,
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyBudget {
    pub max_active_neurons: usize,
    pub max_active_synapses_per_neuron: usize,
    pub max_memory_items: usize,
    pub max_experts: usize,
    pub max_tool_actions: usize,
    pub max_runtime_ms: u64,
}

impl Default for EnergyBudget {
    fn default() -> Self {
        Self {
            max_active_neurons: 32,
            max_active_synapses_per_neuron: 16,
            max_memory_items: 8,
            max_experts: 2,
            max_tool_actions: 3,
            max_runtime_ms: 5_000,
        }
    }
}

pub struct EnergyBudgetManager;

impl EnergyBudgetManager {
    pub fn budget_for(task_type: &TaskType) -> EnergyBudget {
        match task_type {
            TaskType::Code | TaskType::FileOperation => EnergyBudget::default(),
            TaskType::Chat => EnergyBudget {
                max_active_neurons: 16,
                max_experts: 1,
                max_tool_actions: 0,
                ..EnergyBudget::default()
            },
            TaskType::Planning => EnergyBudget {
                max_active_neurons: 24,
                max_experts: 2,
                max_tool_actions: 1,
                ..EnergyBudget::default()
            },
            TaskType::Unknown => EnergyBudget {
                max_active_neurons: 12,
                max_experts: 1,
                max_tool_actions: 0,
                ..EnergyBudget::default()
            },
            _ => EnergyBudget::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptiveBudgetDecisionType {
    Reduced,
    Expanded,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveBudgetDecision {
    pub original_budget: EnergyBudget,
    pub adjusted_budget: EnergyBudget,
    pub decision_type: AdaptiveBudgetDecisionType,
    pub reason: String,
    pub confidence: f32,
    pub estimated_savings: f32,
}

pub struct AdaptiveBudgetManager;

impl AdaptiveBudgetManager {
    pub fn decide_for_task(
        store: &DiskStore,
        task_type: &TaskType,
        parsed_goal: Option<&ParsedGoal>,
        habits: &[HabitMatch],
        cache_hit: bool,
        previous_failure: bool,
    ) -> AdaptiveBudgetDecision {
        let original = EnergyBudgetManager::budget_for(task_type);
        let mut adjusted = original.clone();
        let mut decision_type = AdaptiveBudgetDecisionType::Unchanged;
        let mut reason = "no familiar efficient route found".to_string();
        let mut confidence = 0.5;
        let mut estimated_savings = 0.0;

        if previous_failure {
            adjusted.max_active_neurons =
                (adjusted.max_active_neurons + 4).min(original.max_active_neurons + 8);
            adjusted.max_memory_items =
                (adjusted.max_memory_items + 2).min(original.max_memory_items + 4);
            decision_type = AdaptiveBudgetDecisionType::Expanded;
            reason =
                "previous similar task failed; allowing a slightly larger active set".to_string();
            confidence = 0.65;
        } else if cache_hit || habits.iter().any(|habit| habit.confidence >= 0.7) {
            let reduction = if cache_hit && !habits.is_empty() {
                0.30
            } else {
                0.20
            };
            adjusted.max_active_neurons = reduce_usize(adjusted.max_active_neurons, reduction, 8);
            adjusted.max_active_synapses_per_neuron =
                reduce_usize(adjusted.max_active_synapses_per_neuron, reduction, 4);
            adjusted.max_memory_items = reduce_usize(adjusted.max_memory_items, reduction, 3);
            adjusted.max_experts = adjusted.max_experts.max(1);
            decision_type = AdaptiveBudgetDecisionType::Reduced;
            reason = if cache_hit {
                "familiar goal matched plan cache and/or high-confidence habit".to_string()
            } else {
                "high-confidence habit matched this task".to_string()
            };
            confidence = habits
                .iter()
                .map(|habit| habit.confidence)
                .fold(0.7_f32, f32::max)
                .clamp(0.0, 1.0);
            estimated_savings = reduction;
        }

        if parsed_goal.is_some_and(|goal| {
            goal.intent == IntentKind::ModifyProject && goal.requested_features.len() <= 3
        }) {
            adjusted.max_memory_items = adjusted.max_memory_items.min(5);
            if matches!(decision_type, AdaptiveBudgetDecisionType::Unchanged) {
                decision_type = AdaptiveBudgetDecisionType::Reduced;
                reason = "simple project modification; limiting memory/skill fan-out".to_string();
                confidence = 0.6;
                estimated_savings = 0.1;
            }
        }

        if irrelevant_skill_history(store) > 0 {
            adjusted.max_memory_items = adjusted.max_memory_items.min(5);
            reason.push_str("; previous irrelevant skill reuse detected");
        }

        AdaptiveBudgetDecision {
            original_budget: original,
            adjusted_budget: adjusted,
            decision_type,
            reason,
            confidence,
            estimated_savings,
        }
    }
}

fn reduce_usize(value: usize, reduction: f32, min_value: usize) -> usize {
    ((value as f32 * (1.0 - reduction)).round() as usize).max(min_value)
}

fn irrelevant_skill_history(store: &DiskStore) -> usize {
    crate::storage::try_load_json::<serde_json::Value>(
        &store.paths.indexes.join("skill_reuse_quality.json"),
    )
    .ok()
    .flatten()
    .and_then(|value| {
        value
            .get("irrelevant_skill_count")
            .and_then(|count| count.as_u64())
    })
    .unwrap_or(0) as usize
}
