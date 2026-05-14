use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggerResponse {
    pub problem_summary: String,
    pub likely_causes: Vec<String>,
    pub safe_checks: Vec<String>,
    pub recommended_commands: Vec<String>,
    pub what_not_to_do: Vec<String>,
    pub next_step: String,
}

pub fn debugger_response(input: &str) -> DebuggerResponse {
    let lower = input.to_lowercase();
    let likely_causes = if lower.contains("unresolved import") {
        vec![
            "Module path or `pub mod` declaration is missing.".to_string(),
            "The import name does not match the actual file/module name.".to_string(),
            "A feature was added but not re-exported from `mod.rs` or `lib.rs`.".to_string(),
        ]
    } else {
        vec![
            "Recent code change introduced a compile error.".to_string(),
            "Test expectations may not match current public behavior.".to_string(),
        ]
    };
    DebuggerResponse {
        problem_summary: input.to_string(),
        likely_causes,
        safe_checks: vec![
            "Read the exact compiler error and file path.".to_string(),
            "Inspect the relevant module declarations.".to_string(),
            "Check whether tests expect older behavior.".to_string(),
        ],
        recommended_commands: vec![
            "cargo fmt".to_string(),
            "cargo check".to_string(),
            "cargo test -- --nocapture".to_string(),
        ],
        what_not_to_do: vec![
            "Do not run destructive cleanup commands.".to_string(),
            "Do not bypass sandbox or allowlist rules.".to_string(),
            "Do not delete user files to make tests pass.".to_string(),
        ],
        next_step: "Run `cargo check` and inspect the first compiler error.".to_string(),
    }
}

pub fn render_debugger(response: &DebuggerResponse) -> String {
    format!(
        "# Debugger Mode\n\n## Problem Summary\n{}\n\n## Likely Causes\n{}\n\n## Safe Checks\n{}\n\n## Recommended Commands\n{}\n\n## What Not To Do\n{}\n\n## Next Step\n{}\n",
        response.problem_summary,
        bullets(&response.likely_causes),
        bullets(&response.safe_checks),
        bullets(&response.recommended_commands),
        bullets(&response.what_not_to_do),
        response.next_step
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
