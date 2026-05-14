use anyhow::Result;
use eframe::egui::{
    self, Align, Color32, ComboBox, Context, Frame, Layout, Margin, RichText, Rounding, ScrollArea,
    Stroke, TextEdit, Ui, Vec2,
};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};

use crate::agency::AutonomyLevel;
use crate::app_api::{AppApi, MemorySummaryRow, SafetyStatus};
use crate::conversation::ConversationMode;
use crate::core::brain::BrainStatus;
use crate::gui::{default_theme, GuiState};
use crate::ONYX_VERSION;

const DEMO_LABEL: &str = "Demo preview where backend wiring is still read-only.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Chat,
    Autonomy,
    Creative,
    Tasks,
    Artifacts,
    Memory,
    Safety,
    System,
    Settings,
}

impl View {
    fn label(self) -> &'static str {
        match self {
            Self::Chat => "Chat",
            Self::Autonomy => "Autonomy",
            Self::Creative => "Creative Studio",
            Self::Tasks => "Tasks",
            Self::Artifacts => "Artifacts",
            Self::Memory => "Memory",
            Self::Safety => "Safety",
            Self::System => "System",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone)]
struct ChatLine {
    role: String,
    text: String,
    quality: Option<f32>,
}

#[derive(Debug, Clone)]
struct StatusSnapshot {
    version: String,
    neurons: usize,
    synapses: usize,
    memories: usize,
    sessions: usize,
    artifacts: usize,
    reliability: f32,
    autonomy: f32,
    doctor: String,
    demo_note: Option<String>,
}

impl Default for StatusSnapshot {
    fn default() -> Self {
        Self {
            version: ONYX_VERSION.to_string(),
            neurons: 0,
            synapses: 0,
            memories: 0,
            sessions: 0,
            artifacts: 0,
            reliability: 0.0,
            autonomy: 0.0,
            doctor: "Not checked".to_string(),
            demo_note: None,
        }
    }
}

pub struct OnyxNativeApp {
    api: AppApi,
    active: View,
    chat_input: String,
    chat_mode: ConversationMode,
    personality: String,
    chat: Vec<ChatLine>,
    goal_input: String,
    autonomy_level: AutonomyLevel,
    creative_prompt: String,
    creative_type: String,
    duration_days: u32,
    settings_mode: ConversationMode,
    settings_personality: String,
    settings_autonomy: AutonomyLevel,
    theme_name: String,
    run_status: String,
    doctor_status: String,
    active_session: String,
    status: StatusSnapshot,
    safety: SafetyStatus,
    memories: Vec<MemorySummaryRow>,
    pending: Option<Receiver<WorkerResult>>,
}

#[derive(Debug)]
enum WorkerResult {
    Chat { text: String, quality: Option<f32> },
    Autonomy(String),
    Creative(String),
    Doctor(String),
    Regression(String),
    Maintain(String),
}

impl OnyxNativeApp {
    fn new(_cc: &eframe::CreationContext<'_>, root: &Path) -> Self {
        let api = AppApi::new(root);
        let _ = api.init();
        let safety = api.get_safety_status();
        let mut app = Self {
            api,
            active: View::Chat,
            chat_input: String::new(),
            chat_mode: ConversationMode::Standard,
            personality: "Balanced".to_string(),
            chat: vec![ChatLine {
                role: "Onyx".to_string(),
                text: "Native Onyx Brain shell ready. Bounded autonomy and sandbox-first actions are active.".to_string(),
                quality: None,
            }],
            goal_input: String::new(),
            autonomy_level: AutonomyLevel::Standard,
            creative_prompt: String::new(),
            creative_type: "Planning artifact".to_string(),
            duration_days: 7,
            settings_mode: ConversationMode::Standard,
            settings_personality: "Balanced".to_string(),
            settings_autonomy: AutonomyLevel::Standard,
            theme_name: default_theme().name,
            run_status: "Idle".to_string(),
            doctor_status: "Not checked".to_string(),
            active_session: "No active session".to_string(),
            status: StatusSnapshot::default(),
            safety,
            memories: Vec::new(),
            pending: None,
        };
        app.refresh_read_models();
        app
    }

    fn refresh_read_models(&mut self) {
        self.status = self.load_status();
        self.memories = self.api.get_memory_summary().unwrap_or_else(|_| {
            vec![
                MemorySummaryRow {
                    title: "semantic memories: 0".to_string(),
                    memory_type: "Semantic".to_string(),
                    importance: 0.2,
                },
                MemorySummaryRow {
                    title: "procedural memories: 0".to_string(),
                    memory_type: "Procedural".to_string(),
                    importance: 0.2,
                },
            ]
        });
        if let Ok(sessions) = self.api.get_sessions() {
            self.active_session = sessions
                .first()
                .map(|session| format!("Session {}", session.session_id))
                .unwrap_or_else(|| "No active session".to_string());
        }
    }

    fn load_status(&self) -> StatusSnapshot {
        match self.api.get_brain_status() {
            Ok(status) => status_snapshot(status),
            Err(error) => StatusSnapshot {
                demo_note: Some(format!("Status unavailable: {error}")),
                ..StatusSnapshot::default()
            },
        }
    }

    fn apply_theme(ctx: &Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = Color32::from_rgb(18, 18, 18);
        visuals.panel_fill = Color32::from_rgb(20, 21, 21);
        visuals.faint_bg_color = Color32::from_rgb(36, 37, 38);
        visuals.extreme_bg_color = Color32::from_rgb(12, 13, 13);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 27, 27);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 39, 40);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 49, 51);
        visuals.widgets.active.bg_fill = Color32::from_rgb(55, 58, 64);
        visuals.selection.bg_fill = Color32::from_rgb(79, 136, 255);
        ctx.set_visuals(visuals);
    }

    fn poll_worker(&mut self) {
        let Some(rx) = self.pending.take() else {
            return;
        };
        match rx.try_recv() {
            Ok(result) => {
                self.run_status = "Idle".to_string();
                match result {
                    WorkerResult::Chat { text, quality } => self.chat.push(ChatLine {
                        role: "Onyx".to_string(),
                        text,
                        quality,
                    }),
                    WorkerResult::Autonomy(text)
                    | WorkerResult::Creative(text)
                    | WorkerResult::Maintain(text) => self.chat.push(ChatLine {
                        role: "System".to_string(),
                        text,
                        quality: None,
                    }),
                    WorkerResult::Doctor(text) => {
                        self.doctor_status = text.clone();
                        self.chat.push(ChatLine {
                            role: "System".to_string(),
                            text,
                            quality: None,
                        });
                    }
                    WorkerResult::Regression(text) => self.chat.push(ChatLine {
                        role: "System".to_string(),
                        text,
                        quality: None,
                    }),
                }
                self.refresh_read_models();
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.pending = Some(rx);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.run_status = "Worker disconnected".to_string();
            }
        }
    }

    fn spawn_worker(
        &mut self,
        status: &str,
        job: impl FnOnce(AppApi) -> WorkerResult + Send + 'static,
    ) {
        if self.pending.is_some() {
            self.run_status = "Busy".to_string();
            return;
        }
        let api = self.api.clone();
        let (tx, rx) = mpsc::channel();
        self.pending = Some(rx);
        self.run_status = status.to_string();
        std::thread::spawn(move || {
            let result = job(api);
            let _ = tx.send(result);
        });
    }

    fn send_chat(&mut self) {
        let input = self.chat_input.trim().to_string();
        if input.is_empty() {
            return;
        }
        let mode = self.chat_mode.clone();
        self.chat.push(ChatLine {
            role: "You".to_string(),
            text: input.clone(),
            quality: None,
        });
        self.chat_input.clear();
        self.spawn_worker("Thinking", move |api| {
            match api.send_chat_message(&input, mode) {
                Ok(output) => WorkerResult::Chat {
                    text: output.response,
                    quality: Some(output.quality.overall),
                },
                Err(error) => WorkerResult::Chat {
                    text: format!("Chat failed safely: {error}"),
                    quality: None,
                },
            }
        });
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(strong("Onyx Brain"));
            ui.label(muted(ONYX_VERSION));
            ui.separator();
            badge(ui, self.active.label(), INFO);
            badge(ui, "Sandbox active", GOOD);
            badge(ui, &self.doctor_status, INFO);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(
                    ui,
                    &self.run_status,
                    if self.pending.is_some() {
                        RUNNING
                    } else {
                        INFO
                    },
                );
                ui.label(muted(&self.active_session));
            });
        });
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        ui.add_space(8.0);
        ui.label(RichText::new("Onyx Brain").size(17.0).strong());
        ui.label(muted(
            "Consciousness-inspired executive layer, not real consciousness",
        ));
        ui.add_space(16.0);

        for view in [
            View::Chat,
            View::Autonomy,
            View::Creative,
            View::Tasks,
            View::Artifacts,
            View::Memory,
            View::Safety,
            View::System,
            View::Settings,
        ] {
            if nav_button(ui, self.active == view, view.label()).clicked() {
                self.active = view;
            }
        }

        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            ui.add_space(8.0);
            if ui.button("Refresh").clicked() {
                self.refresh_read_models();
            }
            ui.label(muted("Native Rust desktop shell."));
        });
    }

    fn central(&mut self, ui: &mut Ui) {
        match self.active {
            View::Chat => self.chat_view(ui),
            View::Autonomy => self.autonomy_view(ui),
            View::Creative => self.creative_view(ui),
            View::Tasks => self.tasks_view(ui),
            View::Artifacts => self.artifacts_view(ui),
            View::Memory => self.memory_view(ui),
            View::Safety => self.safety_view(ui),
            View::System => self.system_view(ui),
            View::Settings => self.settings_view(ui),
        }
    }

    fn inspector(&mut self, ui: &mut Ui) {
        ui.heading("Inspector");
        ui.add_space(8.0);
        stat_card(
            ui,
            "Run status",
            &self.run_status,
            "Current foreground action",
        );
        stat_card(
            ui,
            "Safety",
            "Bounded",
            "Sandbox active, allowlisted commands only",
        );
        stat_card(
            ui,
            "Doctor",
            &self.doctor_status,
            "Run Doctor updates this badge",
        );
        ui.add_space(8.0);
        ui.label(muted(
            "Creative planning artifacts only. No safety-disabling controls are exposed.",
        ));
    }

    fn chat_view(&mut self, ui: &mut Ui) {
        page_header(ui, "Chat", "Mode-aware conversation through AppApi.");
        ui.horizontal(|ui| {
            mode_combo(ui, "Mode", &mut self.chat_mode);
            string_combo(
                ui,
                "Personality",
                &mut self.personality,
                &["Balanced", "Friendly", "Technical", "Concise", "Mentor"],
            );
        });
        ui.add_space(10.0);
        card(ui, |ui| {
            ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for line in &self.chat {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(strong(&line.role));
                        if let Some(score) = line.quality {
                            badge(ui, &format!("quality {:.0}%", score * 100.0), GOOD);
                        }
                    });
                    ui.label(&line.text);
                    ui.add_space(10.0);
                }
            });
        });
        ui.add_space(10.0);
        let response = ui.add(
            TextEdit::multiline(&mut self.chat_input)
                .hint_text("Ask Onyx anything. Ctrl+Enter sends.")
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );
        let send_shortcut = response.has_focus()
            && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.ctrl);
        ui.horizontal(|ui| {
            if ui.button("Send").clicked() || send_shortcut {
                self.send_chat();
            }
            ui.label(muted(
                "Bounded autonomy; responses come from the local Onyx Brain runtime.",
            ));
        });
    }

    fn autonomy_view(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            "Autonomy",
            "Run bounded goals through safe AppApi entry points.",
        );
        autonomy_combo(ui, "Autonomy level", &mut self.autonomy_level);
        ui.add(
            TextEdit::multiline(&mut self.goal_input)
                .hint_text("Describe a bounded goal")
                .desired_rows(5)
                .desired_width(f32::INFINITY),
        );
        if ui.button("Run bounded goal").clicked() {
            let input = self.goal_input.trim().to_string();
            let level = self.autonomy_level.clone();
            self.spawn_worker("Running bounded goal", move |api| {
                if input.is_empty() {
                    return WorkerResult::Autonomy(
                        "Add a goal before running autonomy.".to_string(),
                    );
                }
                match api.run_autonomous_goal(&input, level) {
                    Ok(result) => WorkerResult::Autonomy(format!(
                        "Autonomy completed: status {:?}, tasks {}, report {}",
                        result.status, result.tasks_completed, result.final_report_path
                    )),
                    Err(error) => {
                        WorkerResult::Autonomy(format!("Autonomy stopped safely: {error}"))
                    }
                }
            });
        }
        ui.columns(3, |columns| {
            metric(&mut columns[0], "Progress", "Idle / running", 0.25);
            metric(&mut columns[1], "Report grade", "Pending", 0.0);
            metric(&mut columns[2], "Validation", "Safe defaults", 0.82);
        });
    }

    fn creative_view(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            "Creative Studio",
            "Plan creative artifacts without unsafe execution.",
        );
        string_combo(
            ui,
            "Project type",
            &mut self.creative_type,
            &[
                "Planning artifact",
                "Story outline",
                "Design brief",
                "Research plan",
            ],
        );
        ui.add(egui::Slider::new(&mut self.duration_days, 1..=30).text("duration days"));
        ui.add(
            TextEdit::multiline(&mut self.creative_prompt)
                .hint_text("Creative brief")
                .desired_rows(5)
                .desired_width(f32::INFINITY),
        );
        if ui.button("Generate plan").clicked() {
            let prompt = format!(
                "{}\nType: {}\nDuration: {} days",
                self.creative_prompt.trim(),
                self.creative_type,
                self.duration_days
            );
            self.spawn_worker("Generating creative plan", move |api| {
                if prompt.trim().is_empty() {
                    return WorkerResult::Creative("Add a creative brief first.".to_string());
                }
                match api.run_creative_project(&prompt) {
                    Ok(_) => WorkerResult::Creative(
                        "Creative planning artifacts generated through AppApi.".to_string(),
                    ),
                    Err(error) => {
                        WorkerResult::Creative(format!("Creative plan failed safely: {error}"))
                    }
                }
            });
        }
        ui.separator();
        checklist(
            ui,
            &[
                "Brief parsed",
                "Plan outline",
                "Deliverables list",
                "Review notes",
            ],
        );
    }

    fn tasks_view(&mut self, ui: &mut Ui) {
        page_header(ui, "Task Board", DEMO_LABEL);
        ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for (title, items, color) in [
                    ("Planned", vec!["Review goal", "Prepare sandbox"], INFO),
                    ("Running", vec!["None"], RUNNING),
                    ("Completed", vec!["Native shell compile target"], GOOD),
                    ("Failed", vec!["None"], BAD),
                    ("Blocked", vec!["None"], WARN),
                    ("SafetyStopped", vec!["None"], WARN),
                ] {
                    board_column(ui, title, &items, color);
                }
            });
        });
    }

    fn artifacts_view(&mut self, ui: &mut Ui) {
        page_header(ui, "Artifact Browser", DEMO_LABEL);
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut String::from("All artifacts"));
            let _ = ui.button("Export selected");
        });
        ui.columns(3, |columns| {
            artifact_card(
                &mut columns[0],
                "Latest report",
                "validation 82%",
                "quality 88%",
            );
            artifact_card(
                &mut columns[1],
                "Creative plan",
                "validation pending",
                "quality demo",
            );
            artifact_card(
                &mut columns[2],
                "Session package",
                "export ready",
                "quality demo",
            );
        });
        card(ui, |ui| {
            ui.heading("Preview");
            ui.label("Select an artifact to preview. Export uses existing safe backend commands when available.");
        });
    }

    fn memory_view(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            "Memory Browser",
            "Read-only memory summary from AppApi.",
        );
        let mut search = String::new();
        ui.text_edit_singleline(&mut search);
        for row in &self.memories {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(&row.memory_type);
                    badge(
                        ui,
                        &format!("importance {:.0}%", row.importance * 100.0),
                        INFO,
                    );
                });
                ui.label(&row.title);
                ui.label(muted(
                    "Confidence display is summarized; detailed memory browsing can be wired next.",
                ));
            });
        }
    }

    fn safety_view(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            "Safety",
            "Display-only safety posture and safe maintenance actions.",
        );
        ui.columns(3, |columns| {
            stat_card(&mut columns[0], "Sandbox", "Active", "Bounded app actions");
            stat_card(
                &mut columns[1],
                "Network",
                &self.safety.network_default,
                "Native app path",
            );
            stat_card(
                &mut columns[2],
                "Commands",
                "Allowlisted",
                &self.safety.allowlisted_commands.join(", "),
            );
        });
        ui.horizontal(|ui| {
            if ui.button("Run Doctor").clicked() {
                self.spawn_worker("Running doctor", |api| match api.run_doctor() {
                    Ok(report) => WorkerResult::Doctor(format!(
                        "Doctor completed: {} issues, {} critical, {} warnings. Recommendation: {}",
                        report.issues_found, report.critical, report.warnings, report.recommendation
                    )),
                    Err(error) => WorkerResult::Doctor(format!("Doctor failed safely: {error}")),
                });
            }
            if ui.button("Run Regression Check").clicked() {
                self.spawn_worker("Running regression check", |api| match api.run_regression_check() {
                    Ok(report) => WorkerResult::Regression(format!(
                        "Regression check: {} passed, {} failed, status {}",
                        report.checks_passed, report.checks_failed, report.status
                    )),
                    Err(error) => WorkerResult::Regression(format!("Regression check failed safely: {error}")),
                });
            }
            if ui.button("Run Maintain").clicked() {
                self.spawn_worker("Maintain requested", |_api| {
                    WorkerResult::Maintain(
                        "Maintain is intentionally not wired to a one-click mutating GUI action yet. Use the CLI after reviewing scope.".to_string(),
                    )
                });
            }
        });
        card(ui, |ui| {
            ui.label(&self.safety.safety_note);
            ui.label("No button is provided to disable safety.");
        });
    }

    fn system_view(&mut self, ui: &mut Ui) {
        page_header(ui, "System Status", "Runtime state read through AppApi.");
        ui.columns(3, |columns| {
            stat_card(
                &mut columns[0],
                "Version",
                &self.status.version,
                "Native egui shell",
            );
            stat_card(
                &mut columns[1],
                "Neurons",
                &self.status.neurons.to_string(),
                "Disk-backed runtime",
            );
            stat_card(
                &mut columns[2],
                "Synapses",
                &self.status.synapses.to_string(),
                "Sparse routes",
            );
        });
        ui.columns(3, |columns| {
            stat_card(
                &mut columns[0],
                "Memories",
                &self.status.memories.to_string(),
                "All memory types",
            );
            stat_card(
                &mut columns[1],
                "Sessions",
                &self.status.sessions.to_string(),
                "Work sessions",
            );
            stat_card(
                &mut columns[2],
                "Artifacts",
                &self.status.artifacts.to_string(),
                "Generated outputs",
            );
        });
        ui.columns(3, |columns| {
            metric(
                &mut columns[0],
                "Reliability",
                &format!("{:.0}%", self.status.reliability * 100.0),
                self.status.reliability,
            );
            metric(
                &mut columns[1],
                "Autonomy",
                &format!("{:.0}%", self.status.autonomy * 100.0),
                self.status.autonomy,
            );
            stat_card(
                &mut columns[2],
                "Doctor",
                &self.status.doctor,
                "Latest health",
            );
        });
        if let Some(note) = &self.status.demo_note {
            ui.label(muted(note));
        }
    }

    fn settings_view(&mut self, ui: &mut Ui) {
        page_header(
            ui,
            "Settings",
            "Preferences are local UI state; safety settings are display-only.",
        );
        mode_combo(ui, "Default mode", &mut self.settings_mode);
        string_combo(
            ui,
            "Personality",
            &mut self.settings_personality,
            &["Balanced", "Friendly", "Technical", "Concise", "Mentor"],
        );
        autonomy_combo(ui, "Default autonomy", &mut self.settings_autonomy);
        string_combo(ui, "Theme", &mut self.theme_name, &["Onyx Dark"]);
        card(ui, |ui| {
            ui.heading("Safety settings");
            ui.label("Sandbox active");
            ui.label("Network disabled by default");
            ui.label("Allowlisted commands only");
            ui.label(muted("Display-only in the GUI."));
        });
    }
}

impl eframe::App for OnyxNativeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        Self::apply_theme(ctx);
        self.poll_worker();

        egui::TopBottomPanel::top("top_status")
            .exact_height(44.0)
            .frame(panel_frame())
            .show(ctx, |ui| self.top_bar(ui));

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(220.0)
            .frame(sidebar_frame())
            .show(ctx, |ui| self.sidebar(ui));

        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(245.0)
            .width_range(210.0..=320.0)
            .frame(panel_frame())
            .show(ctx, |ui| self.inspector(ui));

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(Color32::from_rgb(17, 18, 18))
                    .inner_margin(Margin::same(18.0)),
            )
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| self.central(ui));
            });
    }
}

pub fn run_native_gui() -> Result<()> {
    let root = std::env::current_dir()?;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Onyx Brain")
            .with_inner_size(Vec2::new(1280.0, 800.0))
            .with_min_inner_size(Vec2::new(1000.0, 700.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Onyx Brain",
        options,
        Box::new(move |cc| Box::new(OnyxNativeApp::new(cc, &root))),
    )
    .map_err(|error| anyhow::anyhow!("native eframe GUI failed to open: {error}"))
}

pub fn launch_gui(root: &Path) -> Result<GuiLaunchReport> {
    let state = GuiState::new();
    let theme = default_theme();
    Ok(GuiLaunchReport {
        launched: true,
        mode: "native eframe/egui".to_string(),
        asset_path: "no web assets required".to_string(),
        views: state.views,
        theme_name: theme.name,
        note: format!(
            "Use `cargo run -- gui` to launch the native Windows app. Release executable: target/release/onyx_brain.exe. Root: {}",
            root.display()
        ),
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuiLaunchReport {
    pub launched: bool,
    pub mode: String,
    pub asset_path: String,
    pub views: Vec<String>,
    pub theme_name: String,
    pub note: String,
}

fn status_snapshot(status: BrainStatus) -> StatusSnapshot {
    StatusSnapshot {
        version: status.version,
        neurons: status.neurons,
        synapses: status.synapses,
        memories: status
            .memories_by_type
            .iter()
            .filter_map(|row| row.rsplit_once(':'))
            .filter_map(|(_, count)| count.trim().parse::<usize>().ok())
            .sum(),
        sessions: status.active_sessions_count,
        artifacts: status.artifacts_count,
        reliability: status.reliability_score.overall,
        autonomy: status.last_autonomy_score,
        doctor: status.doctor_health_summary,
        demo_note: None,
    }
}

struct BadgeTone {
    bg: Color32,
    fg: Color32,
}

const GOOD: BadgeTone = BadgeTone {
    bg: Color32::from_rgb(21, 72, 45),
    fg: Color32::from_rgb(134, 239, 172),
};
const WARN: BadgeTone = BadgeTone {
    bg: Color32::from_rgb(84, 61, 24),
    fg: Color32::from_rgb(253, 224, 71),
};
const BAD: BadgeTone = BadgeTone {
    bg: Color32::from_rgb(86, 32, 37),
    fg: Color32::from_rgb(252, 165, 165),
};
const INFO: BadgeTone = BadgeTone {
    bg: Color32::from_rgb(29, 51, 89),
    fg: Color32::from_rgb(147, 197, 253),
};
const RUNNING: BadgeTone = BadgeTone {
    bg: Color32::from_rgb(46, 38, 92),
    fg: Color32::from_rgb(196, 181, 253),
};

fn panel_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgb(24, 25, 25))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 46, 47)))
        .inner_margin(Margin::symmetric(14.0, 10.0))
}

fn sidebar_frame() -> Frame {
    Frame::none()
        .fill(Color32::from_rgb(28, 29, 29))
        .stroke(Stroke::new(1.0, Color32::from_rgb(48, 49, 49)))
        .inner_margin(Margin::symmetric(12.0, 12.0))
}

fn card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    Frame::none()
        .fill(Color32::from_rgb(30, 31, 32))
        .stroke(Stroke::new(1.0, Color32::from_rgb(54, 55, 57)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, add);
}

fn page_header(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.heading(RichText::new(title).size(24.0).strong());
    ui.label(muted(subtitle));
    ui.add_space(14.0);
}

fn nav_button(ui: &mut Ui, active: bool, label: &str) -> egui::Response {
    let text = if active {
        RichText::new(label)
            .strong()
            .color(Color32::from_rgb(230, 238, 255))
    } else {
        RichText::new(label).color(Color32::from_rgb(186, 188, 188))
    };
    ui.add_sized([196.0, 34.0], egui::SelectableLabel::new(active, text))
}

fn badge(ui: &mut Ui, text: &str, tone: BadgeTone) {
    Frame::none()
        .fill(tone.bg)
        .rounding(Rounding::same(20.0))
        .inner_margin(Margin::symmetric(9.0, 4.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(12.0).strong().color(tone.fg));
        });
}

fn strong(text: &str) -> RichText {
    RichText::new(text)
        .strong()
        .color(Color32::from_rgb(238, 239, 240))
}

fn muted(text: &str) -> RichText {
    RichText::new(text).color(Color32::from_rgb(160, 162, 162))
}

fn stat_card(ui: &mut Ui, title: &str, value: &str, detail: &str) {
    card(ui, |ui| {
        ui.label(muted(title));
        ui.heading(RichText::new(value).size(18.0));
        ui.label(muted(detail));
    });
}

fn metric(ui: &mut Ui, title: &str, value: &str, amount: f32) {
    card(ui, |ui| {
        ui.label(muted(title));
        ui.heading(value);
        ui.add(egui::ProgressBar::new(amount.clamp(0.0, 1.0)).show_percentage());
    });
}

fn board_column(ui: &mut Ui, title: &str, items: &[&str], tone: BadgeTone) {
    ui.vertical(|ui| {
        ui.set_min_width(190.0);
        badge(ui, title, tone);
        ui.add_space(8.0);
        for item in items {
            card(ui, |ui| {
                ui.label(*item);
            });
        }
    });
}

fn artifact_card(ui: &mut Ui, title: &str, validation: &str, quality: &str) {
    card(ui, |ui| {
        ui.heading(title);
        badge(ui, validation, INFO);
        badge(ui, quality, GOOD);
        let _ = ui.button("Preview");
    });
}

fn checklist(ui: &mut Ui, items: &[&str]) {
    card(ui, |ui| {
        for item in items {
            ui.horizontal(|ui| {
                ui.label("[ ]");
                ui.label(*item);
            });
        }
    });
}

fn mode_combo(ui: &mut Ui, label: &str, value: &mut ConversationMode) {
    ComboBox::from_label(label)
        .selected_text(format!("{value:?}"))
        .show_ui(ui, |ui| {
            for option in [
                ConversationMode::Standard,
                ConversationMode::Debate,
                ConversationMode::Teacher,
                ConversationMode::Critic,
                ConversationMode::Planner,
                ConversationMode::Architect,
                ConversationMode::Debugger,
                ConversationMode::Creative,
                ConversationMode::SafetyReview,
            ] {
                ui.selectable_value(value, option.clone(), format!("{option:?}"));
            }
        });
}

fn autonomy_combo(ui: &mut Ui, label: &str, value: &mut AutonomyLevel) {
    ComboBox::from_label(label)
        .selected_text(format!("{value:?}"))
        .show_ui(ui, |ui| {
            for option in [
                AutonomyLevel::Assisted,
                AutonomyLevel::Standard,
                AutonomyLevel::High,
                AutonomyLevel::FullBounded,
                AutonomyLevel::ReviewOnly,
                AutonomyLevel::RepairOnly,
                AutonomyLevel::Studio,
                AutonomyLevel::Executive,
            ] {
                ui.selectable_value(value, option.clone(), format!("{option:?}"));
            }
        });
}

fn string_combo(ui: &mut Ui, label: &str, value: &mut String, options: &[&str]) {
    ComboBox::from_label(label)
        .selected_text(value.as_str())
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(value, (*option).to_string(), *option);
            }
        });
}
