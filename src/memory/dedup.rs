use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    memory::{MemoryItem, MemoryType},
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryHygieneReport {
    pub total_memories: usize,
    pub semantic_memories: usize,
    pub procedural_memories: usize,
    pub project_memories: usize,
    pub archived_memories: usize,
    pub duplicate_groups: usize,
    pub duplicate_project_memories: usize,
    pub duplicate_procedural_skills: usize,
    pub top_reusable_skills: Vec<String>,
    pub stale_memories: usize,
    pub memory_index_size: usize,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDedupReport {
    pub duplicate_groups: usize,
    pub memories_archived: usize,
    pub report_path: String,
}

pub fn inspect_memory_hygiene(store: &DiskStore) -> Result<MemoryHygieneReport> {
    let memories = load_active_memories(store)?;
    let archived_memories = count_json(&store.paths.memories.join("archive"))?;
    let mut report = MemoryHygieneReport {
        total_memories: memories.len(),
        semantic_memories: memories
            .iter()
            .filter(|memory| memory.memory_type == MemoryType::Semantic)
            .count(),
        procedural_memories: memories
            .iter()
            .filter(|memory| memory.memory_type == MemoryType::Procedural)
            .count(),
        project_memories: memories
            .iter()
            .filter(|memory| memory.memory_type == MemoryType::Project)
            .count(),
        archived_memories,
        stale_memories: memories
            .iter()
            .filter(|memory| memory.access_count == 0 && memory.importance < 0.4)
            .count(),
        memory_index_size: store.memory_tag_index()?.0.len()
            + store.memory_keyword_index()?.0.len(),
        ..MemoryHygieneReport::default()
    };
    let groups = duplicate_groups(&memories);
    report.duplicate_groups = groups.values().filter(|group| group.len() > 1).count();
    report.duplicate_project_memories = groups
        .iter()
        .filter(|(key, group)| key.starts_with("project:") && group.len() > 1)
        .map(|(_, group)| group.len() - 1)
        .sum();
    report.duplicate_procedural_skills = groups
        .iter()
        .filter(|(key, group)| key.starts_with("skill:") && group.len() > 1)
        .map(|(_, group)| group.len() - 1)
        .sum();
    let mut skills = memories
        .iter()
        .filter(|memory| memory.memory_type == MemoryType::Procedural)
        .map(|memory| {
            (
                memory.importance + memory.access_count as f32 * 0.01,
                format!("{}: {}", memory.id, memory.title),
            )
        })
        .collect::<Vec<_>>();
    skills.sort_by(|a, b| b.0.total_cmp(&a.0));
    report.top_reusable_skills = skills.into_iter().take(10).map(|(_, row)| row).collect();
    report.recommendation = if report.duplicate_groups > 0 {
        "run memory-dedup".to_string()
    } else {
        "memory hygiene looks clean".to_string()
    };
    Ok(report)
}

pub fn dedup_memories(store: &DiskStore) -> Result<MemoryDedupReport> {
    store.ensure_layout()?;
    let memories = load_active_memories(store)?;
    let groups = duplicate_groups(&memories);
    let archive_dir = store.paths.memories.join("archive");
    fs::create_dir_all(&archive_dir)?;
    let mut duplicate_groups_count = 0;
    let mut archived = 0;
    for group in groups.values().filter(|group| group.len() > 1) {
        duplicate_groups_count += 1;
        let mut sorted = group.clone();
        sorted.sort_by(|a, b| {
            b.importance
                .total_cmp(&a.importance)
                .then(b.created_at.cmp(&a.created_at))
        });
        let keep = sorted.remove(0);
        let mut merged = keep.clone();
        for duplicate in sorted {
            merged.access_count += duplicate.access_count;
            merged.importance = merged.importance.max(duplicate.importance);
            for tag in duplicate.tags {
                if !merged.tags.iter().any(|existing| existing == &tag) {
                    merged.tags.push(tag);
                }
            }
            let source = store.memory_path(&duplicate.id);
            if source.exists() {
                let target = unique_archive_path(&archive_dir, &duplicate.id);
                fs::rename(source, target)?;
                archived += 1;
            }
        }
        store.save_memory(&merged)?;
    }
    let report_name = format!("memory_dedup_report_{}", timestamp_slug());
    let report_path = store.paths.logs.join(format!("{report_name}.json"));
    let report = MemoryDedupReport {
        duplicate_groups: duplicate_groups_count,
        memories_archived: archived,
        report_path: report_path.display().to_string(),
    };
    save_json(&report_path, &report)?;
    Ok(report)
}

pub fn duplicate_groups(memories: &[MemoryItem]) -> BTreeMap<String, Vec<MemoryItem>> {
    let mut groups: BTreeMap<String, Vec<MemoryItem>> = BTreeMap::new();
    for memory in memories {
        let key = match memory.memory_type {
            MemoryType::Project => {
                let project = memory
                    .title
                    .trim_start_matches("Project ")
                    .to_lowercase()
                    .replace(' ', "_");
                format!("project:{project}")
            }
            MemoryType::Procedural => format!(
                "skill:{}:{}",
                normalize(&memory.title),
                normalized_tags(&memory.tags)
            ),
            _ => continue,
        };
        groups.entry(key).or_default().push(memory.clone());
    }
    groups
}

fn load_active_memories(store: &DiskStore) -> Result<Vec<MemoryItem>> {
    let mut memories = Vec::new();
    for path in store.memory_files()? {
        if path.components().any(|part| part.as_os_str() == "archive") {
            continue;
        }
        memories.push(load_json(&path)?);
    }
    Ok(memories)
}

fn unique_archive_path(archive_dir: &Path, id: &str) -> PathBuf {
    archive_dir.join(format!("{id}_{}.json", timestamp_slug()))
}

fn count_json(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }
    Ok(fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .count())
}

fn normalized_tags(tags: &[String]) -> String {
    let mut tags = tags.iter().map(|tag| normalize(tag)).collect::<Vec<_>>();
    tags.sort();
    tags.dedup();
    tags.join(",")
}

fn normalize(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
