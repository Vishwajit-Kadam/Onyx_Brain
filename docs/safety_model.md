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

## Snapshots And Rollback

Before risky modifications, the runtime can create snapshots. Transactional edits create backups and journal entries. Rollback and restore are designed to write only inside the sandbox.

## Doctor And Repair

The doctor command checks required directories, indexes, state files, sessions, journals, and other recovery metadata. Repair mode rebuilds safe indexes and archives corrupt JSON instead of deleting it permanently.

## Limitations

This is experimental software. Do not use it to operate sensitive systems. Review outputs, permissions, generated code, and reports before trusting results.
