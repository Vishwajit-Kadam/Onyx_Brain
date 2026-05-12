use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use uuid::Uuid;

use crate::{
    agency::{
        latest_journal_entries, mark_journal_rolled_back, snapshot_restore, ActionJournalSummary,
        ActionStatus,
    },
    storage::{save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackReport {
    pub rollback_id: String,
    pub target_entry_id: String,
    pub project_name: Option<String>,
    pub actions_reverted: u64,
    pub files_restored: u64,
    pub status: String,
    pub errors: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub report_path: String,
}

pub fn rollback_latest(
    store: &DiskStore,
    project_name_filter: Option<&str>,
) -> Result<RollbackReport> {
    let entries = latest_journal_entries(store, 128, None)?;
    let Some(entry) = entries.into_iter().find(|entry| {
        entry.rollback_available
            && entry.status != ActionStatus::RolledBack
            && project_name_filter.is_none_or(|project| {
                entry
                    .target_path
                    .as_ref()
                    .is_some_and(|target| target.contains(project))
                    || entry.project_id.as_deref() == Some(project)
            })
    }) else {
        return Err(anyhow::anyhow!("no rollback-capable journal entry found"));
    };
    rollback_entry(store, &entry, project_name_filter)
}

fn rollback_entry(
    store: &DiskStore,
    entry: &ActionJournalSummary,
    project_name_filter: Option<&str>,
) -> Result<RollbackReport> {
    let rollback_id = format!("rollback_{}_{}", timestamp_slug(), Uuid::new_v4());
    let mut errors = Vec::new();
    let mut files_restored = 0;
    if let Some(ref rollback_ref) = entry.rollback_ref {
        if rollback_ref.starts_with("snapshot_") {
            match snapshot_restore(store, rollback_ref) {
                Ok(report) => {
                    files_restored += report.files_restored;
                    errors.extend(report.errors);
                }
                Err(error) => errors.push(error.to_string()),
            }
        } else {
            let backup = PathBuf::from(rollback_ref);
            let sandbox = store
                .paths
                .sandbox
                .canonicalize()
                .unwrap_or_else(|_| store.paths.sandbox.clone());
            let transactions = store
                .paths
                .transactions
                .canonicalize()
                .unwrap_or_else(|_| store.paths.transactions.clone());
            let backup_check = backup.canonicalize().unwrap_or_else(|_| backup.clone());
            if !(backup_check.starts_with(&sandbox) || backup_check.starts_with(&transactions)) {
                errors.push("refusing to rollback from outside sandbox".to_string());
            } else if let Some(target) = &entry.target_path {
                let target = PathBuf::from(target);
                let target_check = target.canonicalize().unwrap_or_else(|_| target.clone());
                if !target_check.starts_with(&sandbox) {
                    errors.push("refusing to rollback target outside sandbox".to_string());
                } else if project_name_filter
                    .is_some_and(|project| !target.to_string_lossy().contains(project))
                {
                    errors.push("rollback target does not match requested project".to_string());
                } else {
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::copy(&backup, &target)?;
                    files_restored += 1;
                }
            }
        }
    }
    let status = if errors.is_empty() {
        "Completed"
    } else {
        "Failed"
    }
    .to_string();
    if errors.is_empty() {
        mark_journal_rolled_back(store, entry)?;
    }
    let report_path = store
        .paths
        .logs
        .join(format!("rollback_report_{}.json", timestamp_slug()));
    let report = RollbackReport {
        rollback_id,
        target_entry_id: entry.id.clone(),
        project_name: project_name_filter.map(ToOwned::to_owned),
        actions_reverted: u64::from(errors.is_empty()),
        files_restored,
        status,
        errors,
        created_at: Utc::now(),
        report_path: report_path.display().to_string(),
    };
    save_json(&report_path, &report)?;
    if report.status == "Failed" {
        Err(anyhow::anyhow!(
            "rollback refused or failed: {}",
            report.errors.join("; ")
        ))
    } else {
        Ok(report)
    }
}
