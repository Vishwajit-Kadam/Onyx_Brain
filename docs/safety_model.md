# Safety Model

Onyx Brain is designed to keep local agent workflows constrained and inspectable.

## Sandbox

Generated projects are written under `sandbox/`. File tools validate paths and reject traversal attempts such as `../escape`.

## Runtime Data

Runtime state is written under `data/`, including logs, memories, sessions, journals, snapshots, transactions, and reports. Generated runtime data is ignored by default.

## Allowlisted Commands

Terminal use must remain allowlisted. The runtime should not expose arbitrary shell execution.

## Network

Onyx Brain does not use network access by default and does not include an LLM API by default.

## Bounded Autonomy

`FullBounded` autonomy means no follow-up questions inside hard safety limits. It still preserves sandboxed writes, allowlisted terminal commands, finite task and retry limits, no network by default, and no background execution.

Task graphs, queue runs, review-only mode, repair-only mode, and export-package flows are synchronous and finite. Review-only does not modify reviewed artifacts. Repair-only may only apply bounded auto-fixes to validation or quality issues.

Research-like documents use claim caution because the runtime does not verify facts over the network by default. Generated outputs may include citation placeholders and verification notes rather than fabricated sources.

## Snapshots And Rollback

Before risky modifications, the runtime can create snapshots. Transactional edits create backups and journal entries. Rollback and restore are designed to write only inside the sandbox.

## Doctor And Repair

The doctor command checks required directories, indexes, state files, sessions, journals, and other recovery metadata. Repair mode rebuilds safe indexes and archives corrupt JSON instead of deleting it permanently.

## Conversation Safety

The v0.0.3 conversation layer filters unsupported self-claims, fake citations, sandbox-bypass advice, secret extraction, malware-like requests, and destructive command suggestions. Debugger mode recommends safe commands such as `cargo fmt`, `cargo check`, and `cargo test -- --nocapture`.

Research-outline mode creates verification notes and citation placeholders. It does not perform web research by default and should not be treated as externally verified.

## GUI, Executive, And Creative Safety

The desktop GUI uses AppApi instead of direct random file mutation. It does not provide controls to disable sandboxing, command allowlists, or network-disabled-by-default behavior.

The executive self-model is consciousness-inspired terminology for state inspection and orchestration. It is not real consciousness, not sentience, and not AGI.

Creative studio outputs planning documents only. It must not claim actual film/video rendering, and prompts that reference protected franchises are converted into original inspiration with IP caution.

## Limitations

This is experimental software. Do not use it to operate sensitive systems. Review outputs, permissions, generated code, and reports before trusting results.
