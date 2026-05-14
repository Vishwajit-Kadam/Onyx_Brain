# Architecture

Onyx Brain is a Rust-based, disk-backed, brain-inspired cognitive runtime. It is not a biological simulation. It borrows a few useful engineering ideas from brains: sparse activation, memory hierarchy, route strengthening, habit formation, and recovery after failure.

## Runtime Flow

```text
input
  -> classifier
  -> router and energy budget
  -> active virtual neurons and nearby synapses
  -> indexed memories and reusable skills
  -> deterministic experts and sandboxed tools
  -> route trace, journal, reports, learning updates
```

## Virtual Neurons And Synapses

Neurons and synapses are stored as JSON on disk. A command should only load the small set needed for the current task. Synapse strength and route efficiency records help future tasks reuse successful paths.

## Routing

Routing is deterministic and budgeted. The router activates a bounded working set based on task type, memory matches, skills, habits, cached plans, and route efficiency. It does not load the whole graph into RAM.

## Memory Hierarchy

Onyx Brain stores semantic, episodic, procedural, and project memories under `data/`. Lightweight indexes help retrieval avoid scanning every memory file.

## Skills, Habits, And Cache

Successful workflows can become procedural skills. Repeated successful patterns can strengthen habits. Plan and template caches allow familiar Rust project tasks to reuse deterministic steps while preserving safety checks.

## Reliability Layer

The public release includes:

- action journal
- sessions
- transactional edits
- snapshots
- rollback
- recovery plans
- doctor and repair checks
- regression checks
- reliability benchmark

These systems make risky actions easier to inspect and recover. They do not make the runtime production-hardened.

## Worker Mode

Worker mode is bounded and synchronous. It runs a small fixed number of phases and tasks, uses allowlisted tools, and writes output under `sandbox/` and `data/`.

## Autonomous Worker Engine

The `autonomize` command adds a bounded autonomous worker path. It uses rule-based goal understanding, phase planning, artifact generation, validation, repair, journaling, and session reporting. It completes inside the current command; there is no daemon or hidden background loop.

Artifacts are written under `sandbox/artifacts/{session_id}/`. Session reports are written under `data/sessions/{session_id}/`. The engine is deterministic by default and does not use an LLM or network access.

## Expanded Autonomy Layer

The v0.0.2 expansion adds dependency-aware task graphs, a bounded scheduler, multi-artifact packs, workspaces, workflow recipes, report cards, self-question logs, consistency checks, claim caution notes, and reflection memories.

## Conversational Layer

v0.0.3 adds a deterministic conversation layer. It stores active sessions in `data/conversations/`, summaries in `data/conversation_memory/`, profile settings in `data/config/`, and exported transcripts in `sandbox/exports/conversations/`.

The conversation layer is mode based rather than model based. Intent detection, topic extraction, response templates, response quality scoring, and safety filters produce structured answers for teaching, debate, planning, critique, debugging, and research outlines. It does not include an LLM by default and does not claim real understanding.

## Desktop And Executive Layer

v0.0.4 adds a native `eframe`/`egui` desktop GUI shell. GUI actions go through `AppApi`, which wraps existing Brain methods and keeps mutation inside the same sandbox, allowlist, journal, doctor, and regression systems.

The executive layer is consciousness-inspired but not conscious. It tracks a self-model, attention state, metacognitive reports, and executive decisions to make bounded workflow orchestration more inspectable.

Creative production planning is handled as deterministic markdown/package generation under `sandbox/workspaces/`. It creates production plans, timelines, shot lists, and reports; it does not render video.

The release-kit, documentation-pack, learning-pack, and queue-run paths reuse the same safety boundaries: no network by default, no unrestricted shell, no background worker, and sandbox/workspace-only artifact writes.

## Disk-Backed Design

Generated runtime state lives under `data/`. Generated project output lives under `sandbox/`. Both are ignored by default except placeholder files.
