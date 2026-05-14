use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchOutlineResponse {
    pub topic: String,
    pub research_questions: Vec<String>,
    pub source_types_needed: Vec<String>,
    pub search_terms: Vec<String>,
    pub outline: Vec<String>,
    pub verification_notes: Vec<String>,
    pub citation_placeholders: Vec<String>,
}

pub fn research_outline_response(topic: &str) -> ResearchOutlineResponse {
    ResearchOutlineResponse {
        topic: topic.to_string(),
        research_questions: vec![
            format!("What engineering problems does {topic} address?"),
            "Which claims require external evidence?".to_string(),
            "What limitations should be stated clearly?".to_string(),
        ],
        source_types_needed: vec![
            "official documentation".to_string(),
            "peer-reviewed papers".to_string(),
            "project source code and tests".to_string(),
        ],
        search_terms: vec![
            topic.to_string(),
            "sparse activation architecture".to_string(),
            "bounded autonomous agents safety".to_string(),
        ],
        outline: vec![
            "Background".to_string(),
            "Core concepts".to_string(),
            "Implementation patterns".to_string(),
            "Safety and limitations".to_string(),
            "Open questions".to_string(),
        ],
        verification_notes: vec![
            "No web search was performed by default.".to_string(),
            "Replace placeholders with verified sources before publishing.".to_string(),
        ],
        citation_placeholders: vec!["[citation needed]".to_string()],
    }
}

pub fn render_research_outline(response: &ResearchOutlineResponse) -> String {
    format!(
        "# Research Outline Mode\n\n## Topic\n{}\n\n## Research Questions\n{}\n\n## Source Types Needed\n{}\n\n## Search Terms\n{}\n\n## Outline\n{}\n\n## Verification Notes\n{}\n\n## Citation Placeholders\n{}\n",
        response.topic,
        bullets(&response.research_questions),
        bullets(&response.source_types_needed),
        bullets(&response.search_terms),
        bullets(&response.outline),
        bullets(&response.verification_notes),
        bullets(&response.citation_placeholders)
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
