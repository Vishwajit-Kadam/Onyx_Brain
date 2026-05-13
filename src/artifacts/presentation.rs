use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub number: usize,
    pub title: String,
    pub bullets: Vec<String>,
    pub speaker_notes: String,
    pub visual_suggestion: String,
    pub validation_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationArtifact {
    pub title: String,
    pub audience: String,
    pub slide_count: usize,
    pub slides: Vec<Slide>,
    pub design_notes: String,
    pub speaker_notes: String,
    pub validation_summary: String,
}

pub fn build_presentation(topic: &str, audience: &str, slide_count: usize) -> PresentationArtifact {
    let count = slide_count.max(1);
    let mut slides = Vec::new();
    for number in 1..=count {
        let title = match number {
            1 => format!("{topic}: Overview"),
            2 => "Why brain-inspired systems matter".to_string(),
            3 => "Sparse activation".to_string(),
            4 => "Memory hierarchy".to_string(),
            5 => "Routing and energy budgets".to_string(),
            6 => "Skills and habits".to_string(),
            7 => "Safety and bounded autonomy".to_string(),
            8 => "Validation and recovery".to_string(),
            9 => "Limitations and responsible framing".to_string(),
            _ if number == count => "Key takeaways".to_string(),
            _ => format!("Applied idea {number}"),
        };
        slides.push(Slide {
            number,
            title: title.clone(),
            bullets: vec![
                format!("Core idea for {audience}"),
                "Concrete example or classroom discussion prompt".to_string(),
                "Safety-aware implementation note".to_string(),
            ],
            speaker_notes: format!(
                "Explain {title} in practical terms for {audience}. Emphasize that the system is brain-inspired, not conscious or AGI."
            ),
            visual_suggestion: "Use a simple diagram with labeled modules and arrows.".to_string(),
            validation_notes: "Title, bullets, notes, and visual suggestion present.".to_string(),
        });
    }
    PresentationArtifact {
        title: topic.to_string(),
        audience: audience.to_string(),
        slide_count: count,
        slides,
        design_notes: "Use a clean high-contrast academic style, one core idea per slide, and restrained diagrams instead of hype imagery.".to_string(),
        speaker_notes: "Speaker notes are included per slide in the deck and collected in speaker_notes.md.".to_string(),
        validation_summary: "Generated deterministically from prompt requirements.".to_string(),
    }
}

pub fn render_presentation_markdown(presentation: &PresentationArtifact) -> String {
    let mut out = format!(
        "# {}\n\nAudience: {}\n\n",
        presentation.title, presentation.audience
    );
    for slide in &presentation.slides {
        out.push_str(&format!("## Slide {}: {}\n", slide.number, slide.title));
        for bullet in &slide.bullets {
            out.push_str(&format!("- {bullet}\n"));
        }
        out.push_str(&format!(
            "\nSpeaker Notes:\n{}\n\nVisual Suggestion:\n{}\n\n",
            slide.speaker_notes, slide.visual_suggestion
        ));
    }
    out
}

pub fn render_speaker_notes(presentation: &PresentationArtifact) -> String {
    let mut out = format!("# Speaker Notes: {}\n\n", presentation.title);
    for slide in &presentation.slides {
        out.push_str(&format!(
            "## Slide {}: {}\n{}\n\n",
            slide.number, slide.title, slide.speaker_notes
        ));
    }
    out
}

pub fn render_design_guide(presentation: &PresentationArtifact) -> String {
    format!(
        "# Design Guide: {}\n\n{}\n\n## Visual Direction\n- Diagrams over decorative imagery\n- Clear slide titles\n- Two to three bullets per slide\n- Speaker-first notes\n",
        presentation.title, presentation.design_notes
    )
}
