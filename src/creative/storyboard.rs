use crate::creative::VideoEditPlan;

pub fn storyboard_text(plan: &VideoEditPlan) -> String {
    let panels = plan
        .scenes
        .iter()
        .map(|scene| {
            format!(
                "- Panel {}: {} with original visual motifs.",
                scene.scene_number, scene.title
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("# Storyboard Text\n\n{panels}\n")
}
