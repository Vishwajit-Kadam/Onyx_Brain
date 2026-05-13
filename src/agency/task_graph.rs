use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use crate::{
    agency::{DeliverableKind, GoalUnderstanding},
    storage::{load_json, save_json, DiskStore},
    utils::time::timestamp_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskGraphStatus {
    Planned,
    Running,
    Completed,
    Failed,
    Blocked,
    SafetyStopped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutonomousTaskType {
    UnderstandGoal,
    DiscoverContext,
    Plan,
    GenerateArtifact,
    ValidateArtifact,
    ReviseArtifact,
    RunCheck,
    ExportPackage,
    WriteReport,
    MaintainState,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffortLevel {
    Tiny,
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GraphTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub task_type: AutonomousTaskType,
    pub status: GraphTaskStatus,
    pub required_tools: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub validation_rules: Vec<String>,
    pub priority: i32,
    pub estimated_effort: EffortLevel,
    pub retries: usize,
    pub max_retries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEdge {
    pub from_task_id: String,
    pub to_task_id: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    pub graph_id: String,
    pub session_id: String,
    pub goal_id: Option<String>,
    pub nodes: Vec<TaskNode>,
    pub edges: Vec<TaskEdge>,
    pub status: TaskGraphStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn task_graph_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("task_graph_index.json")
}

pub fn build_from_goal_understanding(
    session_id: &str,
    goal_id: Option<String>,
    understanding: &GoalUnderstanding,
) -> TaskGraph {
    let mut nodes = vec![
        node(
            "understand_goal",
            "Understand goal",
            AutonomousTaskType::UnderstandGoal,
            100,
        ),
        node(
            "discover_context",
            "Discover local context",
            AutonomousTaskType::DiscoverContext,
            80,
        ),
        node("plan_work", "Plan work", AutonomousTaskType::Plan, 90),
    ];
    for deliverable in &understanding.deliverables {
        let id = format!("generate_{}", slug(&deliverable.title));
        nodes.push(TaskNode {
            task_id: id,
            title: format!("Generate {}", deliverable.title),
            description: format!("Create required deliverable in {}", deliverable.format),
            task_type: AutonomousTaskType::GenerateArtifact,
            status: GraphTaskStatus::Pending,
            required_tools: vec!["artifact_writer".to_string()],
            expected_outputs: deliverable.path_hint.clone().into_iter().collect(),
            validation_rules: vec!["file exists".to_string(), "file is non-empty".to_string()],
            priority: if deliverable.required { 80 } else { 40 },
            estimated_effort: if matches!(deliverable.kind, DeliverableKind::Report) {
                EffortLevel::Small
            } else {
                EffortLevel::Medium
            },
            retries: 0,
            max_retries: 2,
        });
    }
    nodes.extend([
        node(
            "validate_artifacts",
            "Validate artifacts",
            AutonomousTaskType::ValidateArtifact,
            95,
        ),
        node(
            "revise_artifacts",
            "Revise artifacts",
            AutonomousTaskType::ReviseArtifact,
            70,
        ),
        node(
            "export_package",
            "Export package",
            AutonomousTaskType::ExportPackage,
            60,
        ),
        node(
            "write_report",
            "Write final report",
            AutonomousTaskType::WriteReport,
            85,
        ),
        node(
            "maintain_state",
            "Maintain state",
            AutonomousTaskType::MaintainState,
            50,
        ),
    ]);
    let mut edges = vec![
        edge("understand_goal", "discover_context", "goal before context"),
        edge("discover_context", "plan_work", "context before planning"),
    ];
    for task in nodes
        .iter()
        .filter(|task| task.task_type == AutonomousTaskType::GenerateArtifact)
    {
        edges.push(edge("plan_work", &task.task_id, "plan before generation"));
        edges.push(edge(
            &task.task_id,
            "validate_artifacts",
            "validate generated artifact",
        ));
    }
    edges.extend([
        edge(
            "validate_artifacts",
            "revise_artifacts",
            "revise after validation",
        ),
        edge(
            "revise_artifacts",
            "export_package",
            "export after revision",
        ),
        edge("export_package", "write_report", "report after export"),
        edge("write_report", "maintain_state", "state after report"),
    ]);
    TaskGraph {
        graph_id: format!("graph_{}_{}", timestamp_slug(), Uuid::new_v4()),
        session_id: session_id.to_string(),
        goal_id,
        nodes,
        edges,
        status: TaskGraphStatus::Planned,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

impl TaskGraph {
    pub fn topological_order(&self) -> Vec<String> {
        let mut incoming: BTreeMap<String, usize> = self
            .nodes
            .iter()
            .map(|node| (node.task_id.clone(), 0))
            .collect();
        for edge in &self.edges {
            *incoming.entry(edge.to_task_id.clone()).or_default() += 1;
        }
        let mut ready = incoming
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        ready.sort();
        ready.reverse();
        let mut order = Vec::new();
        while let Some(id) = ready.pop() {
            order.push(id.clone());
            for edge in self.edges.iter().filter(|edge| edge.from_task_id == id) {
                if let Some(count) = incoming.get_mut(&edge.to_task_id) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        ready.push(edge.to_task_id.clone());
                        ready.sort();
                        ready.reverse();
                    }
                }
            }
        }
        order
    }

    pub fn ready_tasks(&self) -> Vec<&TaskNode> {
        self.nodes
            .iter()
            .filter(|node| node.status == GraphTaskStatus::Pending)
            .filter(|node| {
                self.edges
                    .iter()
                    .filter(|edge| edge.to_task_id == node.task_id)
                    .all(|edge| {
                        self.nodes
                            .iter()
                            .find(|candidate| candidate.task_id == edge.from_task_id)
                            .is_some_and(|dependency| {
                                dependency.status == GraphTaskStatus::Completed
                            })
                    })
            })
            .collect()
    }

    pub fn mark_task_completed(&mut self, task_id: &str) {
        if let Some(task) = self.nodes.iter_mut().find(|task| task.task_id == task_id) {
            task.status = GraphTaskStatus::Completed;
        }
        self.updated_at = Utc::now();
    }

    pub fn mark_task_failed(&mut self, task_id: &str) {
        if let Some(task) = self.nodes.iter_mut().find(|task| task.task_id == task_id) {
            task.status = GraphTaskStatus::Failed;
            task.retries += 1;
        }
        self.updated_at = Utc::now();
    }

    pub fn detect_cycles(&self) -> Vec<String> {
        let ordered = self
            .topological_order()
            .into_iter()
            .collect::<BTreeSet<_>>();
        self.nodes
            .iter()
            .filter(|node| !ordered.contains(&node.task_id))
            .map(|node| format!("cycle includes {}", node.task_id))
            .collect()
    }
}

pub fn save_task_graph(store: &DiskStore, graph: &TaskGraph) -> Result<()> {
    let dir = store.paths.sessions.join(&graph.session_id);
    std::fs::create_dir_all(&dir)?;
    save_json(&dir.join("task_graph.json"), graph)?;
    let mut ids: Vec<String> = if task_graph_index_path(store).exists() {
        load_json(&task_graph_index_path(store))?
    } else {
        Vec::new()
    };
    ids.retain(|id| id != &graph.session_id);
    ids.insert(0, graph.session_id.clone());
    if ids.len() > 128 {
        ids.truncate(128);
    }
    save_json(&task_graph_index_path(store), &ids)
}

pub fn load_task_graph(store: &DiskStore, selector: &str) -> Result<TaskGraph> {
    let session_id = if selector.eq_ignore_ascii_case("latest") {
        let ids: Vec<String> = load_json(&task_graph_index_path(store))?;
        ids.first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no task graphs found"))?
    } else {
        selector.to_string()
    };
    load_json(
        &store
            .paths
            .sessions
            .join(session_id)
            .join("task_graph.json"),
    )
}

fn node(id: &str, title: &str, task_type: AutonomousTaskType, priority: i32) -> TaskNode {
    TaskNode {
        task_id: id.to_string(),
        title: title.to_string(),
        description: title.to_string(),
        task_type,
        status: GraphTaskStatus::Pending,
        required_tools: vec!["local_runtime".to_string()],
        expected_outputs: Vec::new(),
        validation_rules: Vec::new(),
        priority,
        estimated_effort: EffortLevel::Small,
        retries: 0,
        max_retries: 2,
    }
}

fn edge(from: &str, to: &str, relation: &str) -> TaskEdge {
    TaskEdge {
        from_task_id: from.to_string(),
        to_task_id: to.to_string(),
        relation: relation.to_string(),
    }
}

fn slug(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
