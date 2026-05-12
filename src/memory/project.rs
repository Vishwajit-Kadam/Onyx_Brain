use anyhow::Result;

use crate::{
    agency::ProjectState,
    memory::{MemoryItem, MemoryType},
    storage::DiskStore,
};

pub fn remember_project_state(store: &DiskStore, state: &ProjectState) -> Result<MemoryItem> {
    let content = format!(
        "Project {} status: {}. Files: {}. Errors: {}. Summary: {}",
        state.project_name,
        state.status,
        state.files_created.join(", "),
        state.errors_seen.join("; "),
        state.final_summary.clone().unwrap_or_default()
    );
    let mut memory = MemoryItem::new(
        format!("project_memory_{}", state.goal_id),
        MemoryType::Project,
        format!("Project {}", state.project_name),
        content,
        vec![
            "project".to_string(),
            state.project_name.clone(),
            state.status.to_lowercase(),
        ],
        vec!["goal_create_project".to_string()],
    );
    memory.importance = if state.status == "Completed" {
        0.8
    } else {
        0.6
    };
    store.save_memory(&memory)?;
    Ok(memory)
}
