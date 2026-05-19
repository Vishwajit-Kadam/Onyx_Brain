use std::time::{Duration, Instant};

use crate::{
    agency::AutonomyLevel,
    app_api::{
        AppApi, AppApiError, CreativeProjectType, MemorySummaryRow, ProjectItem,
        RecentActivityItem, SafetyStatus, SearchResult, TaskItem,
    },
    artifacts::{ArtifactOverview, ArtifactPackOverview},
    conversation::{ConversationMode, PersonalityProfile},
    core::brain::BrainStatus,
};

use super::settings::{GuiSettings, ThemeMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Home,
    Scheduled,
    Search,
    Library,
    Projects,
    Settings,
    Chat,
    Autonomy,
    CreativeStudio,
    Tasks,
    Artifacts,
    Memory,
    Safety,
    System,
}

impl ActiveView {
    pub fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Scheduled => "Scheduled",
            Self::Search => "Search",
            Self::Library => "Library",
            Self::Projects => "Projects",
            Self::Settings => "Settings",
            Self::Chat => "Chat",
            Self::Autonomy => "Autonomy",
            Self::CreativeStudio => "Creative Studio",
            Self::Tasks => "Tasks",
            Self::Artifacts => "Artifacts",
            Self::Memory => "Memory",
            Self::Safety => "Safety",
            Self::System => "System",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalState {
    None,
    CommandPalette,
    Search,
    NewProject,
    DisabledScheduled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct GuiState {
    pub active_view: ActiveView,
    pub current_prompt: String,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub chat_messages: Vec<ChatMessage>,
    pub selected_mode: ConversationMode,
    pub selected_personality: PersonalityProfile,
    pub selected_autonomy_level: AutonomyLevel,
    pub brain_status: Option<BrainStatus>,
    pub safety_status: Option<SafetyStatus>,
    pub artifacts: ArtifactOverview,
    pub artifact_packs: ArtifactPackOverview,
    pub sessions: Vec<crate::agency::WorkSessionSummary>,
    pub tasks: Vec<TaskItem>,
    pub memories: Vec<MemorySummaryRow>,
    pub projects: Vec<ProjectItem>,
    pub recent_activity: Vec<RecentActivityItem>,
    pub settings: GuiSettings,
    pub loading_action: Option<String>,
    pub toast_messages: Vec<Toast>,
    pub error_banner: Option<String>,
    pub selected_item: Option<String>,
    pub modal_state: ModalState,
    pub library_query: String,
    pub library_grid: bool,
    pub creative_prompt: String,
    pub creative_type: CreativeProjectType,
    pub autonomy_prompt: String,
    pub chat_input: String,
}

impl GuiState {
    pub fn new(settings: GuiSettings) -> Self {
        Self {
            active_view: ActiveView::Home,
            current_prompt: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            chat_messages: Vec::new(),
            selected_mode: settings.default_conversation_mode.clone(),
            selected_personality: settings.default_personality.clone(),
            selected_autonomy_level: settings.default_autonomy_level.clone(),
            brain_status: None,
            safety_status: None,
            artifacts: ArtifactOverview::default(),
            artifact_packs: ArtifactPackOverview::default(),
            sessions: Vec::new(),
            tasks: Vec::new(),
            memories: Vec::new(),
            projects: Vec::new(),
            recent_activity: Vec::new(),
            settings,
            loading_action: None,
            toast_messages: Vec::new(),
            error_banner: None,
            selected_item: None,
            modal_state: ModalState::None,
            library_query: String::new(),
            library_grid: true,
            creative_prompt: String::new(),
            creative_type: CreativeProjectType::General,
            autonomy_prompt: String::new(),
            chat_input: String::new(),
        }
    }

    pub fn refresh_all(&mut self, api: &AppApi) {
        self.refresh_status(api);
        match api.list_artifacts() {
            Ok(value) => self.artifacts = value,
            Err(error) => self.handle_api_error(error),
        }
        match api.list_artifact_packs() {
            Ok(value) => self.artifact_packs = value,
            Err(error) => self.handle_api_error(error),
        }
        match api.list_sessions() {
            Ok(value) => self.sessions = value,
            Err(error) => self.handle_api_error(error),
        }
        match api.list_tasks() {
            Ok(value) => self.tasks = value,
            Err(error) => self.handle_api_error(error),
        }
        match api.list_memories() {
            Ok(value) => self.memories = value,
            Err(error) => self.handle_api_error(error),
        }
        match api.list_projects() {
            Ok(value) => self.projects = value,
            Err(error) => self.handle_api_error(error),
        }
        match api.get_recent_activity() {
            Ok(value) => self.recent_activity = value,
            Err(error) => self.handle_api_error(error),
        }
        match api.search_all("") {
            Ok(value) => self.search_results = value,
            Err(error) => self.handle_api_error(error),
        }
    }

    pub fn refresh_status(&mut self, api: &AppApi) {
        match api.get_brain_status() {
            Ok(status) => self.brain_status = Some(status),
            Err(error) => self.show_error(error.message().to_string()),
        }
        match api.get_safety_status() {
            Ok(status) => self.safety_status = Some(status),
            Err(error) => self.show_error(error.message().to_string()),
        }
    }

    pub fn show_error(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.error_banner = Some(message.clone());
        self.toast_messages.push(Toast {
            message,
            kind: ToastKind::Error,
            expires_at: Instant::now() + Duration::from_secs(7),
        });
    }

    pub fn show_success(&mut self, message: impl Into<String>) {
        self.toast_messages.push(Toast {
            message: message.into(),
            kind: ToastKind::Success,
            expires_at: Instant::now() + Duration::from_secs(5),
        });
    }

    pub fn show_info(&mut self, message: impl Into<String>) {
        self.toast_messages.push(Toast {
            message: message.into(),
            kind: ToastKind::Info,
            expires_at: Instant::now() + Duration::from_secs(5),
        });
    }

    pub fn set_loading(&mut self, action: impl Into<String>) {
        self.loading_action = Some(action.into());
    }

    pub fn clear_loading(&mut self) {
        self.loading_action = None;
    }

    pub fn submit_main_prompt(&mut self, api: &AppApi) {
        let prompt = self.current_prompt.trim().to_string();
        if prompt.is_empty() {
            self.show_info("Type a prompt first.");
            return;
        }
        self.set_loading("Generating response...");
        let lower = prompt.to_ascii_lowercase();
        let result_message =
            if lower.contains("video") || lower.contains("movie") || lower.contains("creative") {
                api.run_creative_project(&prompt, CreativeProjectType::Video, 3)
                    .map(|report| {
                        format!(
                            "Creative project created: {}. Artifacts: {}",
                            report.title,
                            report.artifacts_created.len()
                        )
                    })
            } else if lower.contains("build")
                || lower.contains("create")
                || lower.contains("plan")
                || lower.contains("generate")
            {
                api.run_autonomous_goal(&prompt, self.selected_autonomy_level.clone())
                    .map(|report| {
                        format!(
                            "Autonomy run finished: {:?}. Planned {}, completed {}.",
                            report.status, report.tasks_planned, report.tasks_completed
                        )
                    })
            } else {
                api.send_chat_message(
                    &prompt,
                    self.selected_mode.clone(),
                    self.selected_personality.clone(),
                )
                .map(|turn| {
                    self.chat_messages.push(ChatMessage {
                        role: "You".to_string(),
                        content: prompt.clone(),
                    });
                    self.chat_messages.push(ChatMessage {
                        role: "Onyx".to_string(),
                        content: turn.response.clone(),
                    });
                    self.active_view = ActiveView::Chat;
                    turn.response
                })
            };
        self.clear_loading();
        match result_message {
            Ok(message) => {
                self.current_prompt.clear();
                self.show_success("Action completed.");
                if self.active_view != ActiveView::Chat {
                    self.chat_messages.push(ChatMessage {
                        role: "Onyx".to_string(),
                        content: message,
                    });
                    self.active_view = ActiveView::Chat;
                }
                self.refresh_all(api);
            }
            Err(error) => self.handle_api_error(error),
        }
    }

    pub fn run_search(&mut self, api: &AppApi) {
        match api.search_all(&self.search_query) {
            Ok(results) => self.search_results = results,
            Err(error) => self.handle_api_error(error),
        }
    }

    pub fn switch_view(&mut self, view: ActiveView) {
        self.active_view = view;
        self.settings.last_active_view = view.label().to_string();
        if matches!(view, ActiveView::Search) {
            self.modal_state = ModalState::Search;
        }
    }

    pub fn handle_api_error(&mut self, error: AppApiError) {
        let mut message = error.message().to_string();
        if error.doctor_recommended() {
            message.push_str(" Use Safety > Run Doctor for details.");
        }
        self.show_error(message);
    }

    pub fn set_theme(&mut self, theme: ThemeMode) {
        self.settings.theme_mode = theme;
        self.show_success("Theme saved.");
    }

    pub fn prune_toasts(&mut self) {
        let now = Instant::now();
        self.toast_messages.retain(|toast| toast.expires_at > now);
    }
}
