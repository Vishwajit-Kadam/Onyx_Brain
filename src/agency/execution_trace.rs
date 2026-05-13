use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub timestamp: DateTime<Utc>,
    pub phase: String,
    pub action: String,
    pub status: String,
    pub output_summary: String,
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub trace_id: String,
    pub session_id: String,
    pub goal: String,
    pub events: Vec<ExecutionEvent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionTraceIndex {
    #[serde(default)]
    pub traces: Vec<ExecutionTraceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTraceSummary {
    pub trace_id: String,
    pub session_id: String,
    pub goal: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
}

pub fn new_execution_trace(session_id: &str, goal: &str) -> ExecutionTrace {
    let now = Utc::now();
    ExecutionTrace {
        trace_id: format!("trace_{}_{}", timestamp_slug(), uuid::Uuid::new_v4()),
        session_id: session_id.to_string(),
        goal: goal.to_string(),
        events: Vec::new(),
        created_at: now,
        updated_at: now,
    }
}

pub fn push_trace_event(
    trace: &mut ExecutionTrace,
    phase: &str,
    action: &str,
    status: &str,
    output_summary: &str,
    artifact_refs: Vec<String>,
) {
    trace.events.push(ExecutionEvent {
        timestamp: Utc::now(),
        phase: phase.to_string(),
        action: action.to_string(),
        status: status.to_string(),
        output_summary: output_summary.to_string(),
        artifact_refs,
    });
    trace.updated_at = Utc::now();
}

pub fn save_execution_trace(store: &DiskStore, trace: &ExecutionTrace) -> Result<()> {
    let traces_dir = store.paths.data.join("traces");
    fs::create_dir_all(&traces_dir)?;
    let path = traces_dir.join(format!("{}_execution_trace.json", trace.session_id));
    save_json(&path, trace)?;

    let report_dir = store
        .paths
        .sandbox
        .join("workspaces")
        .join(&trace.session_id)
        .join("reports");
    fs::create_dir_all(&report_dir)?;
    fs::write(
        report_dir.join("execution_trace.md"),
        render_trace_markdown(trace),
    )?;

    let mut index = load_trace_index(store)?;
    index
        .traces
        .retain(|row| row.session_id != trace.session_id);
    index.traces.push(ExecutionTraceSummary {
        trace_id: trace.trace_id.clone(),
        session_id: trace.session_id.clone(),
        goal: trace.goal.clone(),
        path: path.display().to_string(),
        created_at: trace.created_at,
    });
    index.traces.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if index.traces.len() > 128 {
        index.traces.truncate(128);
    }
    save_json(&trace_index_path(store), &index)?;
    Ok(())
}

pub fn load_execution_trace(store: &DiskStore, selector: &str) -> Result<ExecutionTrace> {
    let index = load_trace_index(store)?;
    let summary = if selector.eq_ignore_ascii_case("latest") {
        index
            .traces
            .first()
            .ok_or_else(|| anyhow::anyhow!("no execution traces found"))?
            .clone()
    } else {
        index
            .traces
            .into_iter()
            .find(|row| row.session_id == selector || row.trace_id == selector)
            .ok_or_else(|| anyhow::anyhow!("execution trace not found"))?
    };
    load_json(&PathBuf::from(summary.path))
}

fn trace_index_path(store: &DiskStore) -> PathBuf {
    store.paths.indexes.join("execution_trace_index.json")
}

fn load_trace_index(store: &DiskStore) -> Result<ExecutionTraceIndex> {
    let path = trace_index_path(store);
    if path.exists() {
        load_json(&path)
    } else {
        Ok(ExecutionTraceIndex::default())
    }
}

fn render_trace_markdown(trace: &ExecutionTrace) -> String {
    let mut out = format!(
        "# Execution Trace\n\nSession: {}\nGoal: {}\n\n",
        trace.session_id, trace.goal
    );
    for event in &trace.events {
        out.push_str(&format!(
            "- {} | {} | {} | {} | {}\n",
            event.timestamp, event.phase, event.action, event.status, event.output_summary
        ));
    }
    out
}
