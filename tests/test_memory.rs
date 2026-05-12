use onyx_brain::{
    core::{Task, TaskType},
    memory::{retrieve_relevant_memories, MemoryItem, MemoryType},
    storage::DiskStore,
};

#[test]
fn memory_retrieval_returns_only_top_relevant_items() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    store
        .save_memory(&MemoryItem::new(
            "rust",
            MemoryType::Semantic,
            "Rust",
            "Rust owns memory safely.",
            vec!["rust".to_string()],
            vec![],
        ))
        .expect("save rust memory");
    store
        .save_memory(&MemoryItem::new(
            "gardening",
            MemoryType::Semantic,
            "Gardening",
            "Soil and seeds.",
            vec!["garden".to_string()],
            vec![],
        ))
        .expect("save unrelated memory");

    let task = Task::new("Write Rust code".to_string(), TaskType::Code);
    let memories = retrieve_relevant_memories(&store, &task, &[], 1).expect("retrieve");
    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0].id, "rust");
}
