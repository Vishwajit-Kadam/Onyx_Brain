use crate::core::{Task, TaskType};

pub struct Planner;

impl Planner {
    pub fn plan(task: &Task) -> Vec<String> {
        if matches!(task.task_type, TaskType::Code | TaskType::FileOperation)
            && task.input.to_lowercase().contains("hello")
        {
            vec![
                "Create project directory".to_string(),
                "Write Cargo.toml".to_string(),
                "Write src/main.rs".to_string(),
                "Write src/lib.rs".to_string(),
                "Run cargo check if available".to_string(),
                "Save summary".to_string(),
            ]
        } else {
            vec!["Classify task".to_string(), "Prepare response".to_string()]
        }
    }
}
