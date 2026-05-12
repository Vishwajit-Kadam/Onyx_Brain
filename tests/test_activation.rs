use onyx_brain::core::should_activate;

#[test]
fn neuron_activates_only_at_threshold() {
    assert!(!should_activate(0.49, 0.5));
    assert!(should_activate(0.5, 0.5));
}
