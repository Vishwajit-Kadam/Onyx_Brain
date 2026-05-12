use serde::{Deserialize, Serialize};

use crate::agency::ParsedGoal;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CargoValidationPolicy {
    pub run_cargo_check: bool,
    pub run_cargo_test: bool,
    pub reason: String,
    pub confidence: f32,
}

pub fn decide_cargo_validation(
    parsed: &ParsedGoal,
    files_created: &[String],
    files_modified: &[String],
    previous_failure: bool,
) -> CargoValidationPolicy {
    let touched = files_created
        .iter()
        .chain(files_modified.iter())
        .map(|file| file.replace('\\', "/").to_lowercase())
        .collect::<Vec<_>>();
    let code_touched = touched.iter().any(|file| {
        file.ends_with(".rs")
            || file == "cargo.toml"
            || file.contains("/src/")
            || file.contains("/tests/")
    });
    let tests_touched = touched.iter().any(|file| file.contains("test"));
    let readme_only = !touched.is_empty()
        && touched
            .iter()
            .all(|file| file.ends_with("readme.md") || file == ".onyx_edits.log");

    if readme_only && !previous_failure {
        return CargoValidationPolicy {
            run_cargo_check: false,
            run_cargo_test: false,
            reason: "README-only change; safe to skip Cargo validation".to_string(),
            confidence: 0.85,
        };
    }

    CargoValidationPolicy {
        run_cargo_check: code_touched || previous_failure,
        run_cargo_test: parsed.wants_tests || tests_touched || code_touched || previous_failure,
        reason: if code_touched {
            "Rust code or tests changed; cargo check/test preserved".to_string()
        } else if previous_failure {
            "previous failure requires validation".to_string()
        } else {
            "no Rust code changes detected".to_string()
        },
        confidence: 0.9,
    }
}
