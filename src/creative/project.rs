use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeProject {
    pub project_id: String,
    pub session_id: String,
    pub title: String,
    pub project_type: CreativeProjectType,
    pub duration_minutes: Option<u32>,
    pub genre: Option<String>,
    pub target_style: Option<String>,
    pub deliverables: Vec<CreativeDeliverable>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreativeProjectType {
    ShortFilm,
    FeatureFilm,
    Trailer,
    YouTubeVideo,
    MusicVideo,
    Documentary,
    AdCampaign,
    GameCinematic,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreativeDeliverable {
    CreativeBrief,
    StoryOutline,
    SceneBreakdown,
    ShotList,
    StoryboardText,
    TimelinePlan,
    EditDecisionList,
    SoundDesignPlan,
    VfxPlan,
    ColorGradePlan,
    SubtitleScript,
    TrailerPlan,
    ReviewChecklist,
    FinalProductionReport,
}

pub fn parse_creative_project(session_id: &str, prompt: &str) -> CreativeProject {
    let lower = prompt.to_lowercase();
    let project_type = if lower.contains("feature") || lower.contains("3-hour") {
        CreativeProjectType::FeatureFilm
    } else if lower.contains("trailer") {
        CreativeProjectType::Trailer
    } else if lower.contains("documentary") {
        CreativeProjectType::Documentary
    } else {
        CreativeProjectType::Unknown
    };
    CreativeProject {
        project_id: format!("creative_project_{}", Uuid::new_v4()),
        session_id: session_id.to_string(),
        title: title_from_prompt(prompt),
        project_type,
        duration_minutes: if lower.contains("3-hour") {
            Some(180)
        } else {
            Some(12)
        },
        genre: lower
            .contains("sci-fi")
            .then(|| "original sci-fi".to_string()),
        target_style: lower
            .contains("cinematic")
            .then(|| "cinematic planning".to_string()),
        deliverables: default_deliverables(),
        created_at: Utc::now(),
    }
}

fn default_deliverables() -> Vec<CreativeDeliverable> {
    vec![
        CreativeDeliverable::CreativeBrief,
        CreativeDeliverable::StoryOutline,
        CreativeDeliverable::SceneBreakdown,
        CreativeDeliverable::ShotList,
        CreativeDeliverable::TimelinePlan,
        CreativeDeliverable::EditDecisionList,
        CreativeDeliverable::SoundDesignPlan,
        CreativeDeliverable::VfxPlan,
        CreativeDeliverable::ColorGradePlan,
        CreativeDeliverable::SubtitleScript,
        CreativeDeliverable::ReviewChecklist,
        CreativeDeliverable::FinalProductionReport,
    ]
}

fn title_from_prompt(prompt: &str) -> String {
    if prompt.to_lowercase().contains("sci-fi") {
        "Original Sci-Fi Feature Production Plan".to_string()
    } else {
        "Creative Production Plan".to_string()
    }
}
