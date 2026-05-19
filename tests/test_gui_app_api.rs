use onyx_brain::{
    app_api::AppApi,
    conversation::{ConversationMode, PersonalityProfile},
    gui::settings::{load_or_default, save, ThemeMode},
};

#[test]
fn gui_settings_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let mut settings = load_or_default(dir.path()).unwrap();
    settings.theme_mode = ThemeMode::Light;
    settings.default_personality = PersonalityProfile::Technical;
    save(dir.path(), &settings).unwrap();

    let loaded = load_or_default(dir.path()).unwrap();
    assert_eq!(loaded.theme_mode, ThemeMode::Light);
    assert_eq!(loaded.default_personality, PersonalityProfile::Technical);
}

#[test]
fn corrupt_gui_settings_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let path = onyx_brain::gui::settings::settings_path(dir.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{not valid json").unwrap();

    let recovered = load_or_default(dir.path()).unwrap();
    assert_eq!(recovered.theme_mode, ThemeMode::Dark);
    let archived = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("corrupt"));
    assert!(archived);
}

#[test]
fn search_returns_known_task_item() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    api.init().unwrap();
    api.create_task("Research customer onboarding").unwrap();

    let results = api.search_all("onboarding").unwrap();
    assert!(results.iter().any(|row| row.title.contains("onboarding")));
}

#[test]
fn theme_mode_changes_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut settings = load_or_default(dir.path()).unwrap();
    settings.theme_mode = ThemeMode::Auto;
    save(dir.path(), &settings).unwrap();
    let loaded = load_or_default(dir.path()).unwrap();
    assert_eq!(loaded.theme_mode, ThemeMode::Auto);
}

#[test]
fn app_api_brain_status_returns_version() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    api.init().unwrap();
    let status = api.get_brain_status().unwrap();
    assert_eq!(status.version, "v0.0.4");
}

#[test]
fn app_api_safety_status_reports_sandbox_active() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    let status = api.get_safety_status().unwrap();
    assert!(status.sandbox_enabled);
    assert!(status.safety_note.contains("Bounded autonomy"));
}

#[test]
fn create_task_creates_safe_local_task() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    let task = api.create_task("Create a safe demo task").unwrap();
    assert_eq!(task.status, "Created");
    assert!(!task.demo);
}

#[test]
fn send_chat_message_returns_response() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    api.init().unwrap();
    let response = api
        .send_chat_message(
            "Hello Onyx, what can you do?",
            ConversationMode::Standard,
            PersonalityProfile::Balanced,
        )
        .unwrap();
    assert!(!response.response.trim().is_empty());
}

#[test]
fn list_artifacts_handles_empty_state() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    api.init().unwrap();
    let artifacts = api.list_artifacts().unwrap();
    assert_eq!(artifacts.count, 0);
}

#[test]
fn run_doctor_returns_structured_result() {
    let dir = tempfile::tempdir().unwrap();
    let api = AppApi::new(dir.path());
    api.init().unwrap();
    let report = api.run_doctor().unwrap();
    assert!(report.reliability_state_health >= 0.0);
    assert!(!report.recommendation.is_empty());
}
