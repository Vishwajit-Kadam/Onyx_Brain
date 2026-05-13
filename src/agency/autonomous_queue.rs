use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    agency::{AutonomyLevel, WorkerStatus},
    core::brain::Brain,
    storage::{save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueueRunReport {
    pub goals_total: usize,
    pub goals_completed: usize,
    pub goals_failed: usize,
    pub safety_stops: usize,
    pub artifact_packs_created: usize,
    pub report_path: String,
}

pub fn split_queue_goals(input: &str) -> Vec<String> {
    input
        .split("||")
        .map(str::trim)
        .filter(|goal| !goal.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn save_queue_report(_store: &DiskStore, report: &QueueRunReport) -> Result<()> {
    save_json(&std::path::PathBuf::from(&report.report_path), report)
}

pub fn run_queue(brain: &Brain, input: &str) -> Result<QueueRunReport> {
    let goals = split_queue_goals(input);
    let mut completed = 0;
    let mut failed = 0;
    let mut safety_stops = 0;
    let mut packs = 0;
    for goal in &goals {
        if goal.to_lowercase().contains("run doctor") {
            let doctor = brain.doctor(false)?;
            if doctor.critical == 0 {
                completed += 1;
            } else {
                failed += 1;
            }
            continue;
        }
        let output = brain.autonomize(goal.clone(), AutonomyLevel::FullBounded)?;
        if output.status == WorkerStatus::SafetyStopped {
            safety_stops += 1;
            failed += 1;
        } else if output.validation_passed {
            completed += 1;
        } else {
            failed += 1;
        }
        packs += output
            .artifacts_created
            .iter()
            .filter(|path| path.ends_with("artifact_pack.json"))
            .count();
        if safety_stops > 0 {
            break;
        }
    }
    let report_path = brain
        .store()
        .paths
        .logs
        .join(format!("queue_run_{}.json", timestamp_slug()))
        .display()
        .to_string();
    let report = QueueRunReport {
        goals_total: goals.len(),
        goals_completed: completed,
        goals_failed: failed,
        safety_stops,
        artifact_packs_created: packs,
        report_path,
    };
    save_queue_report(brain.store(), &report)?;
    Ok(report)
}
