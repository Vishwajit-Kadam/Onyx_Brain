# Release Kit Example

```bash
cargo run -- init
cargo run -- autonomize --level full-bounded "Create a full launch kit for Onyx Brain v0.0.2 including release notes, changelog entry, GitHub release draft, demo script, technical overview, FAQ, risk notes, social posts, launch checklist, and final report"
cargo run -- artifact-packs
cargo run -- review-artifacts latest
cargo run -- export-package latest
```

Expected outputs include release notes, changelog entry, release draft, demo script, technical overview, FAQ, risk notes, social posts, launch checklist, and a final report.
