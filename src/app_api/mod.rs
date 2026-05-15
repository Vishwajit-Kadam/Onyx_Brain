pub mod events;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{
    agency::{AutonomyLevel, SessionDashboardReport, WorkSessionSummary},
    artifacts::{ArtifactOverview, ArtifactPackOverview},
    conversation::{ConversationMode, ConversationTurnOutput},
    core::brain::{BrainStatus, InspectSummaryLite},
    creative::{create_creative_project, CreativeRunReport},
    executive::{executive_status, ExecutiveStatus},
    memory::MemoryType,
    storage::DoctorReport,
    testing::RegressionCheckReport,
    Brain,
};

pub use events::*;

#[derive(Debug, Clone)]
pub struct AppApi {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SafetyStatus {
    pub sandbox_enabled: bool,
    pub network_default: String,
    pub allowlisted_commands: Vec<String>,
    pub safety_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySummaryRow {
    pub title: String,
    pub memory_type: String,
    pub importance: f32,
}

impl AppApi {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn brain(&self) -> Brain {
        Brain::new(&self.root)
    }

    pub fn init(&self) -> Result<()> {
        self.brain().init()
    }

    pub fn send_chat_message(
        &self,
        input: &str,
        mode: ConversationMode,
    ) -> Result<ConversationTurnOutput> {
        self.brain().run_mode(mode, input, false)
    }

    pub fn run_autonomous_goal(
        &self,
        input: &str,
        level: AutonomyLevel,
    ) -> Result<crate::agency::AutonomousWorkerResult> {
        self.brain().autonomize(input.to_string(), level)
    }

    pub fn get_sessions(&self) -> Result<Vec<WorkSessionSummary>> {
        self.brain().sessions()
    }

    pub fn get_artifacts(&self) -> Result<ArtifactOverview> {
        self.brain().artifacts()
    }

    pub fn get_artifact_packs(&self) -> Result<ArtifactPackOverview> {
        self.brain().artifact_packs()
    }

    pub fn get_memory_summary(&self) -> Result<Vec<MemorySummaryRow>> {
        let report = self.brain().memory_inspect()?;
        Ok(vec![
            row(
                "semantic memories",
                MemoryType::Semantic,
                report.semantic_memories,
            ),
            row(
                "procedural memories",
                MemoryType::Procedural,
                report.procedural_memories,
            ),
            row(
                "project memories",
                MemoryType::Project,
                report.project_memories,
            ),
        ])
    }

    pub fn get_task_graph(&self) -> Result<crate::agency::TaskGraph> {
        self.brain().task_graph("latest")
    }

    pub fn get_brain_status(&self) -> Result<BrainStatus> {
        self.brain().brain_status()
    }

    pub fn get_inspect_summary(&self) -> Result<InspectSummaryLite> {
        self.brain().inspect_summary()
    }

    pub fn get_safety_status(&self) -> SafetyStatus {
        SafetyStatus {
            sandbox_enabled: true,
            network_default: "disabled by default".to_string(),
            allowlisted_commands: vec![
                "cargo fmt".to_string(),
                "cargo check".to_string(),
                "cargo test".to_string(),
            ],
            safety_note: "AppApi actions preserve sandbox/allowlist boundaries.".to_string(),
        }
    }

    pub fn run_doctor(&self) -> Result<DoctorReport> {
        self.brain().doctor(false)
    }

    pub fn run_regression_check(&self) -> Result<RegressionCheckReport> {
        self.brain().regression_check()
    }

    pub fn run_creative_project(&self, prompt: &str) -> Result<CreativeRunReport> {
        create_creative_project(self.brain().store(), prompt)
    }

    pub fn export_latest_package(&self) -> Result<String> {
        Ok("Export requires an active reviewed session. Use session export commands after a safe run."
            .to_string())
    }

    pub fn get_executive_status(&self) -> Result<ExecutiveStatus> {
        executive_status(self.brain().store())
    }

    pub fn session_report(&self, selector: &str) -> Result<SessionDashboardReport> {
        self.brain().session_report(selector)
    }
}

fn row(title: &str, memory_type: MemoryType, count: usize) -> MemorySummaryRow {
    MemorySummaryRow {
        title: format!("{title}: {count}"),
        memory_type: format!("{:?}", memory_type),
        importance: if count > 0 { 0.8 } else { 0.2 },
    }
}
