pub mod settings;
pub mod state;

use std::path::PathBuf;

use eframe::egui::{self, RichText};

use crate::{
    agency::AutonomyLevel,
    app_api::{AppApi, SearchResultKind},
    conversation::{ConversationMode, PersonalityProfile},
};

use self::{
    settings::{save as save_gui_settings, ThemeMode},
    state::{ActiveView, GuiState, ModalState, ToastKind},
};

pub fn run_gui(root: PathBuf) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Onyx Brain",
        options,
        Box::new(move |cc| Ok(Box::new(OnyxGuiApp::new(root.clone(), cc)))),
    )
    .map_err(|error| anyhow::anyhow!("GUI failed to launch: {error}"))
}

struct OnyxGuiApp {
    root: PathBuf,
    api: AppApi,
    state: GuiState,
    command_query: String,
    new_project_prompt: String,
}

impl OnyxGuiApp {
    fn new(root: PathBuf, cc: &eframe::CreationContext<'_>) -> Self {
        let api = AppApi::new(&root);
        let settings = match settings::load_or_default(&root) {
            Ok(settings) => settings,
            Err(_) => settings::GuiSettings::default(),
        };
        apply_theme(&cc.egui_ctx, settings.theme_mode);
        let mut state = GuiState::new(settings);
        if let Err(error) = api.init() {
            state.handle_api_error(error);
        }
        state.refresh_all(&api);
        Self {
            root,
            api,
            state,
            command_query: String::new(),
            new_project_prompt: String::new(),
        }
    }

    fn persist_settings(&mut self, ctx: &egui::Context) {
        apply_theme(ctx, self.state.settings.theme_mode);
        if let Err(error) = save_gui_settings(&self.root, &self.state.settings) {
            self.state
                .show_error(format!("Could not save GUI settings: {error}"));
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.heading(RichText::new("Onyx").size(22.0));
        ui.label(RichText::new("Native Windows app").weak());
        ui.separator();
        self.nav_button(ui, "New task", ActiveView::Home);
        self.nav_button(ui, "Scheduled", ActiveView::Scheduled);
        if ui.button("Search").clicked() {
            self.state.modal_state = ModalState::Search;
            self.state.switch_view(ActiveView::Search);
            self.state.run_search(&self.api);
        }
        self.nav_button(ui, "Library", ActiveView::Library);
        self.nav_button(ui, "Chat", ActiveView::Chat);
        self.nav_button(ui, "Autonomy", ActiveView::Autonomy);
        self.nav_button(ui, "Creative Studio", ActiveView::CreativeStudio);
        self.nav_button(ui, "Tasks", ActiveView::Tasks);
        self.nav_button(ui, "Artifacts", ActiveView::Artifacts);
        self.nav_button(ui, "Memory", ActiveView::Memory);
        self.nav_button(ui, "Safety", ActiveView::Safety);
        self.nav_button(ui, "System", ActiveView::System);
        ui.separator();
        ui.label(RichText::new("Projects").weak());
        if ui.button("+ New project").clicked() {
            self.state.modal_state = ModalState::NewProject;
            self.new_project_prompt.clear();
        }
        if ui.button("All tasks").clicked() {
            self.state.switch_view(ActiveView::Tasks);
        }
        if self.state.projects.is_empty() {
            ui.label(RichText::new("No projects yet").weak());
        } else {
            for project in self.state.projects.clone() {
                if ui
                    .button(format!("{} - {}", project.name, project.status))
                    .clicked()
                {
                    self.state.selected_item = Some(project.id);
                    self.state.switch_view(ActiveView::Projects);
                }
            }
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            if ui.button("Settings").clicked() {
                self.state.switch_view(ActiveView::Settings);
            }
            if ui.button("Get help").clicked() {
                self.state.switch_view(ActiveView::Settings);
                self.state
                    .show_info("Help is available in README.md and docs/gui.md.");
            }
        });
    }

    fn nav_button(&mut self, ui: &mut egui::Ui, label: &str, view: ActiveView) {
        let selected = self.state.active_view == view;
        if ui.selectable_label(selected, label).clicked() {
            self.state.switch_view(view);
        }
    }

    fn top_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(self.state.active_view.label());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Settings").clicked() {
                    self.state.switch_view(ActiveView::Settings);
                }
                if ui.button("Ctrl+K").clicked() {
                    self.state.modal_state = ModalState::CommandPalette;
                }
                if ui.button("Refresh").clicked() {
                    self.state.set_loading("Refreshing status...");
                    self.state.refresh_all(&self.api);
                    self.state.clear_loading();
                    self.state.show_success("Refreshed.");
                }
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
        if let Some(action) = &self.state.loading_action {
            ui.label(RichText::new(action).italics());
        }
        if let Some(error) = self.state.error_banner.clone() {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(64, 34, 34))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(error);
                        if ui.button("Run Doctor").clicked() {
                            self.run_doctor();
                        }
                        if ui.button("Dismiss").clicked() {
                            self.state.error_banner = None;
                        }
                    });
                });
        }
        ui.separator();
    }

    fn home_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            ui.heading(RichText::new("What can I do for you?").size(36.0));
            ui.add_space(12.0);
        });
        egui::Frame::group(ui.style()).show(ui, |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.state.current_prompt)
                    .hint_text("Ask Onyx anything, @ to mention local context")
                    .desired_rows(3)
                    .lock_focus(true),
            );
            if response.lost_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.ctrl)
            {
                self.state.submit_main_prompt(&self.api);
            }
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button("+")
                    .on_hover_text("Create a safe local task entry")
                    .clicked()
                {
                    match self.api.create_task(&self.state.current_prompt) {
                        Ok(task) => {
                            self.state.tasks.push(task);
                            self.state.show_success("Task created.");
                        }
                        Err(error) => self.state.handle_api_error(error),
                    }
                }
                self.mode_selector(ui);
                self.personality_selector(ui);
                self.autonomy_selector(ui);
                if ui.button("Send").clicked() {
                    self.state.submit_main_prompt(&self.api);
                }
            });
        });
        ui.add_space(14.0);
        ui.horizontal_wrapped(|ui| {
            self.quick_chip(ui, "Create slides", "Create a polished slide deck about ");
            self.quick_chip(
                ui,
                "Build website",
                "Create a website plan and starter project for ",
            );
            self.quick_chip(
                ui,
                "Develop desktop apps",
                "Create a native desktop app plan for ",
            );
            self.quick_chip(ui, "Design", "Create a design brief for ");
            if ui.button("More").clicked() {
                self.state.modal_state = ModalState::CommandPalette;
            }
        });
        ui.add_space(16.0);
        ui.heading("Recent activity");
        if self.state.recent_activity.is_empty() {
            ui.label(RichText::new("No recent activity yet.").weak());
        } else {
            for item in &self.state.recent_activity {
                ui.label(format!("{}: {} - {}", item.kind, item.title, item.subtitle));
            }
        }
    }

    fn quick_chip(&mut self, ui: &mut egui::Ui, label: &str, prompt: &str) {
        if ui.button(label).clicked() {
            self.state.current_prompt = prompt.to_string();
            self.state.show_info(format!("{label} prompt loaded."));
        }
    }

    fn scheduled_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scheduled tasks");
        ui.label("Scheduled autonomous tasks are planned but disabled in v0.0.4. Onyx does not run hidden background jobs.");
        for row in [
            "Set up automated monitoring for a safe local topic. Preview only.",
            "Create a daily planning checklist. Preview only.",
            "Turn a manual process into a bounded explicit run. Preview only.",
        ] {
            ui.horizontal(|ui| {
                ui.label(row);
                ui.add_enabled(false, egui::Button::new("Preview only"));
            });
        }
        if ui.button("Create scheduled task").clicked() {
            self.state.modal_state = ModalState::DisabledScheduled;
        }
    }

    fn library_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("All").clicked() {
                self.state.show_success("Showing all library items.");
            }
            if ui.button("My favorites").clicked() {
                self.state
                    .show_info("Favorites are not implemented in v0.0.4.");
            }
            ui.separator();
            ui.label("Search files");
            ui.text_edit_singleline(&mut self.state.library_query);
            if ui
                .button(if self.state.library_grid {
                    "Grid"
                } else {
                    "List"
                })
                .clicked()
            {
                self.state.library_grid = !self.state.library_grid;
            }
            if ui.button("New task").clicked() {
                self.state.switch_view(ActiveView::Home);
            }
        });
        ui.separator();
        let query = self.state.library_query.to_ascii_lowercase();
        let mut rows = Vec::new();
        rows.extend(
            self.state
                .artifacts
                .artifacts
                .iter()
                .map(|a| format!("Artifact: {} ({:?})", a.path, a.artifact_type)),
        );
        rows.extend(
            self.state
                .artifact_packs
                .packs
                .iter()
                .map(|p| format!("Pack: {} ({:.2})", p.title, p.validation_score)),
        );
        rows.extend(
            self.state
                .sessions
                .iter()
                .map(|s| format!("Session: {} ({:?})", s.title, s.status)),
        );
        rows.retain(|row| query.is_empty() || row.to_ascii_lowercase().contains(&query));
        if rows.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Nothing in the library");
                ui.label(
                    RichText::new(
                        "Generated artifacts, packs, exports, and reports will appear here.",
                    )
                    .weak(),
                );
                if ui.button("New task").clicked() {
                    self.state.switch_view(ActiveView::Home);
                }
            });
        } else if self.state.library_grid {
            egui::Grid::new("library_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    for (idx, row) in rows.iter().enumerate() {
                        ui.group(|ui| {
                            ui.label(row);
                        });
                        if idx % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
        } else {
            for row in rows {
                ui.label(row);
            }
        }
    }

    fn settings_view(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("General");
        ui.horizontal(|ui| {
            ui.label("Language");
            ui.add_enabled(false, egui::Button::new("English (display only)"));
        });
        ui.label("Theme");
        ui.horizontal(|ui| {
            for (theme, label) in [
                (ThemeMode::Light, "Light"),
                (ThemeMode::Dark, "Dark"),
                (ThemeMode::Auto, "Auto"),
            ] {
                if ui
                    .selectable_label(self.state.settings.theme_mode == theme, label)
                    .clicked()
                {
                    self.state.set_theme(theme);
                    self.persist_settings(ctx);
                }
            }
        });
        ui.separator();
        ui.heading("Personalization");
        self.personality_selector(ui);
        if ui.button("Save personalization").clicked() {
            self.state.settings.default_personality = self.state.selected_personality.clone();
            self.persist_settings(ctx);
            self.state.show_success("Personalization saved.");
        }
        ui.separator();
        ui.heading("Skills");
        if self.state.memories.is_empty() {
            ui.label("No local skills or memory summaries found.");
        } else {
            for row in &self.state.memories {
                ui.label(format!("{} - {}", row.memory_type, row.title));
            }
        }
        ui.separator();
        ui.heading("Connectors");
        ui.label("External connectors are not enabled in v0.0.4.");
        ui.separator();
        ui.heading("Get help");
        ui.label("Useful commands: cargo run -- chat \"Hello Onyx\"; cargo run -- doctor; cargo run -- regression-check.");
        if ui.button("Close settings").clicked() {
            self.state.switch_view(ActiveView::Home);
        }
    }

    fn chat_view(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .max_height(520.0)
            .show(ui, |ui| {
                if self.state.chat_messages.is_empty() {
                    ui.label(RichText::new("No chat yet. Send a message to start.").weak());
                }
                for message in &self.state.chat_messages {
                    ui.group(|ui| {
                        ui.label(RichText::new(&message.role).strong());
                        ui.label(&message.content);
                    });
                }
            });
        ui.separator();
        ui.horizontal(|ui| {
            self.mode_selector(ui);
            self.personality_selector(ui);
        });
        let response = ui.add(
            egui::TextEdit::multiline(&mut self.state.chat_input)
                .hint_text("Send message to Onyx")
                .desired_rows(2),
        );
        if response.lost_focus()
            && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.ctrl)
        {
            self.send_chat();
        }
        if ui.button("Send message").clicked() {
            self.send_chat();
        }
    }

    fn send_chat(&mut self) {
        let input = self.state.chat_input.trim().to_string();
        if input.is_empty() {
            self.state.show_info("Type a message first.");
            return;
        }
        self.state.chat_messages.push(state::ChatMessage {
            role: "You".to_string(),
            content: input.clone(),
        });
        self.state.set_loading("Generating response...");
        match self.api.send_chat_message(
            &input,
            self.state.selected_mode.clone(),
            self.state.selected_personality.clone(),
        ) {
            Ok(turn) => {
                self.state.chat_messages.push(state::ChatMessage {
                    role: "Onyx".to_string(),
                    content: turn.response,
                });
                self.state.chat_input.clear();
                self.state.show_success("Message sent.");
            }
            Err(error) => self.state.handle_api_error(error),
        }
        self.state.clear_loading();
    }

    fn autonomy_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bounded autonomy");
        self.autonomy_selector(ui);
        ui.add(
            egui::TextEdit::multiline(&mut self.state.autonomy_prompt)
                .hint_text("Describe the explicit goal to run")
                .desired_rows(5),
        );
        if ui.button("Run goal").clicked() {
            let prompt = self.state.autonomy_prompt.trim().to_string();
            if prompt.is_empty() {
                self.state.show_info("Enter an autonomy goal first.");
            } else {
                self.state.set_loading("Running bounded autonomy...");
                match self
                    .api
                    .run_autonomous_goal(&prompt, self.state.selected_autonomy_level.clone())
                {
                    Ok(report) => {
                        self.state.show_success(format!(
                            "Run completed: planned {}, completed {}.",
                            report.tasks_planned, report.tasks_completed
                        ));
                        self.state.refresh_all(&self.api);
                    }
                    Err(error) => self.state.handle_api_error(error),
                }
                self.state.clear_loading();
            }
        }
    }

    fn creative_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Creative Studio");
        egui::ComboBox::from_label("Project type")
            .selected_text(format!("{:?}", self.state.creative_type))
            .show_ui(ui, |ui| {
                for value in [
                    crate::app_api::CreativeProjectType::General,
                    crate::app_api::CreativeProjectType::Video,
                    crate::app_api::CreativeProjectType::Movie,
                    crate::app_api::CreativeProjectType::Design,
                    crate::app_api::CreativeProjectType::Slides,
                ] {
                    ui.selectable_value(
                        &mut self.state.creative_type,
                        value.clone(),
                        format!("{value:?}"),
                    );
                }
            });
        ui.add(
            egui::TextEdit::multiline(&mut self.state.creative_prompt)
                .hint_text("Describe the creative project")
                .desired_rows(5),
        );
        if ui.button("Generate plan").clicked() {
            let prompt = self.state.creative_prompt.trim().to_string();
            if prompt.is_empty() {
                self.state.show_info("Enter a creative prompt first.");
            } else {
                self.state.set_loading("Generating creative project...");
                match self
                    .api
                    .run_creative_project(&prompt, self.state.creative_type.clone(), 3)
                {
                    Ok(report) => {
                        self.state.show_success(format!(
                            "Creative project {} created with {} artifacts.",
                            report.title,
                            report.artifacts_created.len()
                        ));
                        self.state.refresh_all(&self.api);
                    }
                    Err(error) => self.state.handle_api_error(error),
                }
                self.state.clear_loading();
            }
        }
    }

    fn tasks_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                match self.api.list_tasks() {
                    Ok(tasks) => {
                        self.state.tasks = tasks;
                        self.state.show_success("Tasks refreshed.");
                    }
                    Err(error) => self.state.handle_api_error(error),
                }
            }
            if ui.button("New task").clicked() {
                self.state.switch_view(ActiveView::Home);
            }
        });
        for task in &self.state.tasks {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&task.title).strong());
                    ui.label(&task.status);
                    if task.demo {
                        ui.label(RichText::new("Demo only").weak());
                    }
                });
                ui.label(&task.subtitle);
            });
        }
    }

    fn artifacts_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Refresh").clicked() {
                match self.api.list_artifacts() {
                    Ok(artifacts) => self.state.artifacts = artifacts,
                    Err(error) => self.state.handle_api_error(error),
                }
                match self.api.list_artifact_packs() {
                    Ok(packs) => self.state.artifact_packs = packs,
                    Err(error) => self.state.handle_api_error(error),
                }
                self.state.show_success("Artifacts refreshed.");
            }
            if ui.button("Export latest").clicked() {
                match self.api.export_latest_package() {
                    Ok(result) => self.state.show_success(result.message),
                    Err(error) => self.state.handle_api_error(error),
                }
            }
            if ui.button("Inspect latest").clicked() {
                self.state.show_info(
                    "Inspect latest is available through artifact details when artifacts exist.",
                );
            }
        });
        ui.heading("Artifacts");
        if self.state.artifacts.artifacts.is_empty() {
            ui.label("No artifacts yet.");
        }
        for artifact in &self.state.artifacts.artifacts {
            ui.label(format!(
                "{} - {:?} - score {:.2}",
                artifact.session_id, artifact.artifact_type, artifact.validation_score
            ));
        }
        ui.heading("Packs");
        for pack in &self.state.artifact_packs.packs {
            ui.label(format!(
                "{} - score {:.2}",
                pack.title, pack.validation_score
            ));
        }
    }

    fn memory_view(&mut self, ui: &mut egui::Ui) {
        if ui.button("Refresh memory").clicked() {
            match self.api.list_memories() {
                Ok(memories) => {
                    self.state.memories = memories;
                    self.state.show_success("Memory refreshed.");
                }
                Err(error) => self.state.handle_api_error(error),
            }
        }
        for row in &self.state.memories {
            ui.label(format!(
                "{} - {} - importance {:.2}",
                row.memory_type, row.title, row.importance
            ));
        }
    }

    fn safety_view(&mut self, ui: &mut egui::Ui) {
        if let Some(status) = &self.state.safety_status {
            ui.label(format!("Sandbox active: {}", status.sandbox_enabled));
            ui.label(format!("Network: {}", status.network_default));
            ui.label(&status.safety_note);
        }
        ui.horizontal(|ui| {
            if ui.button("Run Doctor").clicked() {
                self.run_doctor();
            }
            if ui.button("Regression Check").clicked() {
                self.state.set_loading("Running regression check...");
                match self.api.run_regression_check() {
                    Ok(report) => self.state.show_success(format!(
                        "Regression {}: {} passed, {} failed.",
                        report.status, report.checks_passed, report.checks_failed
                    )),
                    Err(error) => self.state.handle_api_error(error),
                }
                self.state.clear_loading();
            }
            if ui.button("Maintain").clicked() {
                self.state.set_loading("Running maintenance...");
                match self.api.run_maintain() {
                    Ok(result) => self.state.show_success(result.message),
                    Err(error) => self.state.handle_api_error(error),
                }
                self.state.clear_loading();
            }
        });
    }

    fn run_doctor(&mut self) {
        self.state.set_loading("Running doctor...");
        match self.api.run_doctor() {
            Ok(report) => {
                self.state.show_success(format!(
                    "Doctor completed: {} issues, {} critical.",
                    report.issues_found, report.critical
                ));
                self.state.error_banner = None;
            }
            Err(error) => self.state.handle_api_error(error),
        }
        self.state.clear_loading();
    }

    fn system_view(&mut self, ui: &mut egui::Ui) {
        if ui.button("Refresh status").clicked() {
            self.state.refresh_status(&self.api);
            self.state.show_success("System status refreshed.");
        }
        if let Some(status) = &self.state.brain_status {
            ui.label(format!("Version: {}", status.version));
            ui.label(format!("Neurons: {}", status.neurons));
            ui.label(format!("Synapses: {}", status.synapses));
            ui.label(format!(
                "Conversation sessions: {}",
                status.conversation_sessions_count
            ));
            ui.label(format!("Artifacts: {}", status.artifacts_count));
            ui.label(format!("Doctor: {}", status.doctor_health_summary));
        }
        match self.api.get_current_workspace() {
            Ok(workspace) => {
                ui.separator();
                ui.label(format!("Workspace: {}", workspace.root.display()));
                ui.label(format!("Data: {}", workspace.data_dir.display()));
                ui.label(format!("Sandbox: {}", workspace.sandbox_dir.display()));
            }
            Err(error) => self.state.handle_api_error(error),
        }
    }

    fn projects_view(&mut self, ui: &mut egui::Ui) {
        if ui.button("Refresh projects").clicked() {
            match self.api.list_projects() {
                Ok(projects) => self.state.projects = projects,
                Err(error) => self.state.handle_api_error(error),
            }
        }
        if self.state.projects.is_empty() {
            ui.label("No projects yet. Create a project from the sidebar.");
        }
        for project in self.state.projects.clone() {
            ui.group(|ui| {
                ui.label(RichText::new(&project.name).strong());
                ui.label(format!("Status: {}", project.status));
                ui.label(&project.summary);
                if ui.button("Create thread in project").clicked() {
                    self.state.current_prompt =
                        format!("Create a new thread inside project {}: ", project.name);
                    self.state.switch_view(ActiveView::Home);
                }
            });
        }
    }

    fn search_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let response = ui.text_edit_singleline(&mut self.state.search_query);
            if response.changed() {
                self.state.run_search(&self.api);
            }
            if ui.button("New task").clicked() {
                self.state.switch_view(ActiveView::Home);
            }
        });
        self.search_results(ui);
    }

    fn search_results(&mut self, ui: &mut egui::Ui) {
        if self.state.search_results.is_empty() {
            ui.label("No local results found.");
        }
        for result in self.state.search_results.clone() {
            if ui
                .button(format!(
                    "{:?}: {} - {}",
                    result.kind, result.title, result.subtitle
                ))
                .clicked()
            {
                self.open_search_result(&result.action, result.kind);
            }
        }
    }

    fn open_search_result(&mut self, action: &str, kind: SearchResultKind) {
        self.state.modal_state = ModalState::None;
        match action {
            "new_task" => self.state.switch_view(ActiveView::Home),
            "open_tasks" => self.state.switch_view(ActiveView::Tasks),
            "open_chat" => self.state.switch_view(ActiveView::Chat),
            "open_artifacts" => self.state.switch_view(ActiveView::Artifacts),
            "open_memory" => self.state.switch_view(ActiveView::Memory),
            "open_projects" => self.state.switch_view(ActiveView::Projects),
            _ => {
                self.state.show_info(format!(
                    "{kind:?} result is visible, but no deeper inspector exists in v0.0.4."
                ));
            }
        }
    }

    fn mode_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Mode")
            .selected_text(format!("{:?}", self.state.selected_mode))
            .show_ui(ui, |ui| {
                for mode in [
                    ConversationMode::Standard,
                    ConversationMode::Debate,
                    ConversationMode::Teacher,
                    ConversationMode::Socratic,
                    ConversationMode::Critic,
                    ConversationMode::Planner,
                    ConversationMode::Architect,
                    ConversationMode::Debugger,
                    ConversationMode::Creative,
                    ConversationMode::Summarizer,
                    ConversationMode::SafetyReview,
                    ConversationMode::ProductManager,
                    ConversationMode::Coach,
                ] {
                    ui.selectable_value(
                        &mut self.state.selected_mode,
                        mode.clone(),
                        format!("{mode:?}"),
                    );
                }
            });
    }

    fn personality_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Personality")
            .selected_text(format!("{:?}", self.state.selected_personality))
            .show_ui(ui, |ui| {
                for profile in [
                    PersonalityProfile::Balanced,
                    PersonalityProfile::Friendly,
                    PersonalityProfile::Technical,
                    PersonalityProfile::Concise,
                    PersonalityProfile::Mentor,
                ] {
                    ui.selectable_value(
                        &mut self.state.selected_personality,
                        profile.clone(),
                        format!("{profile:?}"),
                    );
                }
            });
    }

    fn autonomy_selector(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("Autonomy")
            .selected_text(format!("{:?}", self.state.selected_autonomy_level))
            .show_ui(ui, |ui| {
                for level in [
                    AutonomyLevel::Assisted,
                    AutonomyLevel::Standard,
                    AutonomyLevel::High,
                    AutonomyLevel::FullBounded,
                    AutonomyLevel::ReviewOnly,
                    AutonomyLevel::RepairOnly,
                    AutonomyLevel::Studio,
                    AutonomyLevel::Executive,
                ] {
                    ui.selectable_value(
                        &mut self.state.selected_autonomy_level,
                        level.clone(),
                        format!("{level:?}"),
                    );
                }
            });
    }

    fn command_palette(&mut self, ctx: &egui::Context) {
        if self.state.modal_state != ModalState::CommandPalette {
            return;
        }
        egui::Window::new("Command palette")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.command_query);
                for (name, view) in [
                    ("New task", Some(ActiveView::Home)),
                    ("Open Chat", Some(ActiveView::Chat)),
                    ("Open Autonomy", Some(ActiveView::Autonomy)),
                    ("Open Creative Studio", Some(ActiveView::CreativeStudio)),
                    ("Open Library", Some(ActiveView::Library)),
                    ("Open Settings", Some(ActiveView::Settings)),
                    ("Run Doctor", None),
                    ("Run Regression Check", None),
                    ("Export Latest Package", None),
                ] {
                    if !self.command_query.is_empty()
                        && !name
                            .to_ascii_lowercase()
                            .contains(&self.command_query.to_ascii_lowercase())
                    {
                        continue;
                    }
                    if ui.button(name).clicked() {
                        match name {
                            "Run Doctor" => self.run_doctor(),
                            "Run Regression Check" => match self.api.run_regression_check() {
                                Ok(report) => self.state.show_success(format!(
                                    "Regression {}: {} passed.",
                                    report.status, report.checks_passed
                                )),
                                Err(error) => self.state.handle_api_error(error),
                            },
                            "Export Latest Package" => match self.api.export_latest_package() {
                                Ok(result) => self.state.show_success(result.message),
                                Err(error) => self.state.handle_api_error(error),
                            },
                            _ => {
                                if let Some(view) = view {
                                    self.state.switch_view(view);
                                }
                            }
                        }
                        self.state.modal_state = ModalState::None;
                    }
                }
                if ui.button("Close").clicked() {
                    self.state.modal_state = ModalState::None;
                }
            });
    }

    fn search_modal(&mut self, ctx: &egui::Context) {
        if self.state.modal_state != ModalState::Search {
            return;
        }
        egui::Window::new("Search tasks")
            .collapsible(false)
            .resizable(true)
            .default_width(680.0)
            .default_height(430.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let response = ui.text_edit_singleline(&mut self.state.search_query);
                    if response.changed() {
                        self.state.run_search(&self.api);
                    }
                    if ui.button("Close").clicked() {
                        self.state.modal_state = ModalState::None;
                    }
                });
                if ui.button("New task").clicked() {
                    self.state.modal_state = ModalState::None;
                    self.state.switch_view(ActiveView::Home);
                }
                self.search_results(ui);
            });
    }

    fn new_project_modal(&mut self, ctx: &egui::Context) {
        if self.state.modal_state != ModalState::NewProject {
            return;
        }
        egui::Window::new("Create project")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Project name or brief");
                ui.text_edit_singleline(&mut self.new_project_prompt);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.state.modal_state = ModalState::None;
                    }
                    if ui.button("Create").clicked() {
                        let prompt = if self.new_project_prompt.trim().is_empty() {
                            "Create a sandbox project".to_string()
                        } else {
                            format!(
                                "Create a sandbox project: {}",
                                self.new_project_prompt.trim()
                            )
                        };
                        match self.api.create_task(&prompt) {
                            Ok(task) => {
                                self.state.tasks.push(task);
                                self.state.show_success("Project task created.");
                                self.state.modal_state = ModalState::None;
                                self.state.switch_view(ActiveView::Tasks);
                            }
                            Err(error) => self.state.handle_api_error(error),
                        }
                    }
                });
            });
    }

    fn disabled_scheduled_modal(&mut self, ctx: &egui::Context) {
        if self.state.modal_state != ModalState::DisabledScheduled {
            return;
        }
        egui::Window::new("Scheduled tasks disabled")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Scheduled autonomous tasks are disabled in v0.0.4.");
                ui.label("Onyx Brain will not start hidden background work.");
                if ui.button("OK").clicked() {
                    self.state.modal_state = ModalState::None;
                }
            });
    }

    fn draw_toasts(&mut self, ctx: &egui::Context) {
        self.state.prune_toasts();
        if self.state.toast_messages.is_empty() {
            return;
        }
        egui::Area::new("toasts".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -16.0])
            .show(ctx, |ui| {
                for toast in &self.state.toast_messages {
                    let color = match toast.kind {
                        ToastKind::Info => egui::Color32::from_rgb(45, 50, 58),
                        ToastKind::Success => egui::Color32::from_rgb(35, 74, 45),
                        ToastKind::Warning => egui::Color32::from_rgb(88, 67, 32),
                        ToastKind::Error => egui::Color32::from_rgb(88, 40, 40),
                    };
                    egui::Frame::group(ui.style()).fill(color).show(ui, |ui| {
                        ui.label(&toast.message);
                    });
                }
            });
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

impl eframe::App for OnyxGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|input| input.key_pressed(egui::Key::K) && input.modifiers.ctrl) {
            self.state.modal_state = ModalState::CommandPalette;
        }

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(230.0)
            .show(ctx, |ui| self.sidebar(ui));

        egui::CentralPanel::default().show(ctx, |ui| {
            self.top_bar(ctx, ui);
            egui::ScrollArea::vertical().show(ui, |ui| match self.state.active_view {
                ActiveView::Home => self.home_view(ui),
                ActiveView::Scheduled => self.scheduled_view(ui),
                ActiveView::Search => self.search_view(ui),
                ActiveView::Library => self.library_view(ui),
                ActiveView::Projects => self.projects_view(ui),
                ActiveView::Settings => self.settings_view(ctx, ui),
                ActiveView::Chat => self.chat_view(ui),
                ActiveView::Autonomy => self.autonomy_view(ui),
                ActiveView::CreativeStudio => self.creative_view(ui),
                ActiveView::Tasks => self.tasks_view(ui),
                ActiveView::Artifacts => self.artifacts_view(ui),
                ActiveView::Memory => self.memory_view(ui),
                ActiveView::Safety => self.safety_view(ui),
                ActiveView::System => self.system_view(ui),
            });
        });

        self.command_palette(ctx);
        self.search_modal(ctx);
        self.new_project_modal(ctx);
        self.disabled_scheduled_modal(ctx);
        self.draw_toasts(ctx);
    }
}

fn apply_theme(ctx: &egui::Context, theme: ThemeMode) {
    match theme {
        ThemeMode::Light => ctx.set_visuals(egui::Visuals::light()),
        ThemeMode::Dark | ThemeMode::Auto => ctx.set_visuals(egui::Visuals::dark()),
    }
}
