use crate::creative::{CreativeProject, VideoEditPlan};

pub fn creative_brief(project: &CreativeProject, prompt: &str) -> String {
    format!(
        "# Creative Brief: {}\n\n## Goal\n{}\n\n## Boundary\nThis creates a planning package only. It does not render actual video.\n\n## Originality\nUse original characters, worlds, names, and visual motifs.\n",
        project.title, prompt
    )
}

pub fn story_outline(plan: &VideoEditPlan) -> String {
    let acts = plan
        .acts
        .iter()
        .map(|act| {
            format!(
                "- Act {}: {} ({} min) - {}",
                act.number, act.title, act.estimated_duration_minutes, act.emotional_goal
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Story Outline: {}\n\n{}\n", plan.title, acts)
}

pub fn scene_breakdown(plan: &VideoEditPlan) -> String {
    let scenes = plan
        .scenes
        .iter()
        .map(|scene| {
            format!(
                "## {}\n{}\n\nDuration: {} min\nSound: {}\nVFX: {}\nEdit: {}\n",
                scene.title,
                scene.summary,
                scene.duration_minutes,
                scene.sound_notes,
                scene.vfx_notes,
                scene.edit_notes
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Scene Breakdown\n\n{scenes}")
}

pub fn edit_decision_list(plan: &VideoEditPlan) -> String {
    let rows = plan
        .scenes
        .iter()
        .map(|scene| {
            format!(
                "- Scene {}: motivated cuts, continuity checks, transition by intent.",
                scene.scene_number
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Edit Decision List\n\n{rows}\n")
}

pub fn sound_design_plan(plan: &VideoEditPlan) -> String {
    let rows = plan
        .scenes
        .iter()
        .map(|scene| format!("- {}: {}", scene.title, scene.sound_notes))
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Sound Design Plan\n\n{rows}\n")
}

pub fn vfx_plan(project: &CreativeProject, plan: &VideoEditPlan) -> String {
    let rows = plan
        .scenes
        .iter()
        .map(|scene| format!("- {}: {}", scene.title, scene.vfx_notes))
        .collect::<Vec<_>>()
        .join("\n");
    format!("# VFX Plan: {}\n\n{rows}\n\nOriginality note: avoid copying protected franchise designs.\n", project.title)
}

pub fn color_grade_plan(project: &CreativeProject) -> String {
    format!("# Color Grade Plan: {}\n\n- Maintain a distinctive original palette.\n- Separate calm discovery scenes from high-pressure scenes.\n- Avoid copying recognizable franchise color identities.\n", project.title)
}

pub fn subtitle_script(plan: &VideoEditPlan) -> String {
    let rows = plan
        .scenes
        .iter()
        .map(|scene| {
            format!(
                "[Scene {}] Placeholder dialogue subtitle line.",
                scene.scene_number
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Subtitle Script\n\n{rows}\n")
}
