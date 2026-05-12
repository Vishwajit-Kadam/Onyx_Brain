use anyhow::Result;
use serde_json::Value;
use std::fs;

use crate::{
    agency::ProjectState,
    core::{RouteTrace, Synapse, SynapseType},
    learning::pruning::should_prune,
    memory::{dedup::dedup_memories, MemoryItem, MemoryType},
    sleep::ConsolidationReport,
    storage::{load_json, DiskStore},
    utils::time::timestamp_slug,
};

pub fn consolidate(store: &DiskStore) -> Result<ConsolidationReport> {
    store.ensure_layout()?;
    let logs = store.list_log_files()?;
    let mut successful_paths = Vec::new();
    for log in &logs {
        let value: Value = load_json(log)?;
        if let Ok(trace) = serde_json::from_value::<RouteTrace>(value.clone()) {
            if trace.success {
                successful_paths.extend(trace.activated_synapses);
            }
        } else if value
            .get("result")
            .and_then(Value::as_str)
            .is_some_and(|result| result == "Success")
        {
            if let Some(ids) = value.get("used_synapses").and_then(Value::as_array) {
                successful_paths.extend(ids.iter().filter_map(Value::as_str).map(str::to_string));
            }
        }
    }

    let mut strengthened_routes = 0;
    successful_paths.sort();
    successful_paths.dedup();
    for id in successful_paths {
        if let Ok(mut synapse) = store.load_synapse(&id) {
            synapse.success_score += 0.5;
            synapse.confidence = (synapse.confidence + 0.02).clamp(0.0, 1.0);
            store.save_synapse(&synapse)?;
            strengthened_routes += 1;
        }
    }

    let mut pruned_synapses = 0;
    for path in store.synapse_files()? {
        let synapse: Synapse = load_json(&path)?;
        if should_prune(&synapse) {
            fs::remove_file(&path)?;
            pruned_synapses += 1;
        }
    }

    consolidate_project_memories(store)?;
    let _ = dedup_memories(store)?;
    archive_old_backups(store)?;
    let shortcut_synapses_created =
        create_seed_shortcut_if_repeated(store)? + create_skill_shortcuts(store)?;
    let report_name = format!("consolidation_report_{}", timestamp_slug());
    let mut report = ConsolidationReport {
        logs_seen: logs.len(),
        strengthened_routes,
        pruned_synapses,
        shortcut_synapses_created,
        report_path: String::new(),
    };
    report.report_path = store
        .paths
        .logs
        .join(format!("{report_name}.json"))
        .display()
        .to_string();
    store.save_log(&report_name, &report)?;
    Ok(report)
}

fn create_skill_shortcuts(store: &DiskStore) -> Result<usize> {
    let routes = [
        (
            "shortcut_task_code_skill_create_rust_cli_project",
            "task_code",
            "skill_create_rust_cli_project",
        ),
        (
            "shortcut_skill_create_rust_cli_project_tool_rust_project",
            "skill_create_rust_cli_project",
            "tool_rust_project",
        ),
        (
            "shortcut_skill_add_rust_unit_tests_tool_code_editor",
            "skill_add_rust_unit_tests",
            "tool_code_editor",
        ),
        (
            "shortcut_skill_run_cargo_check_and_cargo_test_tool_terminal",
            "skill_run_cargo_check_and_cargo_test",
            "tool_terminal",
        ),
    ];
    let mut created = 0;
    for (id, from, to) in routes {
        if store.synapse_path(id).exists() {
            continue;
        }
        let mut synapse = Synapse::new(id, from, to, SynapseType::Shortcut, 0.3);
        synapse.confidence = 0.3;
        store.save_synapse(&synapse)?;
        let mut outgoing = store.read_outgoing_synapse_ids(from)?;
        outgoing.push(id.to_string());
        outgoing.sort();
        outgoing.dedup();
        store.write_outgoing_synapse_ids(from, &outgoing)?;
        created += 1;
    }
    Ok(created)
}

fn archive_old_backups(store: &DiskStore) -> Result<()> {
    let sandbox_projects = store.paths.sandbox.join("projects");
    if !sandbox_projects.exists() {
        return Ok(());
    }
    let mut backups = Vec::new();
    collect_backups(&sandbox_projects, &mut backups)?;
    backups.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|meta| meta.modified()).ok();
        let b_time = b.metadata().and_then(|meta| meta.modified()).ok();
        b_time.cmp(&a_time)
    });
    for backup in backups.into_iter().skip(20) {
        let archive_dir = store.paths.projects.join("archived_backups");
        fs::create_dir_all(&archive_dir)?;
        if let Some(name) = backup.file_name() {
            let target = archive_dir.join(name);
            if target.exists() {
                fs::remove_file(&backup)?;
            } else {
                fs::rename(&backup, target)?;
            }
        }
    }
    Ok(())
}

fn collect_backups(dir: &std::path::Path, backups: &mut Vec<std::path::PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_backups(&path, backups)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains(".bak."))
        {
            backups.push(path);
        }
    }
    Ok(())
}

fn consolidate_project_memories(store: &DiskStore) -> Result<()> {
    if !store.paths.projects.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&store.paths.projects)? {
        let state_path = entry?.path().join("project_state.json");
        if !state_path.exists() {
            continue;
        }
        let state: ProjectState = load_json(&state_path)?;
        if state.status == "Completed" {
            let mut memory = MemoryItem::new(
                format!("procedural_project_workflow_{}", state.project_name),
                MemoryType::Procedural,
                format!("Workflow for {}", state.project_name),
                format!(
                    "Successful project workflow. Commands: {}. Files: {}.",
                    state.commands_run.join(", "),
                    state.files_created.join(", ")
                ),
                vec![
                    "project".to_string(),
                    "workflow".to_string(),
                    "rust".to_string(),
                    "success".to_string(),
                ],
                vec!["goal_create_project".to_string()],
            );
            memory.importance = 0.85;
            store.save_memory(&memory)?;
        } else if !state.errors_seen.is_empty() {
            let mut memory = MemoryItem::new(
                format!("diagnostic_project_failure_{}", state.goal_id),
                MemoryType::Episodic,
                format!("Failure diagnostics for {}", state.project_name),
                state.errors_seen.join("; "),
                vec![
                    "diagnostic".to_string(),
                    "failure".to_string(),
                    state.project_name,
                ],
                vec!["goal_create_project".to_string()],
            );
            memory.importance = 0.7;
            store.save_memory(&memory)?;
        }
    }
    Ok(())
}

fn create_seed_shortcut_if_repeated(store: &DiskStore) -> Result<usize> {
    let id = "shortcut_task_code_tool_rust_project";
    if store.synapse_path(id).exists()
        || !store.neuron_path("task_code").exists()
        || !store.neuron_path("tool_rust_project").exists()
    {
        return Ok(0);
    }
    let mut synapse = Synapse::new(
        id,
        "task_code",
        "tool_rust_project",
        SynapseType::Shortcut,
        0.25,
    );
    synapse.confidence = 0.25;
    store.save_synapse(&synapse)?;
    let mut outgoing = store.read_outgoing_synapse_ids("task_code")?;
    outgoing.push(id.to_string());
    outgoing.sort();
    outgoing.dedup();
    store.write_outgoing_synapse_ids("task_code", &outgoing)?;
    Ok(1)
}
