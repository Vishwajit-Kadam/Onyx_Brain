use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{
    agency::{
        action_journal_index_path, load_action_journal_index, load_project_registry,
        load_session_index, load_workspace_index, recipe_index_path, session_index_path,
        workspace_index_path,
    },
    artifacts::{
        artifact_index_path, artifact_pack_index_path, load_artifact_index,
        load_artifact_pack_index,
    },
    conversation::{
        conversation_index_path, load_conversation_index, personality_path, save_personality,
        PersonalityProfile,
    },
    routing::load_route_efficiency,
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StateIssueType {
    MissingFile,
    CorruptJson,
    SchemaMismatch,
    StaleIndex,
    BrokenReference,
    DuplicateEntry,
    MissingIndex,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateHealthIssue {
    pub path: String,
    pub issue_type: StateIssueType,
    pub severity: Severity,
    pub message: String,
    pub repair_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub issues: Vec<StateHealthIssue>,
    pub issues_found: usize,
    pub critical: usize,
    pub warnings: usize,
    pub repair_available: usize,
    pub repaired: usize,
    pub recommendation: String,
    pub reliability_state_health: f32,
    pub created_at: DateTime<Utc>,
    pub report_path: String,
}

pub fn doctor(store: &DiskStore, repair: bool) -> Result<DoctorReport> {
    store.ensure_layout()?;
    let mut issues = Vec::new();
    let required_indexes = [
        store.paths.indexes.join("label_index.json"),
        store.paths.indexes.join("task_type_index.json"),
        store.paths.indexes.join("memory_tags.json"),
        store.paths.indexes.join("memory_keywords.json"),
        action_journal_index_path(store),
        session_index_path(store),
        artifact_index_path(store),
        artifact_pack_index_path(store),
        workspace_index_path(store),
        recipe_index_path(store),
        conversation_index_path(store),
    ];
    for path in required_indexes {
        if !path.exists() {
            issues.push(StateHealthIssue {
                path: path.display().to_string(),
                issue_type: StateIssueType::MissingIndex,
                severity: Severity::Warning,
                message: "index is missing".to_string(),
                repair_available: true,
            });
            if repair {
                save_json(&path, &serde_json::json!({}))?;
            }
        }
    }
    check_json(
        "project registry",
        crate::agency::registry_path(store),
        &mut issues,
    );
    check_json(
        "goal index",
        store.paths.indexes.join("goal_index.json"),
        &mut issues,
    );
    check_json(
        "route efficiency",
        store.paths.indexes.join("route_efficiency.json"),
        &mut issues,
    );
    check_json(
        "benchmark history",
        store.paths.indexes.join("benchmark_history.json"),
        &mut issues,
    );
    check_json(
        "habit index",
        store.paths.indexes.join("habit_index.json"),
        &mut issues,
    );
    check_json(
        "plan cache index",
        store.paths.indexes.join("plan_cache_index.json"),
        &mut issues,
    );
    check_json(
        "template cache index",
        store.paths.indexes.join("template_cache_index.json"),
        &mut issues,
    );
    check_json(
        "journal index",
        action_journal_index_path(store),
        &mut issues,
    );
    check_json("session index", session_index_path(store), &mut issues);
    check_json("artifact index", artifact_index_path(store), &mut issues);
    check_json(
        "artifact pack index",
        artifact_pack_index_path(store),
        &mut issues,
    );
    check_json("workspace index", workspace_index_path(store), &mut issues);
    check_json("recipe index", recipe_index_path(store), &mut issues);
    check_json(
        "conversation index",
        conversation_index_path(store),
        &mut issues,
    );
    let personality = personality_path(store);
    if !personality.exists() {
        issues.push(StateHealthIssue {
            path: personality.display().to_string(),
            issue_type: StateIssueType::MissingFile,
            severity: Severity::Warning,
            message: "personality config is missing".to_string(),
            repair_available: true,
        });
        if repair {
            let _ = save_personality(store, &PersonalityProfile::Balanced);
        }
    } else {
        check_json("personality config", personality, &mut issues);
    }

    if let Ok(registry) = load_project_registry(store) {
        for project in registry.projects {
            let state = store
                .paths
                .projects
                .join(&project.goal_id)
                .join("project_state.json");
            if !state.exists() {
                issues.push(StateHealthIssue {
                    path: state.display().to_string(),
                    issue_type: StateIssueType::MissingFile,
                    severity: Severity::Error,
                    message: "registered project has no project_state.json".to_string(),
                    repair_available: false,
                });
            }
        }
    }

    if let Ok(index) = load_artifact_index(store) {
        for artifact in index.artifacts.into_iter().take(256) {
            let path = PathBuf::from(&artifact.path);
            if !path.exists() {
                issues.push(StateHealthIssue {
                    path: artifact.path,
                    issue_type: StateIssueType::BrokenReference,
                    severity: Severity::Warning,
                    message: "artifact index references a missing file".to_string(),
                    repair_available: true,
                });
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "artifact_manifest.json")
            {
                check_json("artifact manifest", path, &mut issues);
            }
        }
    }

    if let Ok(sessions) = load_session_index(store) {
        for session in sessions
            .sessions
            .into_iter()
            .filter(|session| session.title.contains("autonomous worker"))
            .take(128)
        {
            let report = store
                .paths
                .sessions
                .join(&session.session_id)
                .join("session_report.json");
            if !report.exists() {
                issues.push(StateHealthIssue {
                    path: report.display().to_string(),
                    issue_type: StateIssueType::MissingFile,
                    severity: Severity::Warning,
                    message: "completed autonomous session is missing session_report.json"
                        .to_string(),
                    repair_available: false,
                });
            }
        }
    }

    if let Ok(index) = load_artifact_pack_index(store) {
        for pack in index.packs.into_iter().take(128) {
            let path = PathBuf::from(&pack.manifest_path);
            if !path.exists() {
                issues.push(StateHealthIssue {
                    path: pack.manifest_path,
                    issue_type: StateIssueType::BrokenReference,
                    severity: Severity::Warning,
                    message: "artifact pack index references a missing pack manifest".to_string(),
                    repair_available: true,
                });
            } else {
                check_json("artifact pack manifest", path, &mut issues);
            }
        }
    }

    if let Ok(index) = load_workspace_index(store) {
        for workspace in index.workspaces.into_iter().take(128) {
            let manifest = PathBuf::from(&workspace.root_path).join("manifest.json");
            if !manifest.exists() {
                issues.push(StateHealthIssue {
                    path: manifest.display().to_string(),
                    issue_type: StateIssueType::MissingFile,
                    severity: Severity::Warning,
                    message: "workspace is missing manifest.json".to_string(),
                    repair_available: false,
                });
            }
        }
    }

    if repair {
        archive_corrupt_json(store, &mut issues)?;
        let _ = load_action_journal_index(store);
        let _ = load_session_index(store);
        let _ = load_artifact_index(store);
        let _ = load_artifact_pack_index(store);
        let _ = load_workspace_index(store);
        let _ = load_route_efficiency(store);
        let _ = load_conversation_index(store);
    }
    let critical = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Critical)
        .count();
    let warnings = issues
        .iter()
        .filter(|issue| issue.severity == Severity::Warning)
        .count();
    let repair_available = issues.iter().filter(|issue| issue.repair_available).count();
    let repaired = if repair { repair_available } else { 0 };
    let recommendation = if critical > 0 {
        "run doctor --repair and review archived corrupt state".to_string()
    } else if issues.is_empty() {
        "state health looks good".to_string()
    } else if repair_available > 0 && !repair {
        "run doctor --repair".to_string()
    } else {
        "review warnings".to_string()
    };
    let reliability_state_health = if critical > 0 {
        0.2
    } else if issues.is_empty() {
        1.0
    } else {
        0.8
    };
    let report_path = store
        .paths
        .logs
        .join(format!("doctor_report_{}.json", timestamp_slug()));
    let report = DoctorReport {
        issues_found: issues.len(),
        issues,
        critical,
        warnings,
        repair_available,
        repaired,
        recommendation,
        reliability_state_health,
        created_at: Utc::now(),
        report_path: report_path.display().to_string(),
    };
    save_json(&report_path, &report)?;
    Ok(report)
}

fn check_json(label: &str, path: PathBuf, issues: &mut Vec<StateHealthIssue>) {
    if !path.exists() {
        return;
    }
    let result = load_json::<serde_json::Value>(&path);
    if let Err(error) = result {
        issues.push(StateHealthIssue {
            path: path.display().to_string(),
            issue_type: StateIssueType::CorruptJson,
            severity: Severity::Critical,
            message: format!("{label} is corrupt: {error}"),
            repair_available: true,
        });
    }
}

fn archive_corrupt_json(store: &DiskStore, issues: &mut [StateHealthIssue]) -> Result<()> {
    let archive = store.paths.recovery.join("corrupt_archive");
    fs::create_dir_all(&archive)?;
    for issue in issues
        .iter()
        .filter(|issue| issue.issue_type == StateIssueType::CorruptJson)
    {
        let path = PathBuf::from(&issue.path);
        if path.exists() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("corrupt.json");
            let dest = archive.join(format!("{}_{}", timestamp_slug(), name));
            fs::rename(&path, dest)?;
            save_json(&path, &replacement_json_for(&path))?;
        }
    }
    Ok(())
}

fn replacement_json_for(path: &std::path::Path) -> serde_json::Value {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    match name {
        "project_registry.json" => serde_json::json!({ "projects": [] }),
        "goal_index.json" => serde_json::json!({ "goals": [] }),
        "action_journal_index.json" => serde_json::json!({ "entries": [] }),
        "session_index.json" => serde_json::json!({ "sessions": [] }),
        "benchmark_history.json" => serde_json::json!([]),
        "route_efficiency.json" => serde_json::json!({ "routes": {} }),
        "habit_index.json" => serde_json::json!({ "habits": {} }),
        "plan_cache_index.json" => serde_json::json!({ "plans": {} }),
        "template_cache_index.json" => serde_json::json!({ "templates": {} }),
        "artifact_index.json" => serde_json::json!({ "artifacts": [] }),
        "conversation_index.json" => serde_json::json!({ "sessions": [] }),
        "personality.json" => serde_json::json!("Balanced"),
        "artifact_manifest.json" => serde_json::json!({
            "session_id": "recovered",
            "goal": "recovered corrupt manifest",
            "artifacts": [],
            "validation_score": 0.0,
            "validation_passed": false,
            "repairs_performed": 0,
            "created_at": chrono::Utc::now()
        }),
        _ => serde_json::json!({}),
    }
}
