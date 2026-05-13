pub fn technical_overview(topic: &str) -> String {
    format!("# Technical Overview: {topic}\n\n## Runtime Model\nDisk-backed state, sparse active execution, deterministic planning, bounded artifact generation, and recovery-first reliability systems.\n\n## Key Components\n- Goal understanding\n- Task graph and scheduler\n- Artifact generators\n- Validator and repair loop\n- Journal, snapshots, rollback, doctor\n\n## Verification Notes\nArchitecture claims should be checked against the repository before publication.\n")
}

pub fn architecture_brief(topic: &str) -> String {
    format!("# Architecture Brief: {topic}\n\n## Flow\nPrompt -> understanding -> work contract -> task graph -> artifacts -> review -> export -> report card.\n\n## Storage\nRuntime data is disk-backed under `data/` and generated outputs stay under `sandbox/`.\n\n## Safety Boundaries\nNo unrestricted shell, no network by default, no outside-sandbox generated files.\n")
}

pub fn component_map(topic: &str) -> String {
    format!("# Component Map: {topic}\n\n| Component | Role |\n| --- | --- |\n| agency | Planning, sessions, autonomy limits |\n| artifacts | Markdown generators and manifests |\n| memory | Reflections and reusable workflow data |\n| storage | Disk-backed JSON state |\n| testing | Regression guard |\n")
}

pub fn safety_model_doc(topic: &str) -> String {
    format!("# Safety Model: {topic}\n\n## Boundaries\n- Sandboxed generated files\n- Allowlisted terminal commands\n- No network access by default\n- Bounded tasks and retries\n\n## Reliability\nAction journal, snapshots, rollback, doctor, and regression-check support recoverable operation.\n")
}

pub fn test_plan(topic: &str) -> String {
    format!("# Test Plan: {topic}\n\n## Commands\n- `cargo fmt`\n- `cargo check`\n- `cargo test -- --nocapture`\n- `cargo run -- doctor`\n- `cargo run -- regression-check`\n\n## Artifact Checks\n- Manifest exists\n- Final report references all artifacts\n- No unsupported claims\n")
}

pub fn contributor_guide(topic: &str) -> String {
    format!("# Contributor Guide: {topic}\n\n## Setup\nRun Rust tooling locally and keep generated runtime data out of commits.\n\n## Safety Rules\nDo not add unrestricted shell execution, network access by default, unsafe Rust, or outside-sandbox writes.\n")
}

pub fn security_notes(topic: &str) -> String {
    format!("# Security Notes: {topic}\n\nOnyx Brain is experimental. Keep file operations sandboxed, terminal commands allowlisted, and generated outputs reviewed before use in sensitive contexts.\n")
}

pub fn limitations_doc(topic: &str) -> String {
    format!("# Limitations: {topic}\n\n- No LLM by default\n- No web research by default\n- Markdown artifact output only for decks\n- Factual claims need external verification\n- Bounded autonomy is not AGI or consciousness\n")
}

pub fn technical_report(topic: &str) -> String {
    format!("# Technical Report: {topic}\n\n## Summary\nA bounded autonomous worker runtime can coordinate local planning, artifacts, validation, repair, and reporting while preserving explicit safety limits.\n\n## Design\nDisk-backed state and task-local activation reduce RAM pressure. Reports explain decisions and limitations.\n\n## Risks\nOverstated claims, missing validation, and unsafe paths are handled by quality review and doctor checks.\n")
}
