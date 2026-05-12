use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    energy::load_performance_index, memory::dedup::inspect_memory_hygiene, storage::DiskStore,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoOptimizeHint {
    pub should_optimize: bool,
    pub reason: String,
    pub recommended_command: String,
}

pub fn auto_optimize_hint(
    store: &DiskStore,
    irrelevant_skills_used: usize,
    habit_missed: bool,
) -> Result<AutoOptimizeHint> {
    let profiles = load_performance_index(store)?;
    let hygiene = inspect_memory_hygiene(store)?;
    if profiles.profiles.len() >= 10 {
        return Ok(hint(
            "profile threshold reached; refresh habits and route efficiency",
            "cargo run -- optimize",
        ));
    }
    if irrelevant_skills_used > 0 {
        return Ok(hint(
            "irrelevant skills were reused; optimize can penalize them",
            "cargo run -- optimize",
        ));
    }
    if hygiene.duplicate_groups > 0 {
        return Ok(hint(
            "duplicate memories detected",
            "cargo run -- memory-dedup",
        ));
    }
    if habit_missed {
        return Ok(hint(
            "matching task completed without a habit; optimize can form one",
            "cargo run -- optimize",
        ));
    }
    Ok(AutoOptimizeHint {
        should_optimize: false,
        reason: "no immediate optimization needed".to_string(),
        recommended_command: "none".to_string(),
    })
}

pub fn lightweight_auto_optimize(store: &DiskStore) -> Result<AutoOptimizeHint> {
    let profiles = load_performance_index(store)?;
    if profiles.profiles.len() >= 5 {
        Ok(hint(
            "maintenance refreshed optimization signals",
            "cargo run -- brain-status",
        ))
    } else {
        Ok(AutoOptimizeHint {
            should_optimize: false,
            reason: "not enough profiles for maintenance optimization".to_string(),
            recommended_command: "none".to_string(),
        })
    }
}

fn hint(reason: &str, command: &str) -> AutoOptimizeHint {
    AutoOptimizeHint {
        should_optimize: true,
        reason: reason.to_string(),
        recommended_command: command.to_string(),
    }
}
