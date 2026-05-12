use crate::{
    agency::ProjectState,
    core::RouteTrace,
    memory::{MemoryItem, MemoryType},
};

pub fn extract_skills_from_project(
    project_state: &ProjectState,
    _route_trace: Option<&RouteTrace>,
    final_report: &str,
) -> Vec<MemoryItem> {
    if project_state.status != "Completed" {
        return Vec::new();
    }
    let mut skills = Vec::new();
    let mut push_skill = |id: &str, title: &str, content: &str| {
        if !skills.iter().any(|skill: &MemoryItem| skill.title == title) {
            let mut memory = MemoryItem::new(
                id,
                MemoryType::Procedural,
                title,
                content,
                vec![
                    "rust".to_string(),
                    "project".to_string(),
                    "skill".to_string(),
                    "workflow".to_string(),
                ],
                vec!["goal_create_project".to_string()],
            );
            memory.importance = 0.85;
            skills.push(memory);
        }
    };

    push_skill(
        "skill_create_rust_cli_project",
        "Create Rust CLI project",
        "Create Cargo.toml, src/main.rs, src/lib.rs, run cargo check, and run cargo test.",
    );
    if project_state
        .files_modified
        .iter()
        .chain(project_state.files_created.iter())
        .any(|file| file.contains("lib.rs"))
    {
        push_skill(
            "skill_add_calculator_operation_to_rust_library",
            "Add calculator operation to Rust library",
            "Add deterministic arithmetic functions to src/lib.rs and verify them with tests.",
        );
    }
    if project_state
        .files_created
        .iter()
        .chain(project_state.files_modified.iter())
        .any(|file| file.contains("test"))
        || final_report.to_lowercase().contains("test")
    {
        push_skill(
            "skill_add_rust_unit_tests",
            "Add Rust unit tests",
            "Write unit or integration tests and run cargo test.",
        );
    }
    if project_state
        .files_created
        .iter()
        .chain(project_state.files_modified.iter())
        .any(|file| file.to_lowercase().contains("readme"))
    {
        push_skill(
            "skill_update_readme_for_rust_project",
            "Update README for Rust project",
            "Document project purpose and run/test commands in README.md.",
        );
    }
    push_skill(
        "skill_run_cargo_check_and_cargo_test",
        "Run cargo check and cargo test",
        "Use the safe terminal allowlist to run cargo check and cargo test inside the sandbox project.",
    );
    skills
}
