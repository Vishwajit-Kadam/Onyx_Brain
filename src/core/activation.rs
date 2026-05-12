pub fn should_activate(score: f32, threshold: f32) -> bool {
    score >= threshold
}

pub fn integrate_activation(
    base_activation: f32,
    task_relevance: f32,
    incoming_excitation: f32,
    incoming_inhibition: f32,
    memory_relevance: f32,
    past_success_bonus: f32,
    energy_penalty: f32,
) -> f32 {
    base_activation + task_relevance + incoming_excitation - incoming_inhibition
        + memory_relevance
        + past_success_bonus
        - energy_penalty
}

pub fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}
