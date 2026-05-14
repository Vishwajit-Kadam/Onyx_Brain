//! Core type aliases — intentionally small.
//!
//! Provides stable, descriptive type aliases for IDs used throughout the codebase.
//! A separate file keeps these types easily discoverable and importable without
//! pulling in heavy dependencies.
pub type NeuronId = String;
pub type SynapseId = String;
pub type MemoryId = String;
pub type TaskId = String;

pub fn stable_id(label: &str) -> String {
    label
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}
