use serde::{Deserialize, Serialize};

use crate::tools::sanitize_project_name;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentKind {
    CreateProject,
    ModifyProject,
    InspectProject,
    ResumeProject,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedGoal {
    pub intent: IntentKind,
    pub project_name: Option<String>,
    pub requested_features: Vec<String>,
    pub wants_tests: bool,
    pub wants_readme: bool,
    pub original_prompt: String,
}

pub fn parse_goal(prompt: &str) -> ParsedGoal {
    let lower = prompt.to_lowercase();
    let intent = if lower.contains("resume") {
        IntentKind::ResumeProject
    } else if lower.contains("inspect") {
        IntentKind::InspectProject
    } else if lower.contains("modify")
        || lower.contains("update ")
        || lower.contains("add feature")
        || lower.starts_with("add ")
    {
        IntentKind::ModifyProject
    } else if lower.contains("create") || lower.contains("make ") {
        IntentKind::CreateProject
    } else {
        IntentKind::Unknown
    };
    ParsedGoal {
        intent,
        project_name: extract_project_name(prompt),
        requested_features: extract_features(prompt),
        wants_tests: lower.contains("test"),
        wants_readme: lower.contains("readme"),
        original_prompt: prompt.to_string(),
    }
}

pub fn extract_project_name(prompt: &str) -> Option<String> {
    let words = prompt
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-'))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    for marker in ["called", "named"] {
        if let Some(index) = words
            .iter()
            .position(|word| word.eq_ignore_ascii_case(marker))
        {
            return words.get(index + 1).map(|name| sanitize_project_name(name));
        }
    }

    if let Some(index) = words
        .iter()
        .position(|word| word.eq_ignore_ascii_case("the"))
    {
        if words
            .get(index + 2)
            .is_some_and(|word| word.eq_ignore_ascii_case("project"))
        {
            return words.get(index + 1).map(|name| sanitize_project_name(name));
        }
    }

    for marker in ["update", "in", "for"] {
        if let Some(index) = words
            .iter()
            .position(|word| word.eq_ignore_ascii_case(marker))
        {
            if marker == "update" {
                return words.get(index + 1).map(|name| sanitize_project_name(name));
            }
            if words
                .get(index + 1)
                .is_some_and(|word| word.eq_ignore_ascii_case("the"))
                && words
                    .get(index + 3)
                    .is_some_and(|word| word.eq_ignore_ascii_case("project"))
            {
                return words.get(index + 2).map(|name| sanitize_project_name(name));
            }
            if words
                .get(index + 1)
                .is_some_and(|word| word.eq_ignore_ascii_case("project"))
            {
                return words.get(index + 2).map(|name| sanitize_project_name(name));
            }
        }
    }
    None
}

pub fn extract_features(prompt: &str) -> Vec<String> {
    let lower = prompt.to_lowercase();
    let mut features = Vec::new();
    if lower.contains("add and")
        || lower.contains("add function")
        || lower.contains("add,")
        || lower.contains(" add/")
    {
        features.push("add".to_string());
    }
    for (needle, label) in [
        ("subtract", "subtract"),
        ("multiply", "multiply"),
        ("divide", "divide"),
        ("modulo", "modulo"),
        ("power", "power"),
        ("test", "tests"),
        ("readme", "README"),
        ("cli", "CLI"),
        ("module", "module"),
        ("function", "function"),
    ] {
        if lower.contains(needle) && !features.iter().any(|feature| feature == label) {
            features.push(label.to_string());
        }
    }
    features
}
