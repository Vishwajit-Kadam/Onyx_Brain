# Calculator Worker Example

This example runs a bounded worker task that creates and improves a small Rust calculator project inside the sandbox.

```bash
cargo run -- init
cargo run -- worker "Create and improve a Rust calculator project called worker_calc"
cargo run -- doctor
cargo run -- regression-check
cargo run -- brain-status
```

Expected high-level behavior:

- creates `sandbox/projects/worker_calc`
- journals file and command actions
- creates snapshots where useful
- writes files through sandboxed tools
- runs safe validation when available
- records sessions, transactions, and reports under `data/`
- stays inside the sandbox

The exact output depends on the local Rust toolchain and existing generated runtime state.
