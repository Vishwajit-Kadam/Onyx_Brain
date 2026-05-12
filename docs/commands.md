# Commands

## Setup And Core Runtime

- `cargo run -- init`: create the disk-backed runtime layout.
- `cargo run -- think "..."`: run a single sparse cognitive task.
- `cargo run -- inspect`: show runtime state.
- `cargo run -- inspect --summary`: show a concise runtime summary.
- `cargo run -- brain-status`: show the dashboard.
- `cargo run -- brain-status --summary`: show a concise dashboard.

## Project And Goal Work

- `cargo run -- project "..."`: create or modify a sandbox Rust project.
- `cargo run -- goal "..."`: create a goal memory and execute it through the project worker.
- `cargo run -- worker "..."`: run a bounded worker simulation.
- `cargo run -- projects`: list registered projects.
- `cargo run -- project-inspect <name>`: inspect a registered project.
- `cargo run -- resume <goal_id>`: resume a project goal.

## Reliability

- `cargo run -- journal`: list recent journal entries.
- `cargo run -- journal --session latest`: list recent entries for the latest session.
- `cargo run -- sessions`: list sessions.
- `cargo run -- session-start "title"`: start a session.
- `cargo run -- session-status latest`: show session status.
- `cargo run -- session-end latest`: end a session.
- `cargo run -- session-resume latest`: resume retryable pending work.
- `cargo run -- snapshots`: list snapshots.
- `cargo run -- snapshot-create <project> --reason "reason"`: create a project snapshot.
- `cargo run -- snapshot-restore <snapshot_id>`: restore a snapshot.
- `cargo run -- transactions`: list transactional edits.
- `cargo run -- rollback latest`: roll back the latest rollback-capable action.
- `cargo run -- rollback --project <name> latest`: roll back the latest action for a project.
- `cargo run -- recover latest`: print or run a safe recovery plan.
- `cargo run -- doctor`: check runtime state health.
- `cargo run -- doctor --repair`: rebuild safe indexes and archive corrupt JSON.
- `cargo run -- regression-check`: run release safety checks.

## Memory, Skills, And Optimization

- `cargo run -- memory-add --type procedural --title "..." --tags "..." --content "..."`: add a memory.
- `cargo run -- memory-inspect`: inspect memory hygiene.
- `cargo run -- memory-dedup`: archive duplicate memories.
- `cargo run -- habits`: list habits.
- `cargo run -- routes`: inspect route efficiency.
- `cargo run -- cache-inspect`: inspect plan cache.
- `cargo run -- template-cache-inspect`: inspect template cache.
- `cargo run -- optimize`: update habits/routes from profiles.
- `cargo run -- cleanup-backups`: clean old backups inside the sandbox.
- `cargo run -- maintain`: run safe maintenance.
- `cargo run -- consolidate`: consolidate memories and routes.

## Benchmarks

- `cargo run -- benchmark basic`: run a small deterministic benchmark.
- `cargo run -- benchmark reliability`: run a reliability benchmark.
- `cargo run -- benchmark compare`: compare benchmark history.
