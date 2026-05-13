use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agency::{Deliverable, GoalType, GoalUnderstanding};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkPhaseStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub task_type: String,
    pub status: WorkPhaseStatus,
    pub required_tools: Vec<String>,
    pub expected_output: String,
    pub validation_rule: String,
    pub retry_count: usize,
    pub max_retries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPhase {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tasks: Vec<AutonomousTask>,
    pub status: WorkPhaseStatus,
    pub dependencies: Vec<String>,
    pub validation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationPlan {
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousPlan {
    pub plan_id: String,
    pub goal_id: String,
    pub phases: Vec<WorkPhase>,
    pub deliverables: Vec<Deliverable>,
    pub validation_plan: ValidationPlan,
    pub estimated_complexity: ComplexityLevel,
    pub estimated_runtime_minutes: u64,
    pub risk_level: RiskLevel,
}

pub fn plan_autonomous_work(goal_id: &str, understanding: &GoalUnderstanding) -> AutonomousPlan {
    let phases = if understanding.goal_type == GoalType::Presentation {
        presentation_phases()
    } else {
        generic_phases()
    };
    AutonomousPlan {
        plan_id: format!("plan_{}_{}", Utc::now().timestamp(), Uuid::new_v4()),
        goal_id: goal_id.to_string(),
        phases,
        deliverables: understanding.deliverables.clone(),
        validation_plan: ValidationPlan {
            rules: match understanding.goal_type {
                GoalType::Presentation => vec![
                    "correct slide count".to_string(),
                    "all slides include title, bullets, notes, and visual suggestion".to_string(),
                    "manifest and final report exist".to_string(),
                ],
                GoalType::CodeProject => vec![
                    "required files exist".to_string(),
                    "cargo check/test validation when applicable".to_string(),
                    "final report exists".to_string(),
                ],
                _ => vec![
                    "title and sections exist".to_string(),
                    "final report exists".to_string(),
                ],
            },
        },
        estimated_complexity: if understanding.deliverables.len() > 4 {
            ComplexityLevel::Moderate
        } else {
            ComplexityLevel::Simple
        },
        estimated_runtime_minutes: 5,
        risk_level: RiskLevel::Low,
    }
}

fn presentation_phases() -> Vec<WorkPhase> {
    let titles = [
        "parse topic and audience",
        "create slide structure",
        "create slide content",
        "create speaker notes",
        "create design guide",
        "validate slide count and sections",
        "repair missing artifact pieces",
        "write final markdown deck and report",
    ];
    titles
        .iter()
        .enumerate()
        .map(|(index, title)| phase(index + 1, title, "presentation"))
        .collect()
}

fn generic_phases() -> Vec<WorkPhase> {
    let titles = [
        "understand goal",
        "create workspace",
        "gather local context",
        "design solution",
        "create artifacts",
        "validate artifacts",
        "repair issues",
        "final report",
    ];
    titles
        .iter()
        .enumerate()
        .map(|(index, title)| phase(index + 1, title, "generic"))
        .collect()
}

fn phase(number: usize, title: &str, task_type: &str) -> WorkPhase {
    WorkPhase {
        id: format!("phase_{number}"),
        title: title.to_string(),
        description: format!("Autonomous phase: {title}"),
        tasks: vec![AutonomousTask {
            id: format!("task_{number}_1"),
            title: title.to_string(),
            description: format!("Complete phase task: {title}"),
            task_type: task_type.to_string(),
            status: WorkPhaseStatus::Pending,
            required_tools: vec!["filesystem".to_string()],
            expected_output: "phase output recorded".to_string(),
            validation_rule: "required output exists".to_string(),
            retry_count: 0,
            max_retries: 2,
        }],
        status: WorkPhaseStatus::Pending,
        dependencies: if number > 1 {
            vec![format!("phase_{}", number - 1)]
        } else {
            Vec::new()
        },
        validation_steps: vec!["check expected output".to_string()],
    }
}
