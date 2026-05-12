use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    agency::{parse_goal, ProjectState},
    learning::{
        find_matching_habits, form_or_strengthen_habit_from_project, habit_signature,
        strengthen_habit,
    },
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiveHabitUpdate {
    pub task_signature: String,
    pub habit_created: bool,
    pub habit_strengthened: bool,
    pub habit_id: Option<String>,
    pub confidence_delta: f32,
    pub reason: String,
}

pub fn update_live_habit_after_project(
    store: &DiskStore,
    prompt: &str,
    state: &ProjectState,
    plan_template: Vec<String>,
    runtime_ms: u64,
    energy: f32,
) -> Result<LiveHabitUpdate> {
    let parsed = parse_goal(prompt);
    let signature = habit_signature(&parsed);
    if state.status != "Completed" {
        return Ok(LiveHabitUpdate {
            task_signature: signature,
            reason: "project did not complete; live habit unchanged".to_string(),
            ..LiveHabitUpdate::default()
        });
    }

    let before = find_matching_habits(store, &parsed, 1)?;
    if let Some(existing) = before.first() {
        strengthen_habit(store, &existing.habit_id, true, runtime_ms, energy)?;
        return Ok(LiveHabitUpdate {
            task_signature: signature,
            habit_created: false,
            habit_strengthened: true,
            habit_id: Some(existing.habit_id.clone()),
            confidence_delta: 0.05,
            reason: "matching habit strengthened during successful work".to_string(),
        });
    }

    let (created, strengthened) = form_or_strengthen_habit_from_project(
        store,
        &parsed,
        state,
        plan_template,
        runtime_ms,
        energy,
    )?;
    let after = find_matching_habits(store, &parsed, 1)?;
    Ok(LiveHabitUpdate {
        task_signature: signature,
        habit_created: created > 0,
        habit_strengthened: strengthened > 0,
        habit_id: after.first().map(|habit| habit.habit_id.clone()),
        confidence_delta: if created > 0 {
            0.72
        } else if strengthened > 0 {
            0.07
        } else {
            0.0
        },
        reason: if created > 0 {
            "repeated successful workflow created a live habit".to_string()
        } else if strengthened > 0 {
            "existing workflow habit strengthened live".to_string()
        } else {
            "not enough repeated successes for a new habit yet".to_string()
        },
    })
}
