use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureResponse {
    pub system_goal: String,
    pub modules: Vec<String>,
    pub data_flow: Vec<String>,
    pub storage_design: Vec<String>,
    pub safety_boundaries: Vec<String>,
    pub tradeoffs: Vec<String>,
    pub implementation_steps: Vec<String>,
}

pub fn architecture_response(goal: &str) -> ArchitectureResponse {
    ArchitectureResponse {
        system_goal: goal.to_string(),
        modules: vec![
            "Adapter interface module".to_string(),
            "Policy and permissions module".to_string(),
            "Transcript and audit module".to_string(),
            "Fallback deterministic mode".to_string(),
        ],
        data_flow: vec![
            "User input -> policy check -> deterministic planner -> optional adapter boundary"
                .to_string(),
            "Response -> safety review -> transcript save".to_string(),
        ],
        storage_design: vec![
            "Configuration under data/config".to_string(),
            "Session state under data/conversations".to_string(),
            "No secrets stored by default".to_string(),
        ],
        safety_boundaries: vec![
            "disabled by default".to_string(),
            "no network unless explicitly configured later".to_string(),
            "no unrestricted shell".to_string(),
        ],
        tradeoffs: vec![
            "Optional adapters improve flexibility but expand the review surface.".to_string(),
            "Strict policy reduces surprise but may feel less capable.".to_string(),
        ],
        implementation_steps: vec![
            "Define trait boundary".to_string(),
            "Add config defaults".to_string(),
            "Add tests for disabled-by-default behavior".to_string(),
        ],
    }
}

pub fn render_architecture(response: &ArchitectureResponse) -> String {
    format!(
        "# Architect Mode\n\nSystem goal: {}\n\n## Modules\n{}\n\n## Data Flow\n{}\n\n## Storage Design\n{}\n\n## Safety Boundaries\n{}\n\n## Tradeoffs\n{}\n\n## Implementation Steps\n{}\n",
        response.system_goal,
        bullets(&response.modules),
        bullets(&response.data_flow),
        bullets(&response.storage_design),
        bullets(&response.safety_boundaries),
        bullets(&response.tradeoffs),
        bullets(&response.implementation_steps)
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
