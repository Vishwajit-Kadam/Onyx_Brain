use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TeachingLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingResponse {
    pub topic: String,
    pub level: TeachingLevel,
    pub explanation: Vec<String>,
    pub examples: Vec<String>,
    pub analogy: Option<String>,
    pub mini_exercise: Option<String>,
    pub recap: Vec<String>,
}

pub fn teaching_level(input: &str) -> TeachingLevel {
    let lower = input.to_lowercase();
    if lower.contains("beginner") || lower.contains("simple") || lower.contains("new to") {
        TeachingLevel::Beginner
    } else if lower.contains("advanced")
        || lower.contains("technical")
        || lower.contains("architecture")
    {
        TeachingLevel::Advanced
    } else {
        TeachingLevel::Intermediate
    }
}

pub fn teaching_response(topic: &str, input: &str) -> TeachingResponse {
    TeachingResponse {
        topic: topic.to_string(),
        level: teaching_level(input),
        explanation: vec![
            format!("{topic} is easiest to understand as using only the parts of a system needed for the current task."),
            "In Onyx Brain terms, the runtime keeps most state on disk and activates a small useful working set.".to_string(),
            "This is an engineering analogy, not a biological simulation or real understanding.".to_string(),
        ],
        examples: vec![
            "A project command loads the registry, task queue, relevant memories, and tools; it does not load every memory.".to_string(),
            "A conversation turn updates one active session instead of scanning every transcript.".to_string(),
        ],
        analogy: Some("Like opening only the notebooks needed for today instead of dumping the whole library onto your desk.".to_string()),
        mini_exercise: Some("Name one piece of state that should stay on disk until it becomes relevant.".to_string()),
        recap: vec![
            "Sparse means selective.".to_string(),
            "Disk-backed means memory-light.".to_string(),
            "Bounded means safety limits stay active.".to_string(),
        ],
    }
}

pub fn render_teaching(response: &TeachingResponse) -> String {
    format!(
        "# Teaching Mode\n\n## Topic\n{}\n\n## Level\n{:?}\n\n## Explanation\n{}\n\n## Examples\n{}\n\n## Analogy\n{}\n\n## Mini Exercise\n{}\n\n## Recap\n{}\n",
        response.topic,
        response.level,
        bullets(&response.explanation),
        bullets(&response.examples),
        response.analogy.clone().unwrap_or_default(),
        response.mini_exercise.clone().unwrap_or_default(),
        bullets(&response.recap)
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
