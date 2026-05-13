pub mod dedup;
pub mod episodic;
pub mod hygiene;
pub mod procedural;
pub mod project;
pub mod reflection;
pub mod self_critique;
pub mod semantic;
pub mod working;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

use crate::{
    core::{MemoryId, NeuronId, Task, TaskType},
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryType {
    Working,
    Episodic,
    Semantic,
    Procedural,
    Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: MemoryId,
    pub memory_type: MemoryType,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub importance: f32,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub access_count: u64,
    pub linked_neurons: Vec<NeuronId>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

impl MemoryItem {
    pub fn new(
        id: impl Into<String>,
        memory_type: MemoryType,
        title: impl Into<String>,
        content: impl Into<String>,
        tags: Vec<String>,
        linked_neurons: Vec<NeuronId>,
    ) -> Self {
        let content = content.into();
        Self {
            id: id.into(),
            memory_type,
            title: title.into(),
            summary: content.chars().take(120).collect(),
            content,
            tags,
            importance: 0.5,
            last_accessed_at: None,
            created_at: Utc::now(),
            access_count: 0,
            linked_neurons,
            metadata: Map::new(),
        }
    }
}

pub fn retrieve_relevant_memories(
    store: &DiskStore,
    task: &Task,
    active_neuron_ids: &[NeuronId],
    limit: usize,
) -> Result<Vec<MemoryItem>> {
    let task_words = tokenize(&task.input);
    let candidate_ids = candidate_memory_ids(store, &task_words)?;
    let mut scored = Vec::new();
    for id in candidate_ids {
        if let Ok(memory) = store.load_memory(&id) {
            let score = score_memory(&memory, task, &task_words, active_neuron_ids);
            if score > 0.0 {
                scored.push((score, memory));
            }
        }
    }
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut selected = scored
        .into_iter()
        .take(limit)
        .map(|(_, mut memory)| {
            memory.last_accessed_at = Some(Utc::now());
            memory.access_count += 1;
            memory
        })
        .collect::<Vec<_>>();
    for memory in &selected {
        store.save_memory(memory)?;
    }
    Ok(std::mem::take(&mut selected))
}

pub fn score_memory(
    memory: &MemoryItem,
    task: &Task,
    task_words: &[String],
    active_neuron_ids: &[NeuronId],
) -> f32 {
    let memory_words = memory_word_set(memory);
    let keyword_overlap = ratio_overlap(task_words, &memory_words);
    let memory_tags = memory
        .tags
        .iter()
        .map(|tag| normalize_token(tag))
        .collect::<BTreeSet<_>>();
    let tag_overlap = ratio_overlap(task_words, &memory_tags);
    let task_type_match = task_type_memory_match(&task.task_type, &memory.memory_type);
    let importance = memory.importance.clamp(0.0, 1.0);
    let linked_neuron_bonus = if active_neuron_ids
        .iter()
        .any(|id| memory.linked_neurons.iter().any(|linked| linked == id))
    {
        1.0
    } else {
        0.0
    };
    let recency_bonus = recency_bonus(memory);
    let access_bonus = (memory.access_count as f32 / 10.0).clamp(0.0, 1.0);

    keyword_overlap * 0.35
        + tag_overlap * 0.20
        + task_type_match * 0.15
        + importance * 0.15
        + linked_neuron_bonus * 0.10
        + recency_bonus * 0.03
        + access_bonus * 0.02
}

pub fn tokenize(input: &str) -> Vec<String> {
    input
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|word| !word.is_empty())
        .map(normalize_token)
        .filter(|word| word.len() > 2)
        .collect()
}

fn candidate_memory_ids(store: &DiskStore, task_words: &[String]) -> Result<BTreeSet<MemoryId>> {
    let tag_index = store.memory_tag_index()?;
    let keyword_index = store.memory_keyword_index()?;
    let mut ids = BTreeSet::new();
    for word in task_words {
        if let Some(matches) = tag_index.0.get(word) {
            ids.extend(matches.iter().cloned());
        }
        if let Some(matches) = keyword_index.0.get(word) {
            ids.extend(matches.iter().cloned());
        }
    }

    if ids.is_empty() {
        for path in store.memory_files()?.into_iter().take(32) {
            if let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) {
                ids.insert(id.to_string());
            }
        }
    }
    Ok(ids)
}

fn memory_word_set(memory: &MemoryItem) -> BTreeSet<String> {
    tokenize(&format!(
        "{} {} {}",
        memory.title, memory.summary, memory.content
    ))
    .into_iter()
    .collect()
}

fn ratio_overlap(task_words: &[String], candidate_words: &BTreeSet<String>) -> f32 {
    if task_words.is_empty() {
        return 0.0;
    }
    let matches = task_words
        .iter()
        .filter(|word| candidate_words.contains(*word))
        .count();
    (matches as f32 / task_words.len() as f32).clamp(0.0, 1.0)
}

fn task_type_memory_match(task_type: &TaskType, memory_type: &MemoryType) -> f32 {
    match (task_type, memory_type) {
        (TaskType::Code | TaskType::FileOperation, MemoryType::Procedural) => 1.0,
        (TaskType::Code | TaskType::FileOperation, MemoryType::Semantic) => 0.6,
        (TaskType::MemoryQuery, MemoryType::Semantic | MemoryType::Episodic) => 1.0,
        (TaskType::Planning, MemoryType::Procedural | MemoryType::Project) => 0.8,
        (TaskType::Chat, MemoryType::Semantic) => 0.4,
        _ => 0.0,
    }
}

fn recency_bonus(memory: &MemoryItem) -> f32 {
    let reference = memory.last_accessed_at.unwrap_or(memory.created_at);
    let age = Utc::now().signed_duration_since(reference).num_days();
    if age <= 1 {
        1.0
    } else if age <= 7 {
        0.7
    } else if age <= 30 {
        0.3
    } else {
        0.0
    }
}

fn normalize_token(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}
