use serde::{Deserialize, Serialize};

use crate::creative::{CreativeProject, CreativeProjectType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEditPlan {
    pub title: String,
    pub duration_minutes: u32,
    pub acts: Vec<StoryAct>,
    pub scenes: Vec<ScenePlan>,
    pub timeline: TimelinePlan,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryAct {
    pub number: u32,
    pub title: String,
    pub purpose: String,
    pub estimated_duration_minutes: u32,
    pub emotional_goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenePlan {
    pub scene_number: u32,
    pub title: String,
    pub summary: String,
    pub duration_minutes: u32,
    pub location: String,
    pub characters: Vec<String>,
    pub shots: Vec<ShotPlan>,
    pub sound_notes: String,
    pub vfx_notes: String,
    pub edit_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotPlan {
    pub shot_number: u32,
    pub shot_type: String,
    pub camera_motion: String,
    pub description: String,
    pub duration_seconds: u32,
    pub transition: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePlan {
    pub total_duration_minutes: u32,
    pub segments: Vec<String>,
    pub pacing_notes: String,
    pub continuity_notes: String,
}

pub fn build_video_edit_plan(project: &CreativeProject, _prompt: &str) -> VideoEditPlan {
    let duration = project.duration_minutes.unwrap_or(12);
    let scenes = (1..=6)
        .map(|number| scene(number, duration / 6))
        .collect::<Vec<_>>();
    VideoEditPlan {
        title: project.title.clone(),
        duration_minutes: duration,
        acts: vec![
            act(1, "Arrival", duration / 3, "wonder and orientation"),
            act(2, "Pressure", duration / 3, "tension and discovery"),
            act(
                3,
                "Resolution",
                duration - (duration / 3 * 2),
                "clarity and release",
            ),
        ],
        scenes,
        timeline: TimelinePlan {
            total_duration_minutes: duration,
            segments: vec![
                "opening hook".to_string(),
                "rising discovery".to_string(),
                "final synthesis".to_string(),
            ],
            pacing_notes: if project.project_type == CreativeProjectType::FeatureFilm {
                "Use act-level rhythm and scene clusters; this is a planning document, not rendered footage.".to_string()
            } else {
                "Keep pacing concise and legible.".to_string()
            },
            continuity_notes:
                "Track spatial orientation, emotional continuity, and recurring visual motifs."
                    .to_string(),
        },
        deliverables: project
            .deliverables
            .iter()
            .map(|row| format!("{:?}", row))
            .collect(),
    }
}

fn act(number: u32, title: &str, duration: u32, goal: &str) -> StoryAct {
    StoryAct {
        number,
        title: title.to_string(),
        purpose: format!("Act {number} establishes {title}."),
        estimated_duration_minutes: duration.max(1),
        emotional_goal: goal.to_string(),
    }
}

fn scene(number: u32, duration: u32) -> ScenePlan {
    ScenePlan {
        scene_number: number,
        title: format!("Scene {number}: Signal {number}"),
        summary: "Original sci-fi scene beat focused on discovery, stakes, and character choice.".to_string(),
        duration_minutes: duration.max(1),
        location: "original orbital research setting".to_string(),
        characters: vec!["Lead engineer".to_string(), "Mission analyst".to_string()],
        shots: vec![shot(1), shot(2), shot(3)],
        sound_notes: "Layer restrained ambience, low-frequency movement, and clear dialogue space.".to_string(),
        vfx_notes: "Use original interface language, abstract light fields, and non-franchise visual motifs.".to_string(),
        edit_notes: "Cut on intent changes; preserve geography and emotional continuity.".to_string(),
    }
}

fn shot(number: u32) -> ShotPlan {
    ShotPlan {
        shot_number: number,
        shot_type: ["wide", "medium", "insert"][(number as usize - 1).min(2)].to_string(),
        camera_motion: "controlled drift".to_string(),
        description: "Show original environment detail without copying protected designs."
            .to_string(),
        duration_seconds: 8,
        transition: "motivated cut".to_string(),
        notes: "Planning note only; no video is rendered.".to_string(),
    }
}
