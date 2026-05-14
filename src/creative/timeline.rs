use crate::creative::TimelinePlan;

pub fn timeline_markdown(timeline: &TimelinePlan) -> String {
    format!(
        "# Timeline Plan\n\nTotal duration: {} minutes\n\n## Segments\n{}\n\n## Pacing Notes\n{}\n\n## Continuity Notes\n{}\n",
        timeline.total_duration_minutes,
        timeline.segments.iter().map(|row| format!("- {row}")).collect::<Vec<_>>().join("\n"),
        timeline.pacing_notes,
        timeline.continuity_notes
    )
}
