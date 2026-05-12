use serde::{Deserialize, Serialize};

use crate::{
    agency::{GoalMemoryItem, GoalStatus},
    core::Priority,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityScore {
    pub goal_id: String,
    pub score: f32,
}

pub struct PriorityScheduler;

impl PriorityScheduler {
    pub fn score_goal(
        goal: &GoalMemoryItem,
        estimated_energy_cost: f32,
        skill_confidence_bonus: f32,
    ) -> PriorityScore {
        let priority_weight = match goal.priority {
            Priority::High => 3.0,
            Priority::Normal => 2.0,
            Priority::Low => 1.0,
        };
        let expected_value = 1.0 + goal.success_score;
        let blocked_penalty = if matches!(goal.status, GoalStatus::Blocked | GoalStatus::Failed) {
            3.0
        } else {
            0.0
        };
        PriorityScore {
            goal_id: goal.goal_id.clone(),
            score: priority_weight + expected_value + skill_confidence_bonus
                - estimated_energy_cost
                - blocked_penalty,
        }
    }

    pub fn order_goals(mut goals: Vec<GoalMemoryItem>) -> Vec<GoalMemoryItem> {
        goals.sort_by(|a, b| {
            let a_score = Self::score_goal(a, 0.5, 0.5).score;
            let b_score = Self::score_goal(b, 0.5, 0.5).score;
            b_score.total_cmp(&a_score)
        });
        goals
    }
}
