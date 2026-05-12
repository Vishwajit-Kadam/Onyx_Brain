use onyx_brain::{
    core::{Synapse, SynapseType},
    sleep::consolidate,
    storage::DiskStore,
};

#[test]
fn consolidation_prunes_weak_synapses() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let mut weak = Synapse::new("weak", "a", "b", SynapseType::Excitatory, 0.1);
    weak.confidence = 0.05;
    store.save_synapse(&weak).expect("save");

    let report = consolidate(&store).expect("consolidate");
    assert_eq!(report.pruned_synapses, 1);
    assert!(!store.synapse_path("weak").exists());
}
