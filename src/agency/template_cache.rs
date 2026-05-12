use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};

use crate::{
    agency::ParsedGoal,
    experts::CodeExpert,
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub relative_path: String,
    pub content_template: String,
    pub placeholders: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCacheEntry {
    pub template_id: String,
    pub template_name: String,
    pub project_type: String,
    pub features: Vec<String>,
    pub files: Vec<TemplateFile>,
    pub success_count: u64,
    pub failure_count: u64,
    pub average_runtime_saved_ms: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCacheSummary {
    pub template_id: String,
    pub template_name: String,
    pub project_type: String,
    pub features: Vec<String>,
    pub success_count: u64,
    pub failure_count: u64,
    pub average_runtime_saved_ms: f32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateCacheIndex {
    pub templates: BTreeMap<String, TemplateCacheSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateCacheOverview {
    pub entries: usize,
    pub top_templates: Vec<String>,
    #[serde(default)]
    pub estimated_runtime_saved: f32,
    #[serde(default)]
    pub cache_hit_rate: f32,
}

pub fn template_cache_dir(store: &DiskStore) -> std::path::PathBuf {
    store.paths.data.join("cache").join("templates")
}

pub fn template_cache_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("template_cache_index.json")
}

pub fn template_cache_path(store: &DiskStore, template_id: &str) -> std::path::PathBuf {
    template_cache_dir(store).join(format!("{template_id}.json"))
}

pub fn load_template_cache_index(store: &DiskStore) -> Result<TemplateCacheIndex> {
    let path = template_cache_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(TemplateCacheIndex::default())
    }
}

pub fn save_template_cache_entry(store: &DiskStore, entry: &TemplateCacheEntry) -> Result<()> {
    fs::create_dir_all(template_cache_dir(store))?;
    save_json(&template_cache_path(store, &entry.template_id), entry)?;
    let mut index = load_template_cache_index(store)?;
    index.templates.insert(
        entry.template_id.clone(),
        TemplateCacheSummary {
            template_id: entry.template_id.clone(),
            template_name: entry.template_name.clone(),
            project_type: entry.project_type.clone(),
            features: entry.features.clone(),
            success_count: entry.success_count,
            failure_count: entry.failure_count,
            average_runtime_saved_ms: entry.average_runtime_saved_ms,
            updated_at: entry.updated_at,
        },
    );
    save_json(&template_cache_index_path(store), &index)
}

pub fn load_template_cache_entry(
    store: &DiskStore,
    template_id: &str,
) -> Result<TemplateCacheEntry> {
    load_json(&template_cache_path(store, template_id))
}

pub fn find_template_for_goal(
    store: &DiskStore,
    parsed: &ParsedGoal,
) -> Result<Option<TemplateCacheEntry>> {
    let index = load_template_cache_index(store)?;
    let mut best: Option<TemplateCacheSummary> = None;
    for summary in index.templates.values() {
        if summary.project_type != "rust_cli_calculator" {
            continue;
        }
        let feature_overlap = parsed
            .requested_features
            .iter()
            .filter(|feature| {
                summary
                    .features
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(feature))
            })
            .count();
        if feature_overlap >= 2
            && best
                .as_ref()
                .is_none_or(|current| summary.success_count > current.success_count)
        {
            best = Some(summary.clone());
        }
    }
    best.map(|summary| load_template_cache_entry(store, &summary.template_id))
        .transpose()
}

pub fn render_template_files(
    entry: &TemplateCacheEntry,
    project_name: &str,
) -> Vec<(String, String)> {
    entry
        .files
        .iter()
        .map(|file| {
            (
                file.relative_path.clone(),
                file.content_template
                    .replace("{{project_name}}", project_name),
            )
        })
        .collect()
}

pub fn store_or_strengthen_rust_cli_template(
    store: &DiskStore,
    parsed: &ParsedGoal,
    project_name: &str,
    runtime_saved_ms: f32,
) -> Result<TemplateCacheEntry> {
    let mut entry = find_template_for_goal(store, parsed)?.unwrap_or_else(|| {
        let now = Utc::now();
        rust_cli_calculator_template(project_name, parsed.requested_features.clone(), now)
    });
    entry.success_count += 1;
    let count = entry.success_count + entry.failure_count;
    entry.average_runtime_saved_ms =
        ((entry.average_runtime_saved_ms * count.saturating_sub(1) as f32) + runtime_saved_ms)
            / count.max(1) as f32;
    entry.updated_at = Utc::now();
    save_template_cache_entry(store, &entry)?;
    Ok(entry)
}

pub fn template_cache_overview(store: &DiskStore) -> Result<TemplateCacheOverview> {
    let index = load_template_cache_index(store)?;
    let mut templates = index.templates.values().cloned().collect::<Vec<_>>();
    templates.sort_by(|a, b| b.success_count.cmp(&a.success_count));
    Ok(TemplateCacheOverview {
        entries: templates.len(),
        estimated_runtime_saved: templates
            .iter()
            .map(|template| template.average_runtime_saved_ms)
            .sum(),
        cache_hit_rate: if templates.is_empty() {
            0.0
        } else {
            let successes = templates
                .iter()
                .map(|template| template.success_count)
                .sum::<u64>() as f32;
            let total = templates
                .iter()
                .map(|template| template.success_count + template.failure_count)
                .sum::<u64>()
                .max(1) as f32;
            successes / total
        },
        top_templates: templates
            .iter()
            .take(10)
            .map(|template| {
                format!(
                    "{} | {} | success {} | saved {:.0}ms | features {}",
                    template.template_name,
                    template.project_type,
                    template.success_count,
                    template.average_runtime_saved_ms,
                    template.features.join(", ")
                )
            })
            .collect(),
    })
}

fn rust_cli_calculator_template(
    _project_name: &str,
    features: Vec<String>,
    now: DateTime<Utc>,
) -> TemplateCacheEntry {
    TemplateCacheEntry {
        template_id: "rust_cli_calculator_with_tests".to_string(),
        template_name: "rust_cli_calculator_with_tests".to_string(),
        project_type: "rust_cli_calculator".to_string(),
        features,
        files: vec![
            TemplateFile {
                relative_path: "Cargo.toml".to_string(),
                content_template:
                    "[package]\nname = \"{{project_name}}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n"
                        .to_string(),
                placeholders: vec!["project_name".to_string()],
            },
            TemplateFile {
                relative_path: "src/main.rs".to_string(),
                content_template: CodeExpert::calculator_main("{{project_name}}"),
                placeholders: vec!["project_name".to_string()],
            },
            TemplateFile {
                relative_path: "src/lib.rs".to_string(),
                content_template: CodeExpert::calculator_lib().to_string(),
                placeholders: Vec::new(),
            },
            TemplateFile {
                relative_path: "tests/calculator.rs".to_string(),
                content_template: CodeExpert::calculator_tests("{{project_name}}"),
                placeholders: vec!["project_name".to_string()],
            },
            TemplateFile {
                relative_path: "README.md".to_string(),
                content_template: "# {{project_name}}\n\nGenerated by Onyx Brain inside the sandbox.\n\n## Run\n\n```bash\ncargo run\ncargo test\n```\n".to_string(),
                placeholders: vec!["project_name".to_string()],
            },
        ],
        success_count: 0,
        failure_count: 0,
        average_runtime_saved_ms: 0.0,
        created_at: now,
        updated_at: now,
    }
}
