# Onyx Brain

A disk-backed, brain-inspired cognitive runtime for sparse, safe, recoverable agent workflows.

Onyx Brain is an experimental Rust project. It explores how a small deterministic runtime can route work through disk-backed memories, skills, habits, sessions, journals, snapshots, and recovery tools while keeping the active working set small.

Onyx Brain is not AGI. It is not conscious. It is not a biological brain simulation. It does not include an LLM by default, does not access the network by default, and is not production-ready automation software.

## What Is Onyx Brain?

Onyx Brain is a cognitive runtime skeleton for safe agent-style workflows. It stores most long-lived state on disk and activates only the task-local pieces needed for a run.

It is designed around:

- Rust implementation
- disk-backed virtual neurons and synapses
- sparse routing and activation
- indexed memory
- reusable procedural skills
- live habits and plan/template caches
- sandboxed project work
- action journaling, snapshots, rollback, and recovery
- doctor and regression checks

## What It Is Not

Onyx Brain is not:

- AGI
- conscious
- a neural network framework
- an LLM wrapper
- a replacement for human code review
- a production operations agent
- a system for unrestricted shell execution

The current release is a deterministic cognitive runtime skeleton. Future releases may add optional adapters, but the default system stays local, sandboxed, and conservative.

## Core Ideas

- Sparse activation: the whole brain exists as potential; only the useful part becomes active.
- Disk-backed state: memories, goals, sessions, journals, snapshots, caches, and reports are stored as files.
- Safety first: file operations stay inside the sandbox and terminal commands are allowlisted.
- Recoverability: risky actions are journaled, transactions create backups, and snapshots can be restored.
- Learning as compression: repeated successful workflows can become reusable skills, habits, and cached plans.

## Features

- Sparse cognitive runtime with virtual neurons and synapses
- Memory hierarchy with semantic, procedural, episodic, and project memories
- Project creation and modification inside `sandbox/`
- Goal execution and bounded worker mode
- Skill extraction and reuse
- Habit formation, plan cache, and template cache
- Runtime profiling and adaptive budget reporting
- Action journal and session tracking
- Transactional file edits
- Project snapshots and rollback
- Doctor, repair, recovery plans, and regression checks
- Basic and reliability benchmark modes

## Safety Model

Onyx Brain is built to keep actions constrained:

- Generated project files are written under `sandbox/`.
- Runtime state is written under `data/`.
- File paths are checked to reject traversal outside the sandbox.
- Terminal commands are allowlisted.
- Network access is not part of the default runtime.
- Snapshots, journals, transactions, and doctor reports make state easier to inspect and recover.

This does not make Onyx Brain safe for sensitive production systems. Review tool permissions and outputs before using it with important data.

## Quick Start

```bash
cargo run -- init
cargo run -- worker "Create and improve a Rust calculator project called worker_calc"
cargo run -- doctor
cargo run -- regression-check
cargo run -- brain-status
```

The worker command creates a bounded sandbox project workflow. It should journal actions, create recoverability metadata, run safe validation when available, and write reports under `data/`.

## Commands

Common commands:

```bash
cargo run -- init
cargo run -- think "Plan a small Rust project"
cargo run -- project "Create a Rust CLI calculator project called calc_cli with tests and README"
cargo run -- goal "Create a Rust CLI calculator project called goal_calc with tests and README"
cargo run -- worker "Create and improve a Rust calculator project called worker_calc"
cargo run -- doctor
cargo run -- regression-check
cargo run -- brain-status
cargo run -- inspect --summary
```

Maintenance and reliability commands:

```bash
cargo run -- journal
cargo run -- sessions
cargo run -- snapshots
cargo run -- transactions
cargo run -- rollback latest
cargo run -- recover latest
cargo run -- benchmark reliability
cargo run -- maintain
```

See [docs/commands.md](docs/commands.md) for the full command list.

## Example Worker Task

```bash
cargo run -- init
cargo run -- worker "Create and improve a Rust calculator project called worker_calc"
```

Expected high-level behavior:

- creates a sandbox project
- decomposes work into bounded phases
- writes files through sandboxed tools
- journals actions
- creates snapshots when appropriate
- runs safe Rust validation when available
- writes final reports and reliability metadata

See [examples/calculator_worker.md](examples/calculator_worker.md).

## Project Structure

```text
src/
  agency/      project, goal, session, snapshot, rollback, worker logic
  core/        brain runtime, tasks, route traces
  energy/      budgets, profiling, route efficiency
  learning/    skills, habits, optimization, cache reuse
  memory/      disk-backed memories and indexes
  routing/     sparse route selection
  storage/     disk store, state recovery, JSON helpers
  tools/       sandboxed filesystem, code editor, diagnostics, transactions
  testing/     regression guard
docs/          public architecture, safety, and release docs
examples/      runnable examples and walkthroughs
data/          generated runtime state, ignored except placeholders
sandbox/       generated project output, ignored except placeholder
```

## Development

Run the standard checks:

```bash
cargo fmt
cargo check
cargo test -- --nocapture
```

Generated runtime data under `data/`, generated projects under `sandbox/`, and Rust build outputs under `target/` are intentionally ignored.

## Roadmap

Public versioning starts at `v0.0.1`. Internal prototype milestones `v0.1` through `v0.9` were private development history.

See [ROADMAP.md](ROADMAP.md).

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
