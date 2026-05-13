pub fn pitch_deck(topic: &str, slide_count: usize) -> String {
    let count = slide_count.clamp(6, 20);
    let mut out = format!("# Pitch Deck: {topic}\n\n");
    let titles = [
        "Problem",
        "Audience",
        "Solution",
        "Workflow",
        "Safety Model",
        "Artifacts",
        "Reliability",
        "Roadmap",
        "Demo",
        "Next Steps",
    ];
    for index in 0..count {
        let title = titles.get(index).unwrap_or(&"Supporting Detail");
        out.push_str(&format!(
            "## Slide {}: {}\n- Practical bounded autonomy\n- Local deterministic artifacts\n- Safety boundaries remain explicit\n\nSpeaker Notes:\nExplain this point without making AGI or consciousness claims.\n\nVisual Suggestion:\nUse a clean workflow diagram or artifact screenshot placeholder.\n\n",
            index + 1,
            title
        ));
    }
    out
}

pub fn landing_page_copy(topic: &str) -> String {
    format!("# Landing Page Copy: {topic}\n\n## Hero\nA disk-backed, bounded autonomous worker runtime for recoverable local workflows.\n\n## Value Props\n- Create multi-artifact markdown packages\n- Validate and repair outputs inside hard limits\n- Journal actions and preserve recovery paths\n\n## Trust Notes\nExperimental. Not AGI. Not conscious. No LLM or network access by default.\n")
}

pub fn demo_script(topic: &str) -> String {
    format!("# Demo Script: {topic}\n\n1. Initialize the runtime.\n2. Run a full-bounded autonomous prompt.\n3. Inspect artifact packs and reports.\n4. Export the package.\n5. Run doctor and regression-check.\n\n## Narration\nEmphasize bounded autonomy and transparent reports.\n")
}

pub fn social_posts(topic: &str) -> String {
    format!("# Social Post Drafts: {topic}\n\n1. Onyx Brain v0.0.2 creates bounded local artifact packs with validation, repair, reports, and exports.\n2. Built in Rust, disk-backed, sandbox-first, and explicit about limitations: no AGI, no consciousness, no network by default.\n3. Try a launch kit, learning pack, or documentation pack and inspect the generated reports.\n")
}

pub fn email_announcement(topic: &str) -> String {
    format!("# Email Announcement: {topic}\n\nSubject: Onyx Brain v0.0.2 bounded autonomy update\n\nHi,\n\nThis release expands Onyx Brain with multi-artifact packages, task graphs, review/repair cycles, export folders, and reliability reports.\n\nIt remains experimental, deterministic by default, and bounded by sandbox and command allowlist rules.\n")
}
