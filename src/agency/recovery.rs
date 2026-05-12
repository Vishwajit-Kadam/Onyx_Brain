use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    agency::{latest_journal_entries, ActionStatus},
    storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureKind {
    CargoCheckFailure,
    CargoTestFailure,
    FileEditFailure,
    MissingProject,
    CorruptState,
    SandboxViolation,
    ToolDenied,
    Timeout,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub recovery_id: String,
    pub failure_kind: FailureKind,
    pub project_name: Option<String>,
    pub failed_task_id: Option<String>,
    pub suggested_steps: Vec<String>,
    pub safe_to_auto_run: bool,
    pub requires_user_review: bool,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub plan: RecoveryPlan,
    pub executed: bool,
    pub status: String,
}

pub fn classify_failure(text: &str) -> FailureKind {
    let text = text.to_ascii_lowercase();
    if text.contains("cargo check") || text.contains("could not compile") {
        FailureKind::CargoCheckFailure
    } else if text.contains("cargo test") || text.contains("panicked") {
        FailureKind::CargoTestFailure
    } else if text.contains("sandbox") || text.contains("path traversal") {
        FailureKind::SandboxViolation
    } else if text.contains("not allowed") {
        FailureKind::ToolDenied
    } else if text.contains("json") || text.contains("parsing") {
        FailureKind::CorruptState
    } else if text.contains("project not found") {
        FailureKind::MissingProject
    } else {
        FailureKind::Unknown
    }
}

pub fn recovery_plan_for_failure(
    failure_text: &str,
    project_name: Option<String>,
    task_id: Option<String>,
) -> RecoveryPlan {
    let kind = classify_failure(failure_text);
    let (steps, safe, review, confidence) = match kind {
        FailureKind::CargoCheckFailure => (
            vec![
                "rerun cargo check".to_string(),
                "apply deterministic Rust fix if diagnostic is supported".to_string(),
                "mark task blocked if fix is not obvious".to_string(),
            ],
            true,
            false,
            0.75,
        ),
        FailureKind::CargoTestFailure => (
            vec![
                "rerun cargo test".to_string(),
                "inspect failing deterministic calculator test".to_string(),
                "restore snapshot if test failure follows risky edit".to_string(),
            ],
            true,
            false,
            0.7,
        ),
        FailureKind::CorruptState => (
            vec![
                "run doctor --repair".to_string(),
                "archive corrupt JSON".to_string(),
            ],
            true,
            false,
            0.8,
        ),
        FailureKind::SandboxViolation | FailureKind::ToolDenied => (
            vec![
                "refuse unsafe action".to_string(),
                "request user review".to_string(),
            ],
            false,
            true,
            0.95,
        ),
        _ => (
            vec![
                "create snapshot if possible".to_string(),
                "request user review".to_string(),
            ],
            false,
            true,
            0.4,
        ),
    };
    RecoveryPlan {
        recovery_id: format!("recovery_{}", Uuid::new_v4()),
        failure_kind: kind,
        project_name,
        failed_task_id: task_id,
        suggested_steps: steps,
        safe_to_auto_run: safe,
        requires_user_review: review,
        confidence,
    }
}

pub fn recover_latest(store: &DiskStore, project_filter: Option<&str>) -> Result<RecoveryResult> {
    let failed = latest_journal_entries(store, 64, None)?
        .into_iter()
        .find(|entry| {
            entry.status == ActionStatus::Failed
                && project_filter.is_none_or(|project| {
                    entry
                        .target_path
                        .as_ref()
                        .is_some_and(|target| target.contains(project))
                })
        });
    let plan = if let Some(entry) = failed {
        recovery_plan_for_failure(
            entry
                .command
                .as_deref()
                .or(entry.target_path.as_deref())
                .unwrap_or("unknown failure"),
            project_filter.map(ToOwned::to_owned),
            Some(entry.id),
        )
    } else {
        recovery_plan_for_failure(
            "unknown failure",
            project_filter.map(ToOwned::to_owned),
            None,
        )
    };
    let executed = plan.safe_to_auto_run && !plan.requires_user_review;
    Ok(RecoveryResult {
        status: if executed {
            "safe recovery plan prepared".to_string()
        } else {
            "manual review required".to_string()
        },
        executed,
        plan,
    })
}
