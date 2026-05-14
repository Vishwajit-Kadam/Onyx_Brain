use crate::creative::VideoEditPlan;

pub fn shot_list(plan: &VideoEditPlan) -> String {
    let rows = plan
        .scenes
        .iter()
        .flat_map(|scene| {
            scene.shots.iter().map(move |shot| {
                format!(
                    "- Scene {} Shot {}: {} / {} / {} sec / {}",
                    scene.scene_number,
                    shot.shot_number,
                    shot.shot_type,
                    shot.camera_motion,
                    shot.duration_seconds,
                    shot.description
                )
            })
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Shot List\n\n{rows}\n")
}
