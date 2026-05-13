use crate::artifacts::ArtifactKind;

pub fn documentation_pack_files() -> Vec<(&'static str, ArtifactKind)> {
    vec![
        ("overview.md", ArtifactKind::MarkdownDocument),
        ("user_guide.md", ArtifactKind::UserGuide),
        ("command_reference.md", ArtifactKind::MarkdownDocument),
        (
            "architecture_summary.md",
            ArtifactKind::ArchitectureDocument,
        ),
        ("troubleshooting.md", ArtifactKind::MarkdownDocument),
        ("faq.md", ArtifactKind::Faq),
        ("final_report.md", ArtifactKind::FinalReport),
    ]
}

pub fn generate_documentation_file(file_name: &str, topic: &str) -> String {
    match file_name {
        "overview.md" => format!("# Overview: {topic}\n\nThis document summarizes the bounded autonomous worker runtime, safety model, and local artifact workflows.\n"),
        "user_guide.md" => "# User Guide\n\n## Start\nRun `cargo run -- init`.\n\n## Autonomous Work\nUse `autonomize` for bounded artifact workflows.\n\n## Inspect\nUse artifact, session, doctor, and regression commands.\n".to_string(),
        "command_reference.md" => "# Command Reference\n\n- `init`\n- `autonomize`\n- `artifact-packs`\n- `review-artifacts`\n- `repair-artifacts`\n- `export-package`\n- `doctor`\n- `regression-check`\n".to_string(),
        "architecture_summary.md" => "# Architecture Summary\n\nDisk-backed state, sparse active runtime, deterministic planning, artifact packs, validation, repair, journaling, snapshots, rollback, and doctor checks.\n".to_string(),
        "troubleshooting.md" => "# Troubleshooting\n\n## Missing Artifact\nRun `review-artifacts latest`, then `repair-artifacts latest`.\n\n## State Warning\nRun `doctor` and review recommendations.\n".to_string(),
        "faq.md" => "# FAQ\n\n## Is this conscious?\nNo.\n\n## Is this AGI?\nNo.\n\n## Does it use the network by default?\nNo.\n".to_string(),
        "final_report.md" => format!("# Final Report: {topic}\n\nDocumentation pack generated and ready for local review.\n"),
        _ => format!("# {topic}\n\nDocumentation artifact.\n"),
    }
}
