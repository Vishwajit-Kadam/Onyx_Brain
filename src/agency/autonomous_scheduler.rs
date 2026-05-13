use serde::{Deserialize, Serialize};

use crate::agency::{AutonomousTaskType, GraphTaskStatus, TaskGraph, TaskNode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDecision {
    pub task_id: String,
    pub selected: bool,
    pub reason: String,
    pub priority_score: f32,
    pub risk_score: f32,
    pub estimated_cost: f32,
}

pub fn schedule_ready_tasks(graph: &TaskGraph, max_tasks: usize) -> Vec<ScheduleDecision> {
    let mut decisions = graph
        .nodes
        .iter()
        .filter(|task| task.status == GraphTaskStatus::Pending)
        .map(|task| schedule_task(graph, task))
        .collect::<Vec<_>>();
    decisions.sort_by(|a, b| b.priority_score.total_cmp(&a.priority_score));
    let mut selected = 0;
    for decision in &mut decisions {
        if decision.selected && selected < max_tasks {
            selected += 1;
        } else {
            decision.selected = false;
            if selected >= max_tasks {
                decision.reason =
                    "not selected because max scheduled tasks would be exceeded".to_string();
            }
        }
    }
    decisions
}

fn schedule_task(graph: &TaskGraph, task: &TaskNode) -> ScheduleDecision {
    let dependencies_ready = graph
        .edges
        .iter()
        .filter(|edge| edge.to_task_id == task.task_id)
        .all(|edge| {
            graph
                .nodes
                .iter()
                .find(|node| node.task_id == edge.from_task_id)
                .is_some_and(|node| node.status == GraphTaskStatus::Completed)
        });
    let dependency_readiness = if dependencies_ready { 1.0 } else { 0.0 };
    let deliverable_importance = if task.task_type == AutonomousTaskType::GenerateArtifact {
        1.0
    } else {
        0.7
    };
    let risk_score = match task.task_type {
        AutonomousTaskType::ReviseArtifact => 0.35,
        AutonomousTaskType::ExportPackage => 0.15,
        _ => 0.05,
    };
    let estimated_cost = (task.priority.max(1) as f32 / 100.0).clamp(0.05, 1.0);
    let priority_score = task.priority as f32 * 0.30 / 100.0
        + dependency_readiness * 0.25
        + deliverable_importance * 0.20
        + 0.75 * 0.15
        - risk_score * 0.10;
    ScheduleDecision {
        task_id: task.task_id.clone(),
        selected: dependencies_ready,
        reason: if dependencies_ready {
            "dependencies complete and task is within budget".to_string()
        } else {
            "waiting for dependencies".to_string()
        },
        priority_score,
        risk_score,
        estimated_cost,
    }
}
