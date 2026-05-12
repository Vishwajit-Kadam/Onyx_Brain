use onyx_brain::{
    core::{Synapse, SynapseType},
    learning::update_routes,
    storage::DiskStore,
};

#[test]
fn successful_route_strengthens_synapse() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let synapse = Synapse::new("s1", "a", "b", SynapseType::Excitatory, 0.2);
    store.save_synapse(&synapse).expect("save");
    update_routes(
        &store,
        &[synapse],
        &["a".to_string(), "b".to_string()],
        true,
    )
    .expect("learn");
    let updated = store.load_synapse("s1").expect("load");
    assert!(updated.weight > 0.2);
    assert_eq!(updated.success_score, 1.0);
}
