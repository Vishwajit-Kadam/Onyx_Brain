use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};
use uuid::Uuid;

use crate::{
    agency::{IntentKind, ParsedGoal, ProjectState},
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub habit_id: String,
    pub title: String,
    pub trigger_patterns: Vec<String>,
    pub task_type: String,
    pub project_type: Option<String>,
    pub required_features: Vec<String>,
    pub plan_template: Vec<String>,
    pub preferred_skills: Vec<String>,
    pub preferred_tools: Vec<String>,
    pub average_runtime_ms: f32,
    pub average_energy: f32,
    pub success_count: u64,
    pub failure_count: u64,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HabitIndex {
    pub habits: BTreeMap<String, HabitSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitSummary {
    pub habit_id: String,
    pub title: String,
    pub trigger_patterns: Vec<String>,
    pub confidence: f32,
    pub success_count: u64,
    pub failure_count: u64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitMatch {
    pub habit_id: String,
    pub title: String,
    pub relevance_score: f32,
    pub confidence: f32,
    pub matched_patterns: Vec<String>,
    pub matched_features: Vec<String>,
    pub expected_energy_saving: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HabitOverview {
    pub habit_count: usize,
    pub top_habits: Vec<String>,
    pub total_successes: u64,
    pub total_failures: u64,
}

pub fn habits_dir(store: &DiskStore) -> std::path::PathBuf {
    store.paths.data.join("habits")
}

pub fn habit_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("habit_index.json")
}

pub fn habit_path(store: &DiskStore, habit_id: &str) -> std::path::PathBuf {
    habits_dir(store).join(format!("{habit_id}.json"))
}

pub fn load_habit_index(store: &DiskStore) -> Result<HabitIndex> {
    let path = habit_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(HabitIndex::default())
    }
}

pub fn save_habit(store: &DiskStore, habit: &Habit) -> Result<()> {
    fs::create_dir_all(habits_dir(store))?;
    save_json(&habit_path(store, &habit.habit_id), habit)?;
    let mut index = load_habit_index(store)?;
    index.habits.insert(
        habit.habit_id.clone(),
        HabitSummary {
            habit_id: habit.habit_id.clone(),
            title: habit.title.clone(),
            trigger_patterns: habit.trigger_patterns.clone(),
            confidence: habit.confidence,
            success_count: habit.success_count,
            failure_count: habit.failure_count,
            updated_at: habit.updated_at,
        },
    );
    save_json(&habit_index_path(store), &index)
}

pub fn load_habit(store: &DiskStore, habit_id: &str) -> Result<Habit> {
    load_json(&habit_path(store, habit_id))
}

pub fn list_habits(store: &DiskStore) -> Result<Vec<Habit>> {
    let index = load_habit_index(store)?;
    let mut habits = Vec::new();
    for id in index.habits.keys() {
        if let Ok(habit) = load_habit(store, id) {
            habits.push(habit);
        }
    }
    habits.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then(b.success_count.cmp(&a.success_count))
    });
    Ok(habits)
}

pub fn find_matching_habits(
    store: &DiskStore,
    parsed: &ParsedGoal,
    limit: usize,
) -> Result<Vec<HabitMatch>> {
    let goal_text = normalize(&parsed.original_prompt);
    let goal_features = parsed
        .requested_features
        .iter()
        .map(|feature| normalize(feature))
        .collect::<Vec<_>>();
    let mut matches = Vec::new();
    for habit in list_habits(store)?.into_iter().take(64) {
        let matched_patterns = habit
            .trigger_patterns
            .iter()
            .filter(|pattern| goal_text.contains(&normalize(pattern)))
            .cloned()
            .collect::<Vec<_>>();
        let matched_features = goal_features
            .iter()
            .filter(|feature| {
                habit
                    .required_features
                    .iter()
                    .any(|required| normalize(required) == **feature)
            })
            .cloned()
            .collect::<Vec<_>>();
        let pattern_score = if matched_patterns.is_empty() {
            0.0
        } else {
            0.45
        };
        let feature_score = if goal_features.is_empty() {
            0.25
        } else {
            matched_features.len() as f32 / goal_features.len() as f32 * 0.35
        };
        let intent_score = if habit.task_type == format!("{:?}", parsed.intent) {
            0.1
        } else {
            0.0
        };
        let relevance =
            (pattern_score + feature_score + intent_score + habit.confidence * 0.1).clamp(0.0, 1.0);
        if relevance >= 0.35 && habit.confidence >= 0.45 {
            matches.push(HabitMatch {
                habit_id: habit.habit_id,
                title: habit.title,
                relevance_score: relevance,
                confidence: habit.confidence,
                matched_patterns,
                matched_features,
                expected_energy_saving: (0.1 + habit.confidence * 0.2).clamp(0.0, 0.3),
                reason: "habit matched normalized goal patterns/features".to_string(),
            });
        }
    }
    matches.sort_by(|a, b| {
        b.relevance_score
            .total_cmp(&a.relevance_score)
            .then(b.confidence.total_cmp(&a.confidence))
    });
    matches.truncate(limit);
    Ok(matches)
}

pub fn strengthen_habit(
    store: &DiskStore,
    habit_id: &str,
    success: bool,
    runtime_ms: u64,
    energy: f32,
) -> Result<()> {
    let mut habit = load_habit(store, habit_id)?;
    let count = habit.success_count + habit.failure_count + 1;
    habit.success_count += u64::from(success);
    habit.failure_count += u64::from(!success);
    habit.average_runtime_ms = rolling_average(habit.average_runtime_ms, runtime_ms as f32, count);
    habit.average_energy = rolling_average(habit.average_energy, energy, count);
    habit.confidence = if success {
        (habit.confidence + 0.05).clamp(0.0, 1.0)
    } else {
        (habit.confidence - 0.08).clamp(0.0, 1.0)
    };
    habit.updated_at = Utc::now();
    save_habit(store, &habit)
}

pub fn form_or_strengthen_habit_from_project(
    store: &DiskStore,
    parsed: &ParsedGoal,
    state: &ProjectState,
    plan_template: Vec<String>,
    runtime_ms: u64,
    energy: f32,
) -> Result<(usize, usize)> {
    if state.status != "Completed" {
        return Ok((0, 0));
    }
    let signature = habit_signature(parsed);
    let mut existing = list_habits(store)?.into_iter().find(|habit| {
        habit
            .trigger_patterns
            .iter()
            .any(|pattern| pattern == &signature)
    });
    if existing.is_none() && successful_project_count(store, &signature)? < 3 {
        return Ok((0, 0));
    }
    if let Some(mut habit) = existing.take() {
        let total = habit.success_count + habit.failure_count + 1;
        habit.success_count += 1;
        habit.average_runtime_ms =
            rolling_average(habit.average_runtime_ms, runtime_ms as f32, total);
        habit.average_energy = rolling_average(habit.average_energy, energy, total);
        habit.confidence = (habit.confidence + 0.07).clamp(0.0, 1.0);
        habit.updated_at = Utc::now();
        save_habit(store, &habit)?;
        return Ok((0, 1));
    }
    let now = Utc::now();
    let habit = Habit {
        habit_id: format!("habit_{}", Uuid::new_v4()),
        title: generic_title(parsed),
        trigger_patterns: vec![signature],
        task_type: format!("{:?}", parsed.intent),
        project_type: Some("rust_cli".to_string()),
        required_features: parsed.requested_features.clone(),
        plan_template,
        preferred_skills: vec![
            "skill_create_rust_cli_project".to_string(),
            "skill_add_rust_unit_tests".to_string(),
            "skill_run_cargo_check_and_cargo_test".to_string(),
        ],
        preferred_tools: vec!["CodeEditorTool".to_string(), "TerminalTool".to_string()],
        average_runtime_ms: runtime_ms as f32,
        average_energy: energy,
        success_count: 3,
        failure_count: 0,
        confidence: 0.72,
        created_at: now,
        updated_at: now,
    };
    save_habit(store, &habit)?;
    Ok((1, 0))
}

pub fn habit_overview(store: &DiskStore) -> Result<HabitOverview> {
    let habits = list_habits(store)?;
    Ok(HabitOverview {
        habit_count: habits.len(),
        top_habits: habits
            .iter()
            .take(5)
            .map(|habit| {
                format!(
                    "{} confidence {:.2} success {}/{}",
                    habit.title,
                    habit.confidence,
                    habit.success_count,
                    habit.success_count + habit.failure_count
                )
            })
            .collect(),
        total_successes: habits.iter().map(|habit| habit.success_count).sum(),
        total_failures: habits.iter().map(|habit| habit.failure_count).sum(),
    })
}

pub fn habit_signature(parsed: &ParsedGoal) -> String {
    let mut features = parsed
        .requested_features
        .iter()
        .map(|feature| normalize(feature))
        .filter(|feature| feature != "tests" && feature != "readme")
        .collect::<Vec<_>>();
    features.sort();
    features.dedup();
    let intent = match parsed.intent {
        IntentKind::ModifyProject => "modify",
        IntentKind::CreateProject => "create",
        _ => "goal",
    };
    format!("{intent}:rust_cli:{}", features.join(","))
}

fn successful_project_count(store: &DiskStore, signature: &str) -> Result<usize> {
    let mut count = 0;
    for path in store.list_log_files()? {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("project_trace_") {
            continue;
        }
        let Ok(value) = load_json::<serde_json::Value>(&path) else {
            continue;
        };
        let input = value
            .get("task_input")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let parsed = crate::agency::parse_goal(input);
        if value.get("success").and_then(|value| value.as_bool()) == Some(true)
            && habit_signature(&parsed) == signature
        {
            count += 1;
        }
    }
    Ok(count)
}

fn generic_title(parsed: &ParsedGoal) -> String {
    let features = parsed
        .requested_features
        .iter()
        .map(|feature| feature.to_lowercase())
        .collect::<Vec<_>>();
    if parsed.intent == IntentKind::ModifyProject
        && features.iter().any(|feature| feature == "multiply")
        && features.iter().any(|feature| feature == "divide")
    {
        "Add calculator operations with tests".to_string()
    } else if parsed.intent == IntentKind::CreateProject {
        "Create Rust CLI calculator project".to_string()
    } else {
        "Run Rust project workflow".to_string()
    }
}

fn rolling_average(previous: f32, next: f32, count: u64) -> f32 {
    if count <= 1 {
        next
    } else {
        ((previous * (count - 1) as f32) + next) / count as f32
    }
}

fn normalize(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
