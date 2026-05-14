use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateAnalysis {
    pub topic: String,
    pub side_a: DebateSide,
    pub side_b: DebateSide,
    pub key_arguments: Vec<Argument>,
    pub counterarguments: Vec<CounterArgument>,
    pub weak_points: Vec<String>,
    pub common_ground: Vec<String>,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateSide {
    pub name: String,
    pub position: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub claim: String,
    pub support: String,
    pub caveat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterArgument {
    pub target_claim: String,
    pub counterpoint: String,
    pub strength: f32,
}

pub fn debate_analysis(topic: &str) -> DebateAnalysis {
    DebateAnalysis {
        topic: topic.to_string(),
        side_a: DebateSide {
            name: "Open approach".to_string(),
            position: "Prefer openness when safety and maintenance boundaries are explicit.".to_string(),
            arguments: vec![
                "More reviewers can find flaws.".to_string(),
                "Open artifacts improve trust and learning.".to_string(),
                "Shared tooling can reduce duplicated work.".to_string(),
            ],
        },
        side_b: DebateSide {
            name: "Controlled approach".to_string(),
            position: "Prefer staged release when misuse, quality, or support risks are high.".to_string(),
            arguments: vec![
                "Incomplete systems can be misunderstood.".to_string(),
                "Security-sensitive details may need review.".to_string(),
                "Maintainers need capacity for support.".to_string(),
            ],
        },
        key_arguments: vec![
            Argument {
                claim: "Open source improves auditability.".to_string(),
                support: "Independent users can inspect code, docs, and tests.".to_string(),
                caveat: "Auditability only helps if reviewers actually engage.".to_string(),
            },
            Argument {
                claim: "Controlled release can lower operational risk.".to_string(),
                support: "Teams can fix known issues before broad exposure.".to_string(),
                caveat: "Too much control can reduce trust and feedback.".to_string(),
            },
        ],
        counterarguments: vec![CounterArgument {
            target_claim: "Open source is always safer.".to_string(),
            counterpoint: "Safety depends on design, documentation, defaults, and maintenance.".to_string(),
            strength: 0.85,
        }],
        weak_points: vec![
            "Both sides can overstate certainty without evidence.".to_string(),
            "Context matters: audience, maturity, and risk profile change the answer.".to_string(),
        ],
        common_ground: vec![
            "Use clear safety documentation.".to_string(),
            "Avoid unsupported capability claims.".to_string(),
            "Ship tests and recovery guidance.".to_string(),
        ],
        verdict: "A staged open approach is usually strongest: publish the safe core with honest limitations, then expand as tests and docs mature.".to_string(),
    }
}

pub fn render_debate(analysis: &DebateAnalysis) -> String {
    format!(
        "# Debate\n\n## Topic\n{}\n\n## Side A: {}\n{}\n{}\n\n## Side B: {}\n{}\n{}\n\n## Strongest Arguments\n{}\n\n## Counterarguments\n{}\n\n## Common Ground\n{}\n\n## Balanced Verdict\n{}\n\n## Questions To Explore Next\n- What risks are specific to this context?\n- What evidence would change the conclusion?\n",
        analysis.topic,
        analysis.side_a.name,
        analysis.side_a.position,
        bullets(&analysis.side_a.arguments),
        analysis.side_b.name,
        analysis.side_b.position,
        bullets(&analysis.side_b.arguments),
        analysis
            .key_arguments
            .iter()
            .map(|arg| format!("- {} Support: {} Caveat: {}", arg.claim, arg.support, arg.caveat))
            .collect::<Vec<_>>()
            .join("\n"),
        analysis
            .counterarguments
            .iter()
            .map(|arg| format!("- Against '{}': {} ({:.2})", arg.target_claim, arg.counterpoint, arg.strength))
            .collect::<Vec<_>>()
            .join("\n"),
        bullets(&analysis.common_ground),
        analysis.verdict
    )
}

fn bullets(rows: &[String]) -> String {
    rows.iter()
        .map(|row| format!("- {row}"))
        .collect::<Vec<_>>()
        .join("\n")
}
