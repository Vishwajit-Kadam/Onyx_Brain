use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use uuid::Uuid;

use crate::{
    agency::{complete_journal_entry, fail_journal_entry, start_journal_entry, ActionType},
    storage::{save_json, DiskStore},
    utils::{errors::OnyxError, time::timestamp_slug},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    Started,
    Committed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditTransaction {
    pub transaction_id: String,
    pub project_id: Option<String>,
    pub file_path: String,
    pub backup_path: String,
    pub temp_path: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: TransactionStatus,
    pub diff_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionOverview {
    pub transactions: Vec<String>,
    pub count: usize,
}

pub fn transactional_write_project_file(
    sandbox: &Path,
    project: &str,
    path: &str,
    content: &str,
) -> Result<PathBuf> {
    if content.contains('\0') {
        return Err(anyhow::anyhow!("refusing to write non-text content"));
    }
    let store = store_from_sandbox(sandbox)?;
    store.ensure_layout()?;
    let relative = project_file(project, path)?;
    let target = safe_path(sandbox, &relative)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let transaction_id = format!("tx_{}_{}", timestamp_slug(), Uuid::new_v4());
    let tx_dir = store.paths.transactions.join(&transaction_id);
    fs::create_dir_all(&tx_dir)?;
    let backup_path = tx_dir.join("backup");
    let temp_path = tx_dir.join("temp");
    let project_backup_path = target.with_file_name(format!(
        "{}.bak.{}",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file"),
        timestamp_slug()
    ));
    let mut transaction = FileEditTransaction {
        transaction_id: transaction_id.clone(),
        project_id: Some(project.to_string()),
        file_path: target.display().to_string(),
        backup_path: backup_path.display().to_string(),
        temp_path: temp_path.display().to_string(),
        started_at: Utc::now(),
        completed_at: None,
        status: TransactionStatus::Started,
        diff_summary: format!("write {path} ({} bytes)", content.len()),
    };
    save_transaction(&store, &transaction)?;
    let mut journal = start_journal_entry(
        &store,
        "system",
        if target.exists() {
            ActionType::ModifyFile
        } else {
            ActionType::CreateFile
        },
        None,
        Some(project.to_string()),
        Some(target.display().to_string()),
        None,
        target.exists().then(|| backup_path.display().to_string()),
        serde_json::json!({ "transaction_id": transaction_id }),
    )?;
    let result = (|| -> Result<()> {
        if target.exists() {
            fs::copy(&target, &backup_path)?;
            fs::copy(&target, &project_backup_path)?;
        }
        fs::write(&temp_path, content)?;
        let validated = fs::read_to_string(&temp_path)
            .with_context(|| format!("validating {}", temp_path.display()))?;
        if validated.contains('\0') {
            return Err(anyhow::anyhow!("temp file failed text validation"));
        }
        match fs::rename(&temp_path, &target) {
            Ok(()) => {}
            Err(_) => {
                fs::write(&target, validated)?;
                let _ = fs::remove_file(&temp_path);
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            transaction.status = TransactionStatus::Committed;
            transaction.completed_at = Some(Utc::now());
            save_transaction(&store, &transaction)?;
            let rollback_ref = backup_path
                .exists()
                .then(|| backup_path.display().to_string());
            complete_journal_entry(
                &store,
                &mut journal,
                Some(target.display().to_string()),
                rollback_ref,
            )?;
            Ok(target)
        }
        Err(error) => {
            if backup_path.exists() {
                let _ = fs::copy(&backup_path, &target);
            }
            transaction.status = TransactionStatus::Failed;
            transaction.completed_at = Some(Utc::now());
            transaction.diff_summary = format!("failed write {path}: {error}");
            save_transaction(&store, &transaction)?;
            fail_journal_entry(&store, &mut journal, error.to_string())?;
            Err(error)
        }
    }
}

pub fn transactions(store: &DiskStore) -> Result<TransactionOverview> {
    let mut rows = Vec::new();
    if store.paths.transactions.exists() {
        for entry in fs::read_dir(&store.paths.transactions)? {
            let path = entry?.path().join("transaction.json");
            if path.exists() {
                if let Ok(tx) = crate::storage::load_json::<FileEditTransaction>(&path) {
                    rows.push(format!(
                        "{} | {:?} | {} | {}",
                        tx.transaction_id, tx.status, tx.file_path, tx.diff_summary
                    ));
                }
            }
        }
    }
    rows.sort();
    rows.reverse();
    Ok(TransactionOverview {
        count: rows.len(),
        transactions: rows,
    })
}

pub fn transaction_count(store: &DiskStore) -> Result<usize> {
    Ok(transactions(store)?.count)
}

pub fn save_transaction(store: &DiskStore, tx: &FileEditTransaction) -> Result<()> {
    save_json(
        &store
            .paths
            .transactions
            .join(&tx.transaction_id)
            .join("transaction.json"),
        tx,
    )
}

pub fn simulate_failed_transaction(
    sandbox: &Path,
    project: &str,
    path: &str,
) -> Result<FileEditTransaction> {
    let store = store_from_sandbox(sandbox)?;
    let target = safe_path(sandbox, &project_file(project, path)?)?;
    let before = fs::read_to_string(&target).unwrap_or_default();
    let _ = transactional_write_project_file(sandbox, project, path, &before)?;
    let mut tx = FileEditTransaction {
        transaction_id: format!("tx_failed_{}_{}", timestamp_slug(), Uuid::new_v4()),
        project_id: Some(project.to_string()),
        file_path: target.display().to_string(),
        backup_path: target.display().to_string(),
        temp_path: target.display().to_string(),
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
        status: TransactionStatus::Failed,
        diff_summary: "simulated failure restored backup".to_string(),
    };
    save_transaction(&store, &tx)?;
    tx.status = TransactionStatus::Failed;
    Ok(tx)
}

fn store_from_sandbox(sandbox: &Path) -> Result<DiskStore> {
    let sandbox = sandbox.canonicalize()?;
    let root = sandbox
        .parent()
        .ok_or_else(|| anyhow::anyhow!("sandbox has no parent"))?;
    Ok(DiskStore::new(root))
}

fn safe_path(sandbox: &Path, relative: &str) -> Result<PathBuf> {
    let sandbox = sandbox.canonicalize()?;
    let rel = Path::new(relative);
    if rel.is_absolute() {
        return Err(OnyxError::SandboxEscape.into());
    }
    let mut clean = PathBuf::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            _ => return Err(OnyxError::SandboxEscape.into()),
        }
    }
    Ok(sandbox.join(clean))
}

pub fn project_file(project: &str, path: &str) -> Result<String> {
    if project.contains("..") || path.contains("..") || Path::new(project).is_absolute() {
        return Err(OnyxError::SandboxEscape.into());
    }
    if Path::new(path).is_absolute() {
        return Err(OnyxError::SandboxEscape.into());
    }
    Ok(format!("projects/{project}/{path}"))
}
