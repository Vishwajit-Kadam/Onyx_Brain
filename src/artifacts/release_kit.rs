use crate::artifacts::ArtifactKind;

pub fn release_kit_files() -> Vec<(&'static str, ArtifactKind)> {
    vec![
        ("release_notes.md", ArtifactKind::MarkdownDocument),
        ("changelog_entry.md", ArtifactKind::MarkdownDocument),
        ("github_release_draft.md", ArtifactKind::MarkdownDocument),
        ("demo_script.md", ArtifactKind::MarkdownDocument),
        ("technical_overview.md", ArtifactKind::MarkdownDocument),
        ("faq.md", ArtifactKind::Faq),
        ("social_posts.md", ArtifactKind::MarkdownDocument),
        ("risk_notes.md", ArtifactKind::RiskRegister),
        ("launch_checklist.md", ArtifactKind::Checklist),
        ("final_report.md", ArtifactKind::FinalReport),
    ]
}

pub fn generate_release_kit_file(file_name: &str, topic: &str) -> String {
    match file_name {
        "executive_summary.md" => crate::artifacts::executive_summary(topic),
        "pitch_deck.md" => crate::artifacts::pitch_deck(topic, 10),
        "landing_page_copy.md" => crate::artifacts::landing_page_copy(topic),
        "email_announcement.md" => crate::artifacts::email_announcement(topic),
        "architecture_brief.md" => crate::artifacts::architecture_brief(topic),
        "roadmap.md" => crate::artifacts::roadmap(topic),
        "metrics_plan.md" => crate::artifacts::metrics_plan(topic),
        "risk_register.md" => crate::artifacts::risk_register(topic),
        "release_notes.md" => format!("# Release Notes: {topic}\n\n## Highlights\n- Bounded autonomous worker runtime improvements.\n- Multi-artifact packs, validation, revision, and export support.\n\n## Safety Notes\nThis release does not add AGI, consciousness, network access by default, or unrestricted shell execution.\n"),
        "changelog_entry.md" => format!("# Changelog Entry: {topic}\n\n- Added bounded autonomous launch-kit generation.\n- Added artifact review, repair, report cards, and export metadata.\n- Preserved sandbox, allowlist, journal, snapshot, rollback, doctor, and regression-check behavior.\n"),
        "github_release_draft.md" => format!("# GitHub Release Draft: {topic}\n\n## Summary\nExperimental bounded autonomous worker runtime update.\n\n## Included\n- Release-kit artifacts\n- Validation and repair reports\n- Export package support\n\n## Limitations\nNo LLM by default. No network by default. Not AGI or conscious.\n"),
        "demo_script.md" => "# Demo Script\n\n1. Run `cargo run -- init`.\n2. Run `cargo run -- autonomize --level full-bounded \"Create a launch kit\"`.\n3. Inspect packs, review artifacts, export the package, then run doctor and regression-check.\n".to_string(),
        "technical_overview.md" => format!("# Technical Overview: {topic}\n\n## Architecture\nDisk-backed runtime with sparse active state, bounded autonomy, artifact packs, reliability tools, and local deterministic generators.\n\n## Boundaries\nSandboxed file writes, allowlisted commands, no network by default.\n"),
        "faq.md" => "# FAQ\n\n## Is this AGI?\nNo. It is bounded workflow software.\n\n## Is it conscious?\nNo.\n\n## Does it use an LLM by default?\nNo.\n\n## Does it access the network by default?\nNo.\n".to_string(),
        "social_posts.md" => format!("# Social Post Drafts: {topic}\n\n1. Onyx Brain v0.0.2 expands bounded autonomous artifact workflows with safety-first recovery tools.\n2. New release-kit generation creates markdown deliverables, validation reports, and export folders inside the sandbox.\n"),
        "risk_notes.md" => "# Risk Notes\n\n| Risk | Mitigation |\n| --- | --- |\n| Overstated claims | Explicit not-AGI/not-conscious language |\n| Missing deliverables | Completeness checks and revision cycle |\n| Unsafe paths | Sandbox validation |\n".to_string(),
        "launch_checklist.md" => "# Launch Checklist\n\n- [ ] Review release notes\n- [ ] Review changelog entry\n- [ ] Run doctor\n- [ ] Run regression-check\n- [ ] Inspect export package\n".to_string(),
        "final_report.md" => format!("# Final Report: {topic}\n\nGenerated launch-kit artifacts, validation metadata, assumptions, limitations, and export package references.\n"),
        _ => format!("# {topic}\n\nRelease-kit artifact.\n"),
    }
}
