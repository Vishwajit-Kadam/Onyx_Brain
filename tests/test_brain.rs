use onyx_brain::Brain;

#[test]
fn brain_think_flow_uses_limited_active_state_and_creates_demo_project() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .think("Create a Rust hello world project called mini_hello".to_string())
        .expect("think");

    assert!(output.activated_neurons.len() <= 32);
    assert!(output
        .activated_experts
        .iter()
        .any(|name| name == "CodeExpert"));
    assert!(output.used_memories.iter().any(|id| id == "memory_rust"));
    assert!(temp
        .path()
        .join("sandbox/projects/mini_hello/Cargo.toml")
        .exists());
}
