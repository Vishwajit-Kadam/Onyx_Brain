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

## Disk-Backed Design

Generated runtime state lives under `data/`. Generated project output lives under `sandbox/`. Both are ignored by default except placeholder files.
