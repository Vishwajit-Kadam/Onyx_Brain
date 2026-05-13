# Autonomy

Onyx Brain v0.0.2 introduces a bounded autonomous worker engine. It is deterministic by default and does not claim consciousness, AGI, or true intelligence.

## Autonomy Levels

- `assisted`: conservative mode intended for workflows that may need review.
- `standard`: default bounded autonomy.
- `high`: more autonomous planning inside the same limits.
- `full-bounded`: no follow-up questions; make reasonable assumptions and stop safely when blocked.

## Safety Limits

The default policy limits sessions to finite runtime, tasks, phases, retries, tool actions, artifact count, generated file size, and context files read. Network and unrestricted shell execution are disabled by default.

## What Full-Bounded Means

Full-bounded means autonomous inside hard safety limits:

- sandboxed writes only
- allowlisted commands only
- no background execution
- no network by default
- deterministic artifact creation
- validation and repair before final report

It does not mean unrestricted access, AGI, consciousness, or production readiness.

Advanced runs also write self-questions, knowledge gaps, local-only research notes, a work contract, a done definition, an execution trace, a final audit, and a report card. These reports make the run inspectable without exposing private chain-of-thought.

## Multi-Artifact Packs

The expanded v0.0.2 autonomy layer can create artifact packs for prompts such as:

```bash
cargo run -- autonomize --level full-bounded "Create a complete learning pack about brain-inspired AI for students with a 10-slide deck, speaker notes, study guide, quiz, glossary, design guide, and final report"
```

A pack records related markdown artifacts, dependencies, validation scores, assumptions, limitations, and a final report. New pack metadata is written under `sandbox/workspaces/{session_id}/artifacts/` and indexed in `data/indexes/artifact_pack_index.json`.

## Review And Repair Modes

- `review-only`: inspect the latest pack and write a quality review without modifying reviewed artifacts.
- `repair-only`: apply bounded fixes only for validation or quality issues in the latest pack.

Both modes remain sandboxed and do not add unrelated deliverables.

## Recipes And Progress

Workflow recipes provide reusable high-level plans for presentation packs, learning packs, project proposals, documentation packs, Rust projects, and benchmark reports. Progress events are written to `data/logs/progress/{session_id}.json` and printed as concise phase updates during autonomous runs.

## Example

```bash
cargo run -- autonomize --level full-bounded "Create a 10-slide presentation about brain-inspired AI for students with speaker notes and a design guide"
cargo run -- artifacts
cargo run -- artifact-inspect latest
cargo run -- session-report latest
cargo run -- artifact-packs
cargo run -- export-package latest
```

## Limitations

v0.0.2 creates export-ready markdown artifacts. It does not create binary `.pptx` files, does not browse the web, and uses citation placeholders rather than researched citations.
