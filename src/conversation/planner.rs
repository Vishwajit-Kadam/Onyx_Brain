use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningResponse {
    pub goal: String,
    pub phases: Vec<String>,
    pub tasks: Vec<String>,
    pub dependencies: Vec<String>,
    pub risks: Vec<String>,
    pub estimated_difficulty: String,
    pub next_action: String,
}

pub fn planning_response(goal: &str) -> PlanningResponse {
    PlanningResponse {
        goal: goal.to_string(),
        phases: vec![
            "Clarify objective and safety boundaries".to_string(),
            "Design interfaces and disk-backed state".to_string(),
            "Implement deterministic behavior".to_string(),
            "Add tests and docs".to_string(),
            "Run regression and demo commands".to_string(),
        ],
        tasks: vec![
            "Define public commands and structs".to_string(),
            "Write storage/index helpers".to_string(),
            "Connect CLI output".to_string(),
            "Add focused tests".to_string(),
        ],
        dependencies: vec![
            "Version constants before regression tests".to_string(),
            "Storage helpers before doctor repair".to_string(),
        ],
        risks: vec![
            "Breaking older command output".to_string(),
            "Overclaiming capability".to_string(),
        ],
        estimated_difficulty: "Moderate".to_string(),
        next_action: "Write the smallest safe module boundary first.".to_string(),
    }
}

pub fn render_plan(response: &PlanningResponse) -> String {
    format!(
        "# Planner Mode\n\nGoal: {}\n\n## Phases\n{}\n\n## Tasks\n{}\n\n## Dependencies\n{}\n\n## Risks\n{}\n\nEstimated difficulty: {}\n\nNext action: {}\n",
        response.goal,
        bullets(&response.phases),
        bullets(&response.tasks),
        bullets(&response.dependencies),
        bullets(&response.risks),
        response.estimated_difficulty,
        response.next_action
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
