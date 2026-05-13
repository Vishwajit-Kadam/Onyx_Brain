# Project Proposal Example

Create a local proposal pack:

```bash
cargo run -- init
cargo run -- autonomize --level full-bounded "Create a project proposal with roadmap, risks, architecture, budget table, and final report"
cargo run -- artifact-packs
cargo run -- artifact-pack-inspect latest
cargo run -- export-package latest
```

Expected result:

- proposal markdown
- roadmap
- risk register
- architecture summary
- budget table
- final report
- artifact pack metadata

This is a deterministic bounded-autonomy workflow, not an AGI or conscious system.
