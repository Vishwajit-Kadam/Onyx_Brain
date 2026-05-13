# Learning Pack Example

Create a bounded autonomous learning pack:

```bash
cargo run -- init
cargo run -- autonomize --level full-bounded "Create a complete learning pack about brain-inspired AI for students with a 10-slide deck, speaker notes, study guide, quiz, glossary, design guide, and final report"
cargo run -- artifact-packs
cargo run -- artifact-pack-inspect latest
cargo run -- review-artifacts latest
cargo run -- export-package latest
```

Expected result:

- markdown slide deck
- speaker notes
- study guide
- quiz with answer key
- glossary
- design guide
- assumptions and limitations logs
- artifact pack manifest
- final report

The workflow stays inside the sandbox and uses deterministic local generation.
