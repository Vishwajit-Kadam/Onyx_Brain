use serde::{Deserialize, Serialize};

use crate::tools::CommandResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticKind {
    CargoCheckPassed,
    CargoTestPassed,
    SyntaxError,
    MissingFunction,
    MissingFile,
    MissingModule,
    TypeMismatch,
    TestFailure,
    DependencyError,
    DivideByZeroRisk,
    UnknownError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub kind: DiagnosticKind,
    pub summary: String,
    pub file_hint: Option<String>,
    pub line_hint: Option<u32>,
    pub suggested_fix: Option<String>,
    pub confidence: f32,
    pub raw_output_excerpt: String,
}

pub fn diagnose_command(command: &CommandResult) -> DiagnosticReport {
    let output = format!("{}\n{}", command.stdout, command.stderr);
    let lower = output.to_lowercase();
    let excerpt = output.lines().take(12).collect::<Vec<_>>().join("\n");
    if command.status == Some(0) && command.command == ["cargo", "check"] {
        return report(
            DiagnosticKind::CargoCheckPassed,
            "cargo check passed",
            None,
            None,
            None,
            1.0,
            excerpt,
        );
    }
    if command.status == Some(0) && command.command == ["cargo", "test"] {
        return report(
            DiagnosticKind::CargoTestPassed,
            "cargo test passed",
            None,
            None,
            None,
            1.0,
            excerpt,
        );
    }
    if lower.contains("cannot find function") {
        return report(
            DiagnosticKind::MissingFunction,
            "Rust function is missing",
            file_hint(&output),
            line_hint(&output),
            Some("Add the missing function to src/lib.rs if it is a known safe calculator operation.".to_string()),
            0.8,
            excerpt,
        );
    }
    if lower.contains("mismatched types") {
        return report(
            DiagnosticKind::TypeMismatch,
            "Rust type mismatch detected",
            file_hint(&output),
            line_hint(&output),
            Some("Use numeric i32 return types for simple calculator operations.".to_string()),
            0.75,
            excerpt,
        );
    }
    if lower.contains("panicked")
        || lower.contains("test result: failed")
        || lower.contains("failed")
    {
        return report(
            DiagnosticKind::TestFailure,
            "Rust test failure detected",
            file_hint(&output),
            line_hint(&output),
            Some("Check deterministic calculator operation behavior against tests.".to_string()),
            0.65,
            excerpt,
        );
    }
    if lower.contains("expected") || lower.contains("mismatched closing delimiter") {
        return report(
            DiagnosticKind::SyntaxError,
            "Rust syntax error detected",
            file_hint(&output),
            line_hint(&output),
            Some("Rewrite the affected Rust file with a minimal valid template.".to_string()),
            0.75,
            excerpt,
        );
    }
    if lower.contains("no such file") || lower.contains("could not read") {
        return report(
            DiagnosticKind::MissingFile,
            "A required file appears to be missing",
            file_hint(&output),
            line_hint(&output),
            Some("Create the missing file inside the sandbox project.".to_string()),
            0.7,
            excerpt,
        );
    }
    if lower.contains("file not found for module") || lower.contains("unresolved import") {
        return report(
            DiagnosticKind::MissingModule,
            "A Rust module or import is missing",
            file_hint(&output),
            line_hint(&output),
            Some("Create the module or remove the unresolved import.".to_string()),
            0.7,
            excerpt,
        );
    }
    if lower.contains("divide by zero") || lower.contains("division by zero") {
        return report(
            DiagnosticKind::DivideByZeroRisk,
            "Potential divide by zero risk detected",
            file_hint(&output),
            line_hint(&output),
            Some("Guard division with an Option result.".to_string()),
            0.65,
            excerpt,
        );
    }
    if lower.contains("failed to select a version") || lower.contains("unresolved package") {
        return report(
            DiagnosticKind::DependencyError,
            "Cargo dependency error detected",
            file_hint(&output),
            line_hint(&output),
            Some("Use only available local dependencies or remove the dependency.".to_string()),
            0.7,
            excerpt,
        );
    }
    report(
        DiagnosticKind::UnknownError,
        "Unknown command failure",
        file_hint(&output),
        line_hint(&output),
        None,
        0.3,
        excerpt,
    )
}

fn report(
    kind: DiagnosticKind,
    summary: impl Into<String>,
    file_hint: Option<String>,
    line_hint: Option<u32>,
    suggested_fix: Option<String>,
    confidence: f32,
    raw_output_excerpt: String,
) -> DiagnosticReport {
    DiagnosticReport {
        kind,
        summary: summary.into(),
        file_hint,
        line_hint,
        suggested_fix,
        confidence,
        raw_output_excerpt,
    }
}

fn file_hint(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.split("-->").nth(1))
        .and_then(|hint| hint.trim().split(':').next())
        .map(str::trim)
        .filter(|hint| !hint.is_empty())
        .map(ToOwned::to_owned)
}

fn line_hint(output: &str) -> Option<u32> {
    output
        .lines()
        .find_map(|line| line.split("-->").nth(1))
        .and_then(|hint| hint.trim().split(':').nth(1))
        .and_then(|line| line.parse::<u32>().ok())
}
