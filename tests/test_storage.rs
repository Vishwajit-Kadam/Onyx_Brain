use onyx_brain::{
    core::{NeuronKind, VirtualNeuron},
    storage::DiskStore,
};

#[test]
fn neuron_can_be_saved_and_loaded_from_disk() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let neuron = VirtualNeuron::new("n1", "test neuron", NeuronKind::Concept);
    store.save_neuron(&neuron).expect("save");
    let loaded = store.load_neuron("n1").expect("load");
    assert_eq!(loaded.id, "n1");
    assert_eq!(loaded.label, "test neuron");
}
