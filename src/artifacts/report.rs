pub fn final_report_markdown(
    goal: &str,
    status: &str,
    artifacts: &[String],
    validation_score: f32,
    repairs: usize,
    safety_note: &str,
) -> String {
    format!(
        "# Autonomous Worker Final Report\n\nGoal: {goal}\n\nStatus: {status}\n\nValidation score: {validation_score:.2}\n\nRepairs performed: {repairs}\n\nArtifacts:\n{}\n\nSafety: {safety_note}\n",
        artifacts
            .iter()
            .map(|path| format!("- {path}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
