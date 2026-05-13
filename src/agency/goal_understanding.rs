use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalType {
    CodeProject,
    Document,
    Presentation,
    ResearchSummary,
    FileOrganization,
    Refactor,
    Debugging,
    Benchmark,
    Maintenance,
    LaunchKit,
    StartupPack,
    MarketingPack,
    LearningPack,
    DocumentationPack,
    TechnicalReport,
    ProductSpec,
    PitchDeck,
    StrategyPlan,
    ResearchPack,
    CodePackage,
    MixedArtifactProject,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliverableKind {
    MarkdownDocument,
    PresentationMarkdown,
    SlideOutline,
    CodeProject,
    SourceFile,
    TestFile,
    Report,
    Checklist,
    DataFile,
    StudyGuide,
    Quiz,
    Glossary,
    Roadmap,
    RiskRegister,
    ArchitectureDocument,
    BudgetTable,
    LessonPlan,
    FAQ,
    ComparisonTable,
    TestPlan,
    UserGuide,
    PitchDeck,
    LandingPageCopy,
    DemoScript,
    SocialPostSet,
    EmailAnnouncement,
    ProductSpec,
    TechnicalOverview,
    ArchitectureBrief,
    CompetitiveAnalysis,
    SWOTAnalysis,
    MetricsPlan,
    ReleaseNotes,
    GitHubReleaseDraft,
    ContributorGuide,
    SecurityNotes,
    LaunchChecklist,
    ExecutiveSummary,
    AcceptanceCriteria,
    UserStories,
    SecurityNotesDocument,
    ArchitectureBriefDocument,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    pub kind: DeliverableKind,
    pub title: String,
    pub required: bool,
    pub format: String,
    pub path_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalUnderstanding {
    pub original_prompt: String,
    pub goal_type: GoalType,
    pub deliverables: Vec<Deliverable>,
    pub constraints: Vec<String>,
    pub quality_requirements: Vec<String>,
    pub likely_tools: Vec<String>,
    pub risks: Vec<String>,
    pub needs_research: bool,
    pub needs_artifact: bool,
    pub needs_code: bool,
    pub needs_files: bool,
    pub confidence: f32,
}

pub fn understand_goal(prompt: &str) -> GoalUnderstanding {
    let lower = prompt.to_lowercase();
    let is_presentation = ["ppt", "presentation", "slides", "slide deck"]
        .iter()
        .any(|term| lower.contains(term));
    let is_learning_pack = lower.contains("learning pack")
        || lower.contains("study guide")
        || lower.contains("quiz")
        || lower.contains("glossary");
    let is_proposal = lower.contains("proposal")
        || lower.contains("roadmap")
        || lower.contains("risk")
        || lower.contains("budget");
    let is_release_kit = lower.contains("release kit")
        || lower.contains("launch kit")
        || lower.contains("startup launch")
        || lower.contains("startup pack")
        || lower.contains("product launch")
        || lower.contains("open-source launch")
        || lower.contains("github release")
        || lower.contains("release notes")
        || lower.contains("demo script")
        || lower.contains("social post")
        || lower.contains("social posts")
        || lower.contains("pitch deck")
        || lower.contains("landing page");
    let is_technical_report = lower.contains("technical report")
        || lower.contains("architecture report")
        || lower.contains("system design")
        || lower.contains("design document");
    let is_product_spec = lower.contains("product spec")
        || lower.contains("prd")
        || lower.contains("requirements")
        || lower.contains("product requirements");
    let is_docs_pack = lower.contains("documentation pack")
        || lower.contains("docs pack")
        || lower.contains("command guide")
        || lower.contains("api guide")
        || lower.contains("architecture guide")
        || lower.contains("user guide");
    let is_code = ["rust project", "code project", "cli", "cargo"]
        .iter()
        .any(|term| lower.contains(term));
    let is_debug = ["fix", "bug", "error", "debug"]
        .iter()
        .any(|term| lower.contains(term));
    let goal_type = if is_release_kit {
        if lower.contains("startup") {
            GoalType::StartupPack
        } else if lower.contains("marketing") {
            GoalType::MarketingPack
        } else {
            GoalType::LaunchKit
        }
    } else if is_technical_report {
        GoalType::TechnicalReport
    } else if is_product_spec {
        GoalType::ProductSpec
    } else if is_learning_pack {
        GoalType::LearningPack
    } else if is_docs_pack {
        GoalType::DocumentationPack
    } else if is_proposal {
        GoalType::Mixed
    } else if is_presentation {
        if lower.contains("pitch") {
            GoalType::PitchDeck
        } else {
            GoalType::Presentation
        }
    } else if is_code {
        GoalType::CodeProject
    } else if is_debug {
        GoalType::Debugging
    } else if lower.contains("benchmark") {
        GoalType::Benchmark
    } else if lower.contains("organize") {
        GoalType::FileOrganization
    } else if lower.contains("document") || lower.contains("markdown") {
        GoalType::Document
    } else {
        GoalType::Unknown
    };
    let mut deliverables = Vec::new();
    if is_release_kit {
        for (title, kind, file) in [
            (
                "Executive summary",
                DeliverableKind::ExecutiveSummary,
                "executive_summary.md",
            ),
            ("Pitch deck", DeliverableKind::PitchDeck, "pitch_deck.md"),
            (
                "Speaker notes",
                DeliverableKind::MarkdownDocument,
                "speaker_notes.md",
            ),
            (
                "Landing page copy",
                DeliverableKind::LandingPageCopy,
                "landing_page_copy.md",
            ),
            (
                "Release notes",
                DeliverableKind::ReleaseNotes,
                "release_notes.md",
            ),
            (
                "Changelog entry",
                DeliverableKind::MarkdownDocument,
                "changelog_entry.md",
            ),
            (
                "GitHub release draft",
                DeliverableKind::GitHubReleaseDraft,
                "github_release_draft.md",
            ),
            ("Demo script", DeliverableKind::DemoScript, "demo_script.md"),
            (
                "Technical overview",
                DeliverableKind::TechnicalOverview,
                "technical_overview.md",
            ),
            (
                "Architecture brief",
                DeliverableKind::ArchitectureBrief,
                "architecture_brief.md",
            ),
            ("FAQ", DeliverableKind::FAQ, "faq.md"),
            (
                "Email announcement",
                DeliverableKind::EmailAnnouncement,
                "email_announcement.md",
            ),
            (
                "Social posts",
                DeliverableKind::SocialPostSet,
                "social_posts.md",
            ),
            (
                "Risk register",
                DeliverableKind::RiskRegister,
                "risk_register.md",
            ),
            ("Roadmap", DeliverableKind::Roadmap, "roadmap.md"),
            (
                "Metrics plan",
                DeliverableKind::MetricsPlan,
                "metrics_plan.md",
            ),
            (
                "Launch checklist",
                DeliverableKind::LaunchChecklist,
                "launch_checklist.md",
            ),
            ("Final report", DeliverableKind::Report, "final_report.md"),
        ] {
            deliverables.push(deliverable(kind, title, "markdown", Some(file)));
        }
    } else if is_technical_report {
        for (title, kind, file) in [
            (
                "Technical report",
                DeliverableKind::TechnicalOverview,
                "technical_report.md",
            ),
            (
                "Architecture summary",
                DeliverableKind::ArchitectureBrief,
                "architecture_summary.md",
            ),
            (
                "Component map",
                DeliverableKind::ArchitectureDocument,
                "component_map.md",
            ),
            (
                "Safety model",
                DeliverableKind::SecurityNotes,
                "safety_model.md",
            ),
            (
                "Limitations",
                DeliverableKind::MarkdownDocument,
                "limitations.md",
            ),
            ("Test plan", DeliverableKind::TestPlan, "test_plan.md"),
            (
                "Risk register",
                DeliverableKind::RiskRegister,
                "risk_register.md",
            ),
            ("Final report", DeliverableKind::Report, "final_report.md"),
        ] {
            deliverables.push(deliverable(kind, title, "markdown", Some(file)));
        }
    } else if is_product_spec {
        for (title, kind, file) in [
            (
                "Product spec",
                DeliverableKind::ProductSpec,
                "product_spec.md",
            ),
            (
                "User stories",
                DeliverableKind::UserStories,
                "user_stories.md",
            ),
            (
                "Acceptance criteria",
                DeliverableKind::AcceptanceCriteria,
                "acceptance_criteria.md",
            ),
            ("Roadmap", DeliverableKind::Roadmap, "roadmap.md"),
            (
                "Risk register",
                DeliverableKind::RiskRegister,
                "risk_register.md",
            ),
            (
                "Metrics plan",
                DeliverableKind::MetricsPlan,
                "metrics_plan.md",
            ),
            ("Final report", DeliverableKind::Report, "final_report.md"),
        ] {
            deliverables.push(deliverable(kind, title, "markdown", Some(file)));
        }
    } else if is_docs_pack {
        for (title, kind, file) in [
            ("Overview", DeliverableKind::MarkdownDocument, "overview.md"),
            ("User guide", DeliverableKind::UserGuide, "user_guide.md"),
            (
                "Command reference",
                DeliverableKind::MarkdownDocument,
                "command_reference.md",
            ),
            (
                "Architecture summary",
                DeliverableKind::ArchitectureDocument,
                "architecture_summary.md",
            ),
            (
                "Troubleshooting",
                DeliverableKind::MarkdownDocument,
                "troubleshooting.md",
            ),
            ("FAQ", DeliverableKind::FAQ, "faq.md"),
            ("Final report", DeliverableKind::Report, "final_report.md"),
        ] {
            deliverables.push(deliverable(kind, title, "markdown", Some(file)));
        }
    } else if is_learning_pack {
        deliverables.push(deliverable(
            DeliverableKind::LessonPlan,
            "Lesson plan",
            "markdown",
            Some("lesson_plan.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::PresentationMarkdown,
            "Slide deck",
            "markdown",
            Some("slide_deck.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Speaker notes",
            "markdown",
            Some("speaker_notes.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::StudyGuide,
            "Study guide",
            "markdown",
            Some("study_guide.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Quiz,
            "Quiz",
            "markdown",
            Some("quiz.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Answer key",
            "markdown",
            Some("answer_key.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Glossary,
            "Glossary",
            "markdown",
            Some("glossary.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Design guide",
            "markdown",
            Some("design_guide.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Checklist,
            "Practice tasks",
            "markdown",
            Some("practice_tasks.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Teacher notes",
            "markdown",
            Some("teacher_notes.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Report,
            "Final report",
            "markdown",
            Some("final_report.md"),
        ));
    } else if is_proposal {
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Project proposal",
            "markdown",
            Some("proposal.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Roadmap,
            "Roadmap",
            "markdown",
            Some("roadmap.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::RiskRegister,
            "Risk register",
            "markdown",
            Some("risk_register.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::ArchitectureDocument,
            "Architecture summary",
            "markdown",
            Some("architecture.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::BudgetTable,
            "Budget table",
            "markdown",
            Some("budget_table.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Report,
            "Final report",
            "markdown",
            Some("final_report.md"),
        ));
    } else if matches!(goal_type, GoalType::Presentation | GoalType::PitchDeck) {
        deliverables.push(deliverable(
            DeliverableKind::SlideOutline,
            "Slide outline",
            "markdown",
            Some("presentation.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::PresentationMarkdown,
            "Export-ready markdown deck",
            "markdown",
            Some("presentation.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Speaker notes",
            "markdown",
            Some("speaker_notes.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Design guide",
            "markdown",
            Some("design_guide.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Report,
            "Final report",
            "markdown",
            Some("final_report.md"),
        ));
    } else if matches!(goal_type, GoalType::CodeProject | GoalType::CodePackage) {
        deliverables.push(deliverable(
            DeliverableKind::CodeProject,
            "Sandbox code project",
            "rust",
            None,
        ));
        deliverables.push(deliverable(
            DeliverableKind::Report,
            "Final report",
            "markdown",
            None,
        ));
    } else {
        deliverables.push(deliverable(
            DeliverableKind::MarkdownDocument,
            "Summary document",
            "markdown",
            Some("document.md"),
        ));
        deliverables.push(deliverable(
            DeliverableKind::Report,
            "Final report",
            "markdown",
            Some("final_report.md"),
        ));
    }
    let needs_research = lower.contains("research") || lower.contains("citations");
    let mut constraints = vec![
        "sandboxed file writes only".to_string(),
        "no network by default".to_string(),
        "bounded task count".to_string(),
    ];
    if lower.contains("citation") {
        constraints
            .push("citation placeholders only unless research is explicitly enabled".to_string());
    }
    GoalUnderstanding {
        original_prompt: prompt.to_string(),
        goal_type,
        deliverables,
        constraints,
        quality_requirements: vec![
            "validate required deliverables".to_string(),
            "write final report".to_string(),
            "avoid AGI/consciousness claims".to_string(),
        ],
        likely_tools: if is_code {
            vec![
                "code_editor".to_string(),
                "terminal_cargo_allowlist".to_string(),
            ]
        } else {
            vec!["filesystem".to_string(), "artifact_writer".to_string()]
        },
        risks: vec![
            "missing deliverables".to_string(),
            "exceeding bounded autonomy limits".to_string(),
        ],
        needs_research,
        needs_artifact: !is_code || is_presentation,
        needs_code: is_code,
        needs_files: true,
        confidence: if is_presentation || is_code {
            0.9
        } else {
            0.55
        },
    }
}

fn deliverable(
    kind: DeliverableKind,
    title: &str,
    format: &str,
    path_hint: Option<&str>,
) -> Deliverable {
    Deliverable {
        kind,
        title: title.to_string(),
        required: true,
        format: format.to_string(),
        path_hint: path_hint.map(ToOwned::to_owned),
    }
}

pub fn requested_slide_count(prompt: &str) -> usize {
    let words = prompt.split_whitespace().collect::<Vec<_>>();
    for (index, word) in words.iter().enumerate() {
        let cleaned = word.trim_matches(|ch: char| !ch.is_ascii_digit());
        if let Ok(count) = cleaned.parse::<usize>() {
            if words
                .get(index + 1)
                .is_some_and(|next| next.to_lowercase().starts_with("slide"))
                || words
                    .get(index.saturating_sub(1))
                    .is_some_and(|prev| prev.to_lowercase().contains("slide"))
            {
                return count.clamp(1, 40);
            }
        }
    }
    10
}

pub fn presentation_topic(prompt: &str) -> String {
    let lower = prompt.to_lowercase();
    for marker in ["about ", "on "] {
        if let Some(start) = lower.find(marker) {
            let original_start = start + marker.len();
            let topic = prompt[original_start..]
                .split(" with ")
                .next()
                .unwrap_or(&prompt[original_start..])
                .split(" for ")
                .next()
                .unwrap_or(&prompt[original_start..])
                .trim_matches(|ch: char| ch == '"' || ch == '.')
                .trim();
            if !topic.is_empty() {
                return title_case(topic);
            }
        }
    }
    "Brain-Inspired AI".to_string()
}

pub fn presentation_audience(prompt: &str) -> String {
    let lower = prompt.to_lowercase();
    if let Some(start) = lower.find(" for ") {
        let audience = prompt[start + 5..]
            .split(" with ")
            .next()
            .unwrap_or(&prompt[start + 5..])
            .trim_matches(|ch: char| ch == '"' || ch == '.')
            .trim();
        if !audience.is_empty() {
            return audience.to_string();
        }
    }
    "general audience".to_string()
}

fn title_case(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
