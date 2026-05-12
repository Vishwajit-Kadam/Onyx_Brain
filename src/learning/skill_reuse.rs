use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

use crate::{
    agency::ParsedGoal,
    energy::EnergyBudget,
    memory::{MemoryItem, MemoryType},
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillKind {
    GenericSkill,
    ProjectWorkflow,
    DiagnosticFix,
    ToolRoutine,
}

impl Default for SkillKind {
    fn default() -> Self {
        Self::GenericSkill
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill_id: String,
    pub title: String,
    #[serde(default)]
    pub skill_kind: SkillKind,
    pub relevance_score: f32,
    pub matched_tags: Vec<String>,
    pub matched_features: Vec<String>,
    pub confidence: f32,
    pub expected_energy_saving: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillApplicationResult {
    pub skill_id: String,
    pub applied: bool,
    pub plan_delta: String,
    pub energy_saving_estimate: f32,
}

pub struct SkillReuseEngine;

impl SkillReuseEngine {
    pub fn find_relevant_skills(
        store: &DiskStore,
        parsed_goal: &ParsedGoal,
        active_neurons: &[String],
        budget: &EnergyBudget,
    ) -> Result<Vec<SkillMatch>> {
        let preferred = preferred_tags();
        let goal_features = parsed_goal
            .requested_features
            .iter()
            .map(|feature| normalize(feature))
            .collect::<BTreeSet<_>>();
        let mut matches = Vec::new();
        for path in store.memory_files()?.into_iter().take(128) {
            let memory: MemoryItem = crate::storage::load_json(&path)?;
            if memory.memory_type != MemoryType::Procedural {
                continue;
            }
            let memory_tags = memory
                .tags
                .iter()
                .map(|tag| normalize(tag))
                .collect::<BTreeSet<_>>();
            if !memory_tags.iter().any(|tag| preferred.contains(tag)) {
                continue;
            }
            let matched_tags = memory_tags
                .iter()
                .filter(|tag| preferred.contains(*tag))
                .cloned()
                .collect::<Vec<_>>();
            let haystack = normalize(&format!(
                "{} {} {}",
                memory.title, memory.summary, memory.content
            ));
            let skill_kind = classify_skill(&memory);
            let project_specific_penalty = if matches!(skill_kind, SkillKind::ProjectWorkflow) {
                let matches_project = parsed_goal
                    .project_name
                    .as_ref()
                    .is_some_and(|name| haystack.contains(&normalize(name)));
                if matches_project {
                    0.0
                } else {
                    0.25
                }
            } else {
                0.0
            };
            let matched_features = goal_features
                .iter()
                .filter(|feature| haystack.contains(feature.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let tag_overlap = if preferred.is_empty() {
                0.0
            } else {
                matched_tags.len() as f32 / preferred.len() as f32
            };
            let feature_overlap = if goal_features.is_empty() {
                0.5
            } else {
                matched_features.len() as f32 / goal_features.len() as f32
            };
            let feature_miss_penalty = if goal_features.is_empty() || !matched_features.is_empty() {
                0.0
            } else {
                0.18
            };
            let successful_reuse = metadata_f32(&memory, "successful_reuse_count");
            let failed_reuse = metadata_f32(&memory, "failed_reuse_count");
            let past_success = if successful_reuse + failed_reuse == 0.0 {
                0.5
            } else {
                successful_reuse / (successful_reuse + failed_reuse)
            };
            let importance = memory.importance.clamp(0.0, 1.0);
            let recency = if memory.last_accessed_at.is_some() {
                1.0
            } else {
                0.5
            };
            let energy_saving_estimate =
                (0.2 + past_success * 0.3 + feature_overlap * 0.2).clamp(0.0, 1.0);
            let generic_bonus =
                if matches!(skill_kind, SkillKind::GenericSkill | SkillKind::ToolRoutine) {
                    0.12
                } else {
                    0.0
                };
            let relevance_score = tag_overlap * 0.30
                + feature_overlap * 0.30
                + past_success * 0.20
                + importance * 0.10
                + recency * 0.05
                + energy_saving_estimate * 0.05
                + generic_bonus
                - project_specific_penalty
                + if active_neurons
                    .iter()
                    .any(|neuron| memory.linked_neurons.iter().any(|linked| linked == neuron))
                {
                    0.05
                } else {
                    0.0
                }
                - feature_miss_penalty;
            if relevance_score > 0.25 {
                matches.push(SkillMatch {
                    skill_id: memory.id,
                    title: memory.title,
                    skill_kind,
                    relevance_score,
                    matched_tags,
                    matched_features,
                    confidence: past_success,
                    expected_energy_saving: energy_saving_estimate,
                    reason: "procedural skill matched goal tags/features".to_string(),
                });
            }
        }
        matches.sort_by(|a, b| b.relevance_score.total_cmp(&a.relevance_score));
        if matches.iter().any(|skill| {
            matches!(
                skill.skill_kind,
                SkillKind::GenericSkill | SkillKind::ToolRoutine
            )
        }) {
            matches.retain(|skill| {
                !matches!(skill.skill_kind, SkillKind::ProjectWorkflow)
                    || parsed_goal.project_name.as_ref().is_some_and(|name| {
                        skill.title.to_lowercase().contains(&name.to_lowercase())
                    })
            });
        }
        let complex_task = parsed_goal.requested_features.len() > 4;
        let default_limit = if complex_task {
            budget.max_memory_items
        } else {
            5
        };
        matches.truncate(budget.max_memory_items.min(default_limit));
        Ok(matches)
    }

    pub fn apply_skill_to_plan(
        skill: &SkillMatch,
        project_plan: &[String],
    ) -> SkillApplicationResult {
        let applied = project_plan
            .iter()
            .any(|step| step.to_lowercase().contains("cargo"))
            || skill.title.to_lowercase().contains("rust");
        SkillApplicationResult {
            skill_id: skill.skill_id.clone(),
            applied,
            plan_delta: if applied {
                format!("Reused known workflow: {}", skill.title)
            } else {
                "Skill did not alter plan.".to_string()
            },
            energy_saving_estimate: if applied {
                skill.expected_energy_saving
            } else {
                0.0
            },
        }
    }
}

pub fn irrelevant_skill_count(matches: &[SkillMatch], project_name: Option<&str>) -> usize {
    matches
        .iter()
        .filter(|skill| {
            matches!(skill.skill_kind, SkillKind::ProjectWorkflow)
                && project_name
                    .is_none_or(|name| !skill.title.to_lowercase().contains(&name.to_lowercase()))
        })
        .count()
}

fn classify_skill(memory: &MemoryItem) -> SkillKind {
    let title = memory.title.to_lowercase();
    if title.starts_with("workflow for") {
        SkillKind::ProjectWorkflow
    } else if title.contains("diagnostic") || memory.tags.iter().any(|tag| tag == "diagnostic") {
        SkillKind::DiagnosticFix
    } else if title.contains("cargo check")
        || title.contains("cargo test")
        || title.contains("tool")
    {
        SkillKind::ToolRoutine
    } else {
        SkillKind::GenericSkill
    }
}

pub fn update_skill_usage(
    store: &DiskStore,
    matches: &[SkillMatch],
    success: bool,
    project_id: &str,
) -> Result<()> {
    for skill in matches {
        let mut memory = store.load_memory(&skill.skill_id)?;
        let usage = metadata_u64(&memory, "usage_count") + 1;
        let successful = metadata_u64(&memory, "successful_reuse_count") + u64::from(success);
        let failed = metadata_u64(&memory, "failed_reuse_count") + u64::from(!success);
        let previous_avg = metadata_f32(&memory, "average_energy_saved");
        let next_avg = ((previous_avg * (usage.saturating_sub(1) as f32))
            + skill.expected_energy_saving)
            / usage.max(1) as f32;
        let mut projects = memory
            .metadata
            .get("linked_project_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if !projects
            .iter()
            .any(|value| value.as_str() == Some(project_id))
        {
            projects.push(json!(project_id));
        }
        memory
            .metadata
            .insert("usage_count".to_string(), json!(usage));
        memory
            .metadata
            .insert("successful_reuse_count".to_string(), json!(successful));
        memory
            .metadata
            .insert("failed_reuse_count".to_string(), json!(failed));
        memory
            .metadata
            .insert("last_reused_at".to_string(), json!(Utc::now()));
        memory
            .metadata
            .insert("average_energy_saved".to_string(), json!(next_avg));
        memory
            .metadata
            .insert("linked_project_ids".to_string(), Value::Array(projects));
        memory.access_count += 1;
        store.save_memory(&memory)?;
    }
    Ok(())
}

fn preferred_tags() -> BTreeSet<String> {
    [
        "rust",
        "project",
        "skill",
        "workflow",
        "tests",
        "readme",
        "calculator",
        "cli",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn metadata_u64(memory: &MemoryItem, key: &str) -> u64 {
    memory
        .metadata
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn metadata_f32(memory: &MemoryItem, key: &str) -> f32 {
    memory
        .metadata
        .get(key)
        .and_then(Value::as_f64)
        .unwrap_or(0.0) as f32
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
