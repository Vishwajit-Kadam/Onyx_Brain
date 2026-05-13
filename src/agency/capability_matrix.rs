use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityMatrix {
    pub can_do: Vec<String>,
    pub cannot_do: Vec<String>,
    pub safety_boundaries: Vec<String>,
    pub notes: Vec<String>,
}

pub fn capability_matrix() -> CapabilityMatrix {
    CapabilityMatrix {
        can_do: vec![
            "create deterministic markdown artifact packs".to_string(),
            "create sandboxed Rust projects and reports".to_string(),
            "validate and repair generated artifacts within limits".to_string(),
            "journal actions, create snapshots, and support rollback".to_string(),
            "run bounded worker, autonomize, queue, benchmark, doctor, and regression commands"
                .to_string(),
        ],
        cannot_do: vec![
            "browse the web by default".to_string(),
            "create binary PPTX files in v0.0.2".to_string(),
            "run unrestricted shell commands".to_string(),
            "access files outside sandbox/workspace boundaries by default".to_string(),
            "guarantee factual correctness without external verification".to_string(),
            "provide consciousness, AGI, or superintelligence".to_string(),
        ],
        safety_boundaries: vec![
            "sandboxed writes".to_string(),
            "allowlisted terminal commands".to_string(),
            "finite tasks, retries, runtime, and generated file sizes".to_string(),
            "no hidden background workers".to_string(),
        ],
        notes: vec![
            "Onyx Brain is an experimental bounded autonomous worker runtime.".to_string(),
            "It does not include an LLM by default.".to_string(),
        ],
    }
}
