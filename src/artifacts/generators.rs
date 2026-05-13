use crate::artifacts::{
    render_design_guide, render_presentation_markdown, render_speaker_notes, ArtifactKind,
    PresentationArtifact,
};

pub fn file_name_for_kind(kind: &ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::PresentationMarkdown => "presentation.md",
        ArtifactKind::SpeakerNotes => "speaker_notes.md",
        ArtifactKind::DesignGuide => "design_guide.md",
        ArtifactKind::StudyGuide => "study_guide.md",
        ArtifactKind::Quiz => "quiz.md",
        ArtifactKind::Glossary => "glossary.md",
        ArtifactKind::Checklist => "checklist.md",
        ArtifactKind::Roadmap => "roadmap.md",
        ArtifactKind::RiskRegister => "risk_register.md",
        ArtifactKind::ArchitectureDocument => "architecture.md",
        ArtifactKind::BudgetTable => "budget_table.md",
        ArtifactKind::Faq => "faq.md",
        ArtifactKind::UserGuide => "user_guide.md",
        ArtifactKind::PitchDeck => "pitch_deck.md",
        ArtifactKind::LandingPageCopy => "landing_page_copy.md",
        ArtifactKind::DemoScript => "demo_script.md",
        ArtifactKind::SocialPostSet => "social_posts.md",
        ArtifactKind::EmailAnnouncement => "email_announcement.md",
        ArtifactKind::ExecutiveSummary => "executive_summary.md",
        ArtifactKind::ProductSpec => "product_spec.md",
        ArtifactKind::TechnicalOverview => "technical_overview.md",
        ArtifactKind::ArchitectureBrief => "architecture_brief.md",
        ArtifactKind::CompetitiveAnalysis => "competitive_analysis.md",
        ArtifactKind::SwotAnalysis => "swot_analysis.md",
        ArtifactKind::MetricsPlan => "metrics_plan.md",
        ArtifactKind::ReleaseNotes => "release_notes.md",
        ArtifactKind::GitHubReleaseDraft => "github_release_draft.md",
        ArtifactKind::ContributorGuide => "contributor_guide.md",
        ArtifactKind::SecurityNotes => "security_notes.md",
        ArtifactKind::LaunchChecklist => "launch_checklist.md",
        ArtifactKind::FinalReport => "final_report.md",
        ArtifactKind::Manifest => "artifact_manifest.json",
        _ => "artifact.md",
    }
}

pub fn generate_artifact(
    kind: &ArtifactKind,
    topic: &str,
    presentation: Option<&PresentationArtifact>,
) -> String {
    match kind {
        ArtifactKind::PresentationMarkdown => presentation
            .map(render_presentation_markdown)
            .unwrap_or_else(|| format!("# Presentation: {topic}\n\n## Slide 1: Overview\n- Key idea\n\nSpeaker Notes:\nIntroduce {topic}.\n")),
        ArtifactKind::SpeakerNotes => presentation
            .map(render_speaker_notes)
            .unwrap_or_else(|| format!("# Speaker Notes\n\nExplain {topic} in clear, bounded terms.\n")),
        ArtifactKind::DesignGuide => presentation
            .map(render_design_guide)
            .unwrap_or_else(|| format!("# Design Guide\n\nUse clean diagrams and concise sections for {topic}.\n")),
        ArtifactKind::StudyGuide => study_guide(topic),
        ArtifactKind::Quiz => quiz(topic),
        ArtifactKind::Glossary => glossary(topic),
        ArtifactKind::Checklist => checklist(topic),
        ArtifactKind::Roadmap => roadmap(topic),
        ArtifactKind::RiskRegister => risk_register(topic),
        ArtifactKind::ArchitectureDocument => architecture(topic),
        ArtifactKind::BudgetTable => budget_table(topic),
        ArtifactKind::Faq => faq(topic),
        ArtifactKind::UserGuide => user_guide(topic),
        ArtifactKind::ExecutiveSummary => crate::artifacts::executive_summary(topic),
        ArtifactKind::ProductSpec => crate::artifacts::product_spec(topic),
        ArtifactKind::TechnicalOverview => crate::artifacts::technical_overview(topic),
        ArtifactKind::ArchitectureBrief => crate::artifacts::architecture_brief(topic),
        ArtifactKind::MetricsPlan => crate::artifacts::metrics_plan(topic),
        ArtifactKind::LaunchChecklist => crate::artifacts::launch_checklist(topic),
        ArtifactKind::DemoScript => crate::artifacts::demo_script(topic),
        ArtifactKind::LandingPageCopy => crate::artifacts::landing_page_copy(topic),
        ArtifactKind::SocialPostSet => crate::artifacts::social_posts(topic),
        ArtifactKind::EmailAnnouncement => crate::artifacts::email_announcement(topic),
        ArtifactKind::PitchDeck => crate::artifacts::pitch_deck(topic, 10),
        ArtifactKind::CompetitiveAnalysis => crate::artifacts::competitive_analysis(topic),
        ArtifactKind::SwotAnalysis => crate::artifacts::swot_analysis(topic),
        ArtifactKind::ContributorGuide => crate::artifacts::contributor_guide(topic),
        ArtifactKind::SecurityNotes => crate::artifacts::security_notes(topic),
        ArtifactKind::ReleaseNotes => crate::artifacts::generate_release_kit_file("release_notes.md", topic),
        ArtifactKind::GitHubReleaseDraft => crate::artifacts::generate_release_kit_file("github_release_draft.md", topic),
        ArtifactKind::FinalReport => final_report(topic, &[]),
        _ => format!("# {topic}\n\nGenerated markdown artifact.\n"),
    }
}

pub fn study_guide(topic: &str) -> String {
    format!("# Study Guide: {topic}\n\n## Overview\nA practical learning guide for {topic}.\n\n## Key Concepts\n- Sparse activation\n- Memory hierarchy\n- Bounded autonomy\n- Validation and recovery\n\n## Explanations\nBrain-inspired systems use engineering analogies from cognition without claiming consciousness or AGI.\n\n## Summary\nFocus on safe, deterministic workflows and clear limitations.\n\n## Review Questions\n1. What does sparse activation mean?\n2. Why is validation important?\n")
}

pub fn quiz(topic: &str) -> String {
    format!("# Quiz: {topic}\n\n## Multiple Choice\n1. What does bounded autonomy mean?\n   - A. Unrestricted action\n   - B. Autonomous action inside hard limits\n   - C. Conscious decision-making\n   - D. Network access by default\n\n2. Which output is supported in v0.0.2?\n   - A. Export-ready markdown\n   - B. Binary PPTX by default\n   - C. Unrestricted shell scripts\n   - D. Hidden background jobs\n\n## Short Answer\n1. Explain why Onyx Brain is not AGI.\n2. Name two safety limits.\n\n## Answer Key\n1. B\n2. A\nShort answer: It is deterministic, bounded, and has no consciousness or default LLM. Safety limits include sandboxed writes and allowlisted commands.\n")
}

pub fn glossary(topic: &str) -> String {
    format!("# Glossary: {topic}\n\n- Sparse activation: activating only the needed working set.\n- Artifact pack: a related group of generated files and reports.\n- Bounded autonomy: autonomous execution within hard safety limits.\n- Validation: checking outputs against requirements.\n- Revision cycle: bounded repairs for detected issues.\n- Manifest: structured metadata listing generated artifacts.\n")
}

pub fn checklist(topic: &str) -> String {
    format!("# Checklist: {topic}\n\n- [ ] Review all generated artifacts\n- [ ] Confirm assumptions\n- [ ] Check limitations\n- [ ] Validate manifest paths\n- [ ] Run doctor and regression-check\n")
}

pub fn roadmap(topic: &str) -> String {
    format!("# Roadmap: {topic}\n\n## Phases\n1. Understand goals\n2. Create artifacts\n3. Validate quality\n4. Revise safely\n\n## Milestones\n- Draft pack\n- Quality review\n- Export package\n\n## Risks\n- Missing sections\n- Overstated claims\n\n## Next Steps\nReview outputs and iterate within safety limits.\n")
}

pub fn risk_register(topic: &str) -> String {
    format!("# Risk Register: {topic}\n\n| Risk | Severity | Likelihood | Mitigation |\n| --- | --- | --- | --- |\n| Missing artifact | Medium | Medium | Validate manifest |\n| Unsafe claim | High | Low | Safety review |\n| Weak quiz | Medium | Medium | Require answer key |\n")
}

pub fn architecture(topic: &str) -> String {
    format!("# Architecture Summary: {topic}\n\n## Components\n- Goal understanding\n- Planner\n- Artifact generators\n- Validator\n- Revision cycle\n\n## Data Flow\nPrompt -> plan -> artifacts -> review -> report.\n")
}

pub fn budget_table(topic: &str) -> String {
    format!("# Budget Table: {topic}\n\n| Item | Estimate | Notes |\n| --- | ---: | --- |\n| Planning | 1 unit | Deterministic |\n| Artifact generation | 3 units | Markdown only |\n| Review | 1 unit | Local validation |\n")
}

pub fn faq(topic: &str) -> String {
    format!("# FAQ: {topic}\n\n## Is this AGI?\nNo. It is bounded deterministic workflow software.\n\n## Does it use an LLM by default?\nNo.\n\n## Does it access the network by default?\nNo.\n\n## What files are generated?\nMarkdown artifacts, manifests, and reports inside the sandbox.\n")
}

pub fn user_guide(topic: &str) -> String {
    format!("# User Guide: {topic}\n\n## Start\nRun the bounded command and inspect artifacts.\n\n## Review\nUse artifact inspection, quality review, doctor, and regression-check.\n\n## Export\nUse export-package latest to gather generated files.\n")
}

pub fn final_report(topic: &str, artifacts: &[String]) -> String {
    format!("# Final Report: {topic}\n\n## Artifacts\n{}\n\n## Validation\nGenerated artifacts were checked for required sections and safety framing.\n\n## Safety\nBounded autonomy, sandboxed writes, no network by default, no unrestricted shell.\n", artifacts.iter().map(|artifact| format!("- {artifact}")).collect::<Vec<_>>().join("\n"))
}
