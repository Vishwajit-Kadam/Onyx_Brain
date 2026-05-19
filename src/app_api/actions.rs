use chrono::Utc;
use uuid::Uuid;

use crate::{
    agency::{AutonomyLevel, SessionDashboardReport, WorkSessionSummary},
    artifacts::{ArtifactOverview, ArtifactPackOverview},
    conversation::{
        save_personality, ConversationMode, ConversationTurnOutput, PersonalityProfile,
    },
    core::brain::{BrainStatus, InspectSummaryLite},
    creative::{create_creative_project, CreativeRunReport},
    executive::{executive_status, ExecutiveStatus},
    memory::MemoryType,
    storage::{load_json, save_json, DoctorReport},
    testing::RegressionCheckReport,
    ONYX_VERSION,
};

use super::{
    row, AppActionResult, AppApi, AppApiResult, AppRuntimeSettings, CreativeProjectType,
    MemorySummaryRow, ProjectItem, RecentActivityItem, SafetyStatus, SearchResult,
    SearchResultKind, TaskItem, WorkspaceInfo,
};

impl AppApi {
    pub fn init(&self) -> AppApiResult<()> {
        self.brain().init().map_err(Into::into)
    }

    pub fn send_chat_message(
        &self,
        input: &str,
        mode: ConversationMode,
        personality: PersonalityProfile,
    ) -> AppApiResult<ConversationTurnOutput> {
        let brain = self.brain();
        save_personality(brain.store(), &personality).map_err(anyhow::Error::from)?;
        brain.run_mode(mode, input, false).map_err(Into::into)
    }

    pub fn run_autonomous_goal(
        &self,
        input: &str,
        level: AutonomyLevel,
    ) -> AppApiResult<crate::agency::AutonomousWorkerResult> {
        self.brain()
            .autonomize(input.to_string(), level)
            .map_err(Into::into)
    }

    pub fn run_creative_project(
        &self,
        prompt: &str,
        _project_type: CreativeProjectType,
        _duration_minutes: u32,
    ) -> AppApiResult<CreativeRunReport> {
        create_creative_project(self.brain().store(), prompt).map_err(Into::into)
    }

    pub fn list_sessions(&self) -> AppApiResult<Vec<WorkSessionSummary>> {
        self.brain().sessions().map_err(Into::into)
    }

    pub fn get_sessions(&self) -> AppApiResult<Vec<WorkSessionSummary>> {
        self.list_sessions()
    }

    pub fn list_artifacts(&self) -> AppApiResult<ArtifactOverview> {
        self.brain().artifacts().map_err(Into::into)
    }

    pub fn get_artifacts(&self) -> AppApiResult<ArtifactOverview> {
        self.list_artifacts()
    }

    pub fn list_artifact_packs(&self) -> AppApiResult<ArtifactPackOverview> {
        self.brain().artifact_packs().map_err(Into::into)
    }

    pub fn get_artifact_packs(&self) -> AppApiResult<ArtifactPackOverview> {
        self.list_artifact_packs()
    }

    pub fn list_tasks(&self) -> AppApiResult<Vec<TaskItem>> {
        let mut tasks = Vec::new();
        for goal in self.brain().goals().map_err(anyhow::Error::from)? {
            tasks.push(TaskItem {
                id: goal.goal_id,
                title: goal.title,
                subtitle: goal.original_prompt,
                status: format!("{:?}", goal.status),
                created_at: Some(goal.created_at),
                demo: false,
            });
        }
        let gui_task_dir = self.root.join("data").join("gui_tasks");
        if gui_task_dir.exists() {
            for entry in std::fs::read_dir(gui_task_dir).map_err(anyhow::Error::from)? {
                let path = entry.map_err(anyhow::Error::from)?.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    if let Ok(task) = load_json::<TaskItem>(&path) {
                        tasks.push(task);
                    }
                }
            }
        }
        if tasks.is_empty() {
            tasks.push(TaskItem {
                id: "demo_get_started".to_string(),
                title: "Demo: Create your first bounded task".to_string(),
                subtitle: "Demo data shown because no local tasks exist yet.".to_string(),
                status: "Demo only".to_string(),
                created_at: None,
                demo: true,
            });
        }
        Ok(tasks)
    }

    pub fn list_memories(&self) -> AppApiResult<Vec<MemorySummaryRow>> {
        self.get_memory_summary()
    }

    pub fn get_memory_summary(&self) -> AppApiResult<Vec<MemorySummaryRow>> {
        let report = self.brain().memory_inspect().map_err(anyhow::Error::from)?;
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

    pub fn list_projects(&self) -> AppApiResult<Vec<ProjectItem>> {
        Ok(self
            .brain()
            .projects()
            .map_err(anyhow::Error::from)?
            .into_iter()
            .map(|project| ProjectItem {
                id: project.goal_id,
                name: project.project_name,
                status: project.status,
                summary: project.summary,
                root_path: Some(project.root_path),
                updated_at: Some(project.updated_at),
            })
            .collect())
    }

    pub fn search_all(&self, query: &str) -> AppApiResult<Vec<SearchResult>> {
        let needle = query.trim().to_ascii_lowercase();
        let include = |text: &str| needle.is_empty() || text.to_ascii_lowercase().contains(&needle);
        let mut results = Vec::new();

        for task in self.list_tasks()? {
            if include(&task.title) || include(&task.subtitle) || include(&task.status) {
                results.push(SearchResult {
                    title: task.title,
                    subtitle: task.subtitle,
                    kind: SearchResultKind::Task,
                    path_or_id: task.id,
                    action: "open_tasks".to_string(),
                    demo: task.demo,
                });
            }
        }

        if let Ok(sessions) = self.list_sessions() {
            for session in sessions {
                if include(&session.title) || include(&session.session_id) {
                    results.push(SearchResult {
                        title: session.title,
                        subtitle: format!("{:?} session", session.status),
                        kind: SearchResultKind::Session,
                        path_or_id: session.session_id,
                        action: "open_chat".to_string(),
                        demo: false,
                    });
                }
            }
        }

        if let Ok(overview) = self.list_artifacts() {
            for artifact in overview.artifacts {
                if include(&artifact.path) || include(&artifact.session_id) {
                    let title = match artifact.path.rsplit(['\\', '/']).next() {
                        Some(name) if !name.is_empty() => name.to_string(),
                        _ => "Artifact".to_string(),
                    };
                    results.push(SearchResult {
                        title,
                        subtitle: format!(
                            "{:?} score {:.2}",
                            artifact.artifact_type, artifact.validation_score
                        ),
                        kind: SearchResultKind::Artifact,
                        path_or_id: artifact.session_id,
                        action: "open_artifacts".to_string(),
                        demo: false,
                    });
                }
            }
        }

        if let Ok(overview) = self.list_artifact_packs() {
            for pack in overview.packs {
                if include(&pack.title) || include(&pack.pack_id) {
                    results.push(SearchResult {
                        title: pack.title,
                        subtitle: format!("Artifact pack score {:.2}", pack.validation_score),
                        kind: SearchResultKind::ArtifactPack,
                        path_or_id: pack.pack_id,
                        action: "open_artifacts".to_string(),
                        demo: false,
                    });
                }
            }
        }

        if let Ok(memories) = self.list_memories() {
            for memory in memories {
                if include(&memory.title) || include(&memory.memory_type) {
                    results.push(SearchResult {
                        title: memory.title,
                        subtitle: format!(
                            "{} importance {:.2}",
                            memory.memory_type, memory.importance
                        ),
                        kind: SearchResultKind::Memory,
                        path_or_id: memory.memory_type,
                        action: "open_memory".to_string(),
                        demo: false,
                    });
                }
            }
        }

        if let Ok(projects) = self.list_projects() {
            for project in projects {
                if include(&project.name) || include(&project.summary) || include(&project.status) {
                    results.push(SearchResult {
                        title: project.name,
                        subtitle: project.summary,
                        kind: SearchResultKind::Project,
                        path_or_id: project.id,
                        action: "open_projects".to_string(),
                        demo: false,
                    });
                }
            }
        }

        if results.is_empty() && needle.is_empty() {
            results.push(SearchResult {
                title: "Demo: Start a new task".to_string(),
                subtitle: "Search local Onyx data, tasks, sessions, artifacts, and projects."
                    .to_string(),
                kind: SearchResultKind::Demo,
                path_or_id: "demo_new_task".to_string(),
                action: "new_task".to_string(),
                demo: true,
            });
        }

        Ok(results)
    }

    pub fn run_doctor(&self) -> AppApiResult<DoctorReport> {
        self.brain().doctor(false).map_err(Into::into)
    }

    pub fn run_regression_check(&self) -> AppApiResult<RegressionCheckReport> {
        self.brain().regression_check().map_err(Into::into)
    }

    pub fn run_maintain(&self) -> AppApiResult<AppActionResult> {
        let (dedup, backups, consolidation, compare) =
            self.brain().maintain().map_err(anyhow::Error::from)?;
        Ok(AppActionResult {
            title: "Maintenance completed".to_string(),
            message: format!(
                "Archived {} memories, removed {} backups, strengthened {} routes.",
                dedup.memories_archived, backups.backups_removed, consolidation.strengthened_routes
            ),
            details: vec![format!("Benchmark trend: {}", compare.runtime_trend)],
        })
    }

    pub fn export_latest_package(&self) -> AppApiResult<AppActionResult> {
        let report = self
            .brain()
            .export_package("latest")
            .map_err(anyhow::Error::from)?;
        Ok(AppActionResult {
            title: "Export package created".to_string(),
            message: report.export_path,
            details: vec![format!("Files exported: {}", report.files_exported)],
        })
    }

    pub fn create_task(&self, prompt: &str) -> AppApiResult<TaskItem> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return Err(super::AppApiError::friendly(
                "Enter a task prompt before creating a task.",
            ));
        }
        let brain = self.brain();
        brain.init().map_err(anyhow::Error::from)?;
        let id = format!(
            "gui_task_{}_{}",
            Utc::now().format("%Y%m%d%H%M%S"),
            Uuid::new_v4()
        );
        let path = brain
            .store()
            .paths
            .data
            .join("gui_tasks")
            .join(format!("{id}.json"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(anyhow::Error::from)?;
        }
        let task = TaskItem {
            id,
            title: trimmed.chars().take(80).collect(),
            subtitle: "Safe local GUI task entry. Run autonomy explicitly to execute.".to_string(),
            status: "Created".to_string(),
            created_at: Some(Utc::now()),
            demo: false,
        };
        save_json(&path, &task).map_err(anyhow::Error::from)?;
        Ok(task)
    }

    pub fn get_settings(&self) -> AppApiResult<AppRuntimeSettings> {
        let path = self
            .root
            .join("data")
            .join("config")
            .join("app_runtime_settings.json");
        if path.exists() {
            return load_json::<AppRuntimeSettings>(&path).map_err(Into::into);
        }
        Ok(AppRuntimeSettings {
            default_personality: "Balanced".to_string(),
            default_conversation_mode: "Standard".to_string(),
            default_autonomy_level: "Standard".to_string(),
        })
    }

    pub fn update_settings(
        &self,
        settings: AppRuntimeSettings,
    ) -> AppApiResult<AppRuntimeSettings> {
        let path = self
            .root
            .join("data")
            .join("config")
            .join("app_runtime_settings.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(anyhow::Error::from)?;
        }
        save_json(&path, &settings).map_err(anyhow::Error::from)?;
        Ok(settings)
    }

    pub fn get_recent_activity(&self) -> AppApiResult<Vec<RecentActivityItem>> {
        let mut rows = Vec::new();
        if let Ok(sessions) = self.list_sessions() {
            for session in sessions.into_iter().take(5) {
                rows.push(RecentActivityItem {
                    id: session.session_id,
                    title: session.title,
                    subtitle: format!("{:?}", session.status),
                    kind: "Session".to_string(),
                });
            }
        }
        if let Ok(tasks) = self.list_tasks() {
            for task in tasks.into_iter().take(5) {
                rows.push(RecentActivityItem {
                    id: task.id,
                    title: task.title,
                    subtitle: task.status,
                    kind: "Task".to_string(),
                });
            }
        }
        Ok(rows)
    }

    pub fn get_current_workspace(&self) -> AppApiResult<WorkspaceInfo> {
        Ok(WorkspaceInfo {
            root: self.root.clone(),
            data_dir: self.root.join("data"),
            sandbox_dir: self.root.join("sandbox"),
            version: ONYX_VERSION.to_string(),
        })
    }

    pub fn get_task_graph(&self) -> AppApiResult<crate::agency::TaskGraph> {
        self.brain().task_graph("latest").map_err(Into::into)
    }

    pub fn get_brain_status(&self) -> AppApiResult<BrainStatus> {
        self.brain().brain_status().map_err(Into::into)
    }

    pub fn get_inspect_summary(&self) -> AppApiResult<InspectSummaryLite> {
        self.brain().inspect_summary().map_err(Into::into)
    }

    pub fn get_safety_status(&self) -> AppApiResult<SafetyStatus> {
        Ok(SafetyStatus {
            sandbox_enabled: true,
            network_default: "disabled by default".to_string(),
            allowlisted_commands: vec![
                "cargo fmt".to_string(),
                "cargo check".to_string(),
                "cargo test".to_string(),
            ],
            safety_note:
                "Bounded autonomy uses local sandbox paths and allowlisted runtime actions."
                    .to_string(),
        })
    }

    pub fn get_executive_status(&self) -> AppApiResult<ExecutiveStatus> {
        executive_status(self.brain().store()).map_err(Into::into)
    }

    pub fn session_report(&self, selector: &str) -> AppApiResult<SessionDashboardReport> {
        self.brain().session_report(selector).map_err(Into::into)
    }
}
