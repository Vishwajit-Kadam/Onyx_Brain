use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    artifacts::workspace_artifacts_dir,
    storage::{save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfQuestion {
    pub question: String,
    pub inferred_answer: String,
    pub confidence: f32,
    pub assumption_created: bool,
}

pub fn generate_self_questions(prompt: &str) -> Vec<SelfQuestion> {
    let lower = prompt.to_lowercase();
    vec![
        SelfQuestion {
            question: "Who is the audience?".to_string(),
            inferred_answer: if lower.contains("student") {
                "Students, because the prompt mentions students.".to_string()
            } else {
                "General technical readers, because no audience was specified.".to_string()
            },
            confidence: if lower.contains("student") { 0.9 } else { 0.65 },
            assumption_created: true,
        },
        SelfQuestion {
            question: "Should this be binary PPTX or markdown?".to_string(),
            inferred_answer: "Export-ready markdown, because v0.0.2 does not create binary PPTX files.".to_string(),
            confidence: 0.95,
            assumption_created: true,
        },
        SelfQuestion {
            question: "Should external facts be treated as verified?".to_string(),
            inferred_answer: "No. Network verification is disabled by default, so citation placeholders and verification notes are used.".to_string(),
            confidence: 1.0,
            assumption_created: true,
        },
    ]
}

pub fn write_self_questions(
    store: &DiskStore,
    session_id: &str,
    questions: &[SelfQuestion],
) -> Result<(String, String)> {
    let dir = workspace_artifacts_dir(store, session_id);
    fs::create_dir_all(&dir)?;
    let json = dir.join("self_questions.json");
    let md = dir.join("self_questions.md");
    save_json(&json, &questions.to_vec())?;
    let mut content = "# Self Questions\n\n".to_string();
    for question in questions {
        content.push_str(&format!(
            "## {}\n{}\n\nConfidence: {:.2}\n\n",
            question.question, question.inferred_answer, question.confidence
        ));
    }
    fs::write(&md, content)?;
    Ok((md.display().to_string(), json.display().to_string()))
}
