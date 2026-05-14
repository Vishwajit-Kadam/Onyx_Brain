use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueResponse {
    pub subject: String,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub risks: Vec<String>,
    pub missing_parts: Vec<String>,
    pub improvement_plan: Vec<String>,
    pub score: f32,
}

pub fn critique_response(subject: &str) -> CritiqueResponse {
    CritiqueResponse {
        subject: subject.to_string(),
        strengths: vec![
            "Clear bounded-autonomy framing.".to_string(),
            "Recoverability systems make work inspectable.".to_string(),
            "Disk-backed design supports RAM-minimal operation.".to_string(),
        ],
        weaknesses: vec![
            "Rule-based responses can feel repetitive.".to_string(),
            "No external verification by default.".to_string(),
        ],
        risks: vec![
            "Users may overestimate deterministic templates.".to_string(),
            "Generated claims need review before publication.".to_string(),
        ],
        missing_parts: vec![
            "Clear examples for non-code workflows.".to_string(),
            "More conversation benchmark coverage.".to_string(),
        ],
        improvement_plan: vec![
            "Add mode-specific tests.".to_string(),
            "Keep safety filters strict.".to_string(),
            "Document limitations beside examples.".to_string(),
        ],
        score: 0.82,
    }
}

pub fn render_critique(response: &CritiqueResponse) -> String {
    format!(
        "# Critic Mode\n\n## Subject\n{}\n\n## Strengths\n{}\n\n## Weaknesses\n{}\n\n## Risks\n{}\n\n## Missing Parts\n{}\n\n## Improvement Plan\n{}\n\nScore: {:.2}\n",
        response.subject,
        bullets(&response.strengths),
        bullets(&response.weaknesses),
        bullets(&response.risks),
        bullets(&response.missing_parts),
        bullets(&response.improvement_plan),
        response.score
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
