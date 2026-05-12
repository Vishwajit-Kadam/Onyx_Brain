# Contributing

Thanks for helping improve Onyx Brain. This project is experimental, so contributions should keep the safety model simple, inspectable, and conservative.

## Setup

Install Rust, clone the repository, then run:

```bash
cargo fmt
cargo check
cargo test -- --nocapture
```

## Coding Style

- Prefer small, deterministic modules.
- Keep state disk-backed where practical.
- Use `Result` and clear error messages.
- Avoid unrelated refactors in feature patches.
- Keep tests focused on behavior and safety.

## Safety Rules

- Do not add unrestricted shell execution.
- Do not add network access by default.
- Do not bypass sandbox path checks.
- Do not write generated project files outside `sandbox/`.
- Do not permanently delete recovery data.
- Preserve journals, snapshots, rollback, doctor, and regression checks.
- Keep terminal commands allowlisted.
- Keep the active runtime RAM-minimal.

## Pull Requests

For pull requests:

- Describe the change and why it is needed.
- Include tests for behavior and safety-sensitive paths.
- Run `cargo fmt`, `cargo check`, and `cargo test -- --nocapture`.
- Mention any generated files that should stay ignored.

## Issues

Please include:

- operating system
- Rust version
- command run
- expected behavior
- actual behavior
- relevant output or report paths
