use crate::{agency::QueuedTask, core::TaskType};

pub fn decompose_goal(goal_id: &str, prompt: &str) -> Vec<QueuedTask> {
    let lower = prompt.to_lowercase();
    let mut tasks = vec![
        QueuedTask::new(goal_id, "Understand goal", prompt, TaskType::Planning),
        QueuedTask::new(
            goal_id,
            "Create project directory",
            "Create the sandbox project directory.",
            TaskType::FileOperation,
        ),
        QueuedTask::new(
            goal_id,
            "Write Cargo.toml",
            "Create a minimal Rust Cargo manifest.",
            TaskType::FileOperation,
        ),
        QueuedTask::new(
            goal_id,
            "Write src/main.rs",
            "Create the Rust CLI entrypoint.",
            TaskType::Code,
        ),
        QueuedTask::new(
            goal_id,
            "Write src/lib.rs",
            "Create reusable Rust library functions.",
            TaskType::Code,
        ),
    ];
    if lower.contains("test") {
        tasks.push(QueuedTask::new(
            goal_id,
            "Write tests",
            "Create integration tests if requested.",
            TaskType::Code,
        ));
    }
    if lower.contains("readme") {
        tasks.push(QueuedTask::new(
            goal_id,
            "Write README",
            "Create a project README if requested.",
            TaskType::FileOperation,
        ));
    }
    tasks.extend([
        QueuedTask::new(
            goal_id,
            "Run cargo check",
            "Run safe cargo check.",
            TaskType::ToolUse,
        ),
        QueuedTask::new(
            goal_id,
            "Run cargo test",
            "Run safe cargo test.",
            TaskType::ToolUse,
        ),
        QueuedTask::new(
            goal_id,
            "Inspect result",
            "Inspect files and command outcomes.",
            TaskType::Reasoning,
        ),
        QueuedTask::new(
            goal_id,
            "Create final report",
            "Create the final project report.",
            TaskType::FileOperation,
        ),
    ]);
    for index in 1..tasks.len() {
        tasks[index].dependencies = vec![tasks[index - 1].id.clone()];
    }
    tasks
}
