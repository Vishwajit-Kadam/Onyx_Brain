# Artifact Packs

Artifact packs group related generated files, dependencies, validation scores, quality review output, assumptions, limitations, self questions, report cards, and final reports.

Common commands:

```bash
cargo run -- artifact-packs
cargo run -- artifact-pack-inspect latest
cargo run -- review-artifacts latest
cargo run -- repair-artifacts latest
cargo run -- export-package latest
cargo run -- export-inspect latest
```

Packs are written under `sandbox/workspaces/{session_id}/artifacts/` and remain local by default.
