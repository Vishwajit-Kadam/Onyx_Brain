use crate::core::TaskType;

pub struct Classifier;

impl Classifier {
    pub fn classify(input: &str) -> TaskType {
        let lower = input.to_lowercase();
        if contains_any(
            &lower,
            &["code", "rust", "compile", "cargo", "error", "project"],
        ) {
            TaskType::Code
        } else if contains_any(&lower, &["create file", "folder", "write", "directory"]) {
            TaskType::FileOperation
        } else if contains_any(&lower, &["plan", "steps", "roadmap"]) {
            TaskType::Planning
        } else if contains_any(&lower, &["why", "reason", "analyze"]) {
            TaskType::Reasoning
        } else if contains_any(&lower, &["remember", "memory", "recall"]) {
            TaskType::MemoryQuery
        } else if lower.trim().is_empty() {
            TaskType::Unknown
        } else {
            TaskType::Chat
        }
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
