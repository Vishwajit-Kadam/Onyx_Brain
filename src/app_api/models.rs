use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SafetyStatus {
    pub sandbox_enabled: bool,
    pub network_default: String,
    pub allowlisted_commands: Vec<String>,
    pub safety_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySummaryRow {
    pub title: String,
    pub memory_type: String,
    pub importance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub demo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectItem {
    pub id: String,
    pub name: String,
    pub status: String,
    pub summary: String,
    pub root_path: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentActivityItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub sandbox_dir: PathBuf,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchResultKind {
    Task,
    Artifact,
    ArtifactPack,
    Session,
    Memory,
    Project,
    Command,
    Demo,
}

impl Default for SearchResultKind {
    fn default() -> Self {
        Self::Demo
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResult {
    pub title: String,
    pub subtitle: String,
    pub kind: SearchResultKind,
    pub path_or_id: String,
    pub action: String,
    pub demo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CreativeProjectType {
    #[default]
    General,
    Video,
    Movie,
    Design,
    Slides,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppRuntimeSettings {
    pub default_personality: String,
    pub default_conversation_mode: String,
    pub default_autonomy_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppActionResult {
    pub title: String,
    pub message: String,
    pub details: Vec<String>,
}
