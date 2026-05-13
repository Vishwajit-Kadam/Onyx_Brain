use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{DoneDefinition, GoalUnderstanding},
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkContract {
    pub session_id: String,
    pub user_goal: String,
    pub interpreted_goal: String,
    pub deliverables: Vec<String>,
    pub assumptions: Vec<String>,
    pub limitations: Vec<String>,
    pub safety_boundaries: Vec<String>,
    pub done_definition: DoneDefinition,
}

pub fn create_work_contract(
    session_id: &str,
    understanding: &GoalUnderstanding,
    done_definition: DoneDefinition,
) -> WorkContract {
    WorkContract {
        session_id: session_id.to_string(),
        user_goal: understanding.original_prompt.clone(),
        interpreted_goal: format!("{:?}", understanding.goal_type),
        deliverables: understanding
            .deliverables
            .iter()
            .map(|deliverable| {
                deliverable
                    .path_hint
                    .clone()
                    .unwrap_or_else(|| deliverable.title.clone())
            })
            .collect(),
        assumptions: vec![
            "Use deterministic markdown generation.".to_string(),
            "Use local context only; no network research by default.".to_string(),
        ],
        limitations: vec![
            "Binary PPTX export is not supported in this version.".to_string(),
            "Factual claims require external verification by the user.".to_string(),
        ],
        safety_boundaries: vec![
            "sandboxed writes only".to_string(),
            "allowlisted commands only".to_string(),
            "bounded task and retry counts".to_string(),
        ],
        done_definition,
    }
}

pub fn write_work_contract(store: &DiskStore, contract: &WorkContract) -> Result<(String, String)> {
    let root = store
        .paths
        .sandbox
        .join("workspaces")
        .join(&contract.session_id);
    fs::create_dir_all(&root)?;
    let json_path = root.join("work_contract.json");
    let md_path = root.join("work_contract.md");
    save_json(&json_path, contract)?;
    fs::write(&md_path, render_work_contract(contract))?;
    Ok((
        md_path.display().to_string(),
        json_path.display().to_string(),
    ))
}

fn render_work_contract(contract: &WorkContract) -> String {
    format!(
        "# Work Contract\n\nGoal: {}\n\nInterpreted goal: {}\n\n## Deliverables\n{}\n\n## Assumptions\n{}\n\n## Limitations\n{}\n\n## Safety Boundaries\n{}\n",
        contract.user_goal,
        contract.interpreted_goal,
        bullets(&contract.deliverables),
        bullets(&contract.assumptions),
        bullets(&contract.limitations),
        bullets(&contract.safety_boundaries)
    )
}

fn bullets(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}
