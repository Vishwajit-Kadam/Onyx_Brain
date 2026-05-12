use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerModeOutput {
    pub session_id: String,
    pub goal: String,
    pub phases_completed: u64,
    pub tasks_completed: usize,
    pub failures: Vec<String>,
    pub recovery_actions: Vec<String>,
    pub final_report: String,
}

pub fn extract_worker_project_name(prompt: &str) -> String {
    let words = prompt.split_whitespace().collect::<Vec<_>>();
    for marker in ["called", "named"] {
        if let Some(index) = words
            .iter()
            .position(|word| word.eq_ignore_ascii_case(marker))
        {
            if let Some(name) = words.get(index + 1) {
                return name
                    .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                    .to_string();
            }
        }
    }
    "worker_calc".to_string()
}
