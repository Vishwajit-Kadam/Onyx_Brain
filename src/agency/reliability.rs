use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    agency::{journal_count, snapshot_count},
    storage::{doctor, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReliabilityScore {
    pub rollback_readiness: f32,
    pub snapshot_coverage: f32,
    pub journal_completeness: f32,
    pub state_health: f32,
    pub recovery_confidence: f32,
    pub test_success: f32,
    pub overall: f32,
    pub notes: Vec<String>,
}

pub fn reliability_score(
    store: &DiskStore,
    tests_passed: bool,
    recovery_confidence: f32,
) -> Result<ReliabilityScore> {
    let journals = journal_count(store).unwrap_or(0);
    let snapshots = snapshot_count(store).unwrap_or(0);
    let doctor = doctor(store, false)?;
    let rollback_readiness = if snapshots > 0 { 1.0 } else { 0.55 };
    let snapshot_coverage = if snapshots > 0 { 1.0 } else { 0.5 };
    let journal_completeness = if journals > 0 { 1.0 } else { 0.5 };
    let state_health = doctor.reliability_state_health;
    let test_success = if tests_passed { 1.0 } else { 0.45 };
    let overall = rollback_readiness * 0.2
        + snapshot_coverage * 0.2
        + journal_completeness * 0.2
        + state_health * 0.15
        + recovery_confidence * 0.1
        + test_success * 0.15;
    let mut notes = Vec::new();
    if journals == 0 {
        notes.push("no journal entries recorded yet".to_string());
    }
    if snapshots == 0 {
        notes.push("no snapshots available yet".to_string());
    }
    if doctor.critical > 0 {
        notes.push("doctor found critical state issues".to_string());
    }
    Ok(ReliabilityScore {
        rollback_readiness,
        snapshot_coverage,
        journal_completeness,
        state_health,
        recovery_confidence,
        test_success,
        overall,
        notes,
    })
}
