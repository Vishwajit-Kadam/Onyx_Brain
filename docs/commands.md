# Commands

## Setup And Core Runtime

- `cargo run -- init`: create the disk-backed runtime layout.
- `cargo run -- think "..."`: run a single sparse cognitive task.
- `cargo run -- inspect`: show runtime state.
- `cargo run -- inspect --summary`: show a concise runtime summary.
- `cargo run -- brain-status`: show the dashboard.
- `cargo run -- brain-status --summary`: show a concise dashboard.
- `cargo run -- gui`: launch the native Rust `eframe`/`egui` desktop app.

## Conversation

- `cargo run -- chat`: start an interactive local chat loop.
- `cargo run -- chat "..."`: run one deterministic conversation turn.
- `cargo run -- modes`: list conversation modes.
- `cargo run -- mode debate "..."`: map two sides, counterarguments, common ground, and a verdict.
- `cargo run -- mode teacher "..."`: explain with examples, exercise, and recap.
- `cargo run -- mode socratic "..."`: guide with questions and hints.
- `cargo run -- mode critic "..."`: review strengths, weaknesses, risks, and improvements.
- `cargo run -- mode planner "..."`: build phases, tasks, dependencies, risks, and next action.
- `cargo run -- mode architect "..."`: outline modules, data flow, storage, safety, and tradeoffs.
- `cargo run -- mode debugger "..."`: propose safe checks and allowlisted Cargo commands.
- `cargo run -- mode research-outline "..."`: create research questions, source types, verification notes, and citation placeholders.
- `cargo run -- personality`: show the active personality profile.
- `cargo run -- personality set friendly`: persist a light wording profile.
- `cargo run -- conversation-memory`: list recent conversation summaries.
- `cargo run -- prompt-library`: list reusable prompt patterns.
- `cargo run -- transcript latest`: show the latest transcript.
- `cargo run -- transcript-export latest`: export transcript markdown and metadata.

## Project And Goal Work

- `cargo run -- project "..."`: create or modify a sandbox Rust project.
- `cargo run -- goal "..."`: create a goal memory and execute it through the project worker.
- `cargo run -- worker "..."`: run a bounded worker simulation.
- `cargo run -- autonomize "..."`: run the v0.0.2 bounded autonomous worker engine.
- `cargo run -- autonomize --level full-bounded "..."`: run without follow-up questions inside hard safety limits.
- `cargo run -- autonomize --level review-only "Review latest artifact pack"`: review the latest pack without changing reviewed artifacts.
- `cargo run -- autonomize --level repair-only "Repair latest artifact pack"`: repair validation/quality issues without creating unrelated deliverables.
- `cargo run -- auto --level full-bounded "..."`: alias for `autonomize`.
- `cargo run -- queue-run "Goal 1 || Goal 2"`: run multiple bounded goals sequentially.
- `cargo run -- autonomize --level executive "..."`: use bounded executive orchestration.
- `cargo run -- autonomize --level studio "..."`: use creative/project studio-oriented autonomy.
- `cargo run -- projects`: list registered projects.
- `cargo run -- project-inspect <name>`: inspect a registered project.
- `cargo run -- resume <goal_id>`: resume a project goal.

## Autonomy And Artifacts

- `cargo run -- autonomy-policy`: show hard autonomy limits.
- `cargo run -- artifacts`: list recent generated artifacts.
- `cargo run -- artifact-inspect latest`: inspect the latest artifact manifest and files.
- `cargo run -- artifact-packs`: list multi-artifact packs.
- `cargo run -- artifact-pack-inspect latest`: inspect the latest pack, dependency graph, and validation scores.
- `cargo run -- packs`: alias for `artifact-packs`.
- `cargo run -- pack-inspect latest`: alias for `artifact-pack-inspect latest`.
- `cargo run -- review-artifacts latest`: run quality review on an artifact pack.
- `cargo run -- repair-artifacts latest`: repair auto-fixable artifact issues.
- `cargo run -- workspaces`: list autonomous workspaces.
- `cargo run -- workspace-inspect latest`: inspect a workspace layout.
- `cargo run -- recipes`: list reusable workflow recipes.
- `cargo run -- recipe-inspect latest`: inspect a workflow recipe.
- `cargo run -- autonomy-status`: show autonomy sessions, packs, quality scores, and recommendations.
- `cargo run -- auto-status`: alias for `autonomy-status`.
- `cargo run -- export-package latest`: copy latest pack artifacts into a clean export folder.
- `cargo run -- export latest`: alias for `export-package latest`.
- `cargo run -- export-inspect latest`: inspect an exported package.
- `cargo run -- exports`: list export folders.
- `cargo run -- session-report latest`: write and show the latest session report.
- `cargo run -- report latest`: alias for `session-report latest`.
- `cargo run -- task-graph latest`: inspect the latest autonomous task graph.
- `cargo run -- reflections`: list recent autonomous reflection memories.
- `cargo run -- improve-recipes`: strengthen workflow recipes from recent usage.
- `cargo run -- capabilities`: show what the bounded runtime can and cannot do.
- `cargo run -- trace latest`: show the transparent execution trace for the latest autonomous run.
- `cargo run -- autonomy-history`: list recent autonomous sessions, grades, packs, and exports.
- `cargo run -- cleanup-autonomy`: remove only safe workspace temp files.
- `cargo run -- creative "..."`: create a creative production planning package.
- `cargo run -- self-model`: show the consciousness-inspired self-model.
- `cargo run -- attention`: show current attention state.
- `cargo run -- metacognition "..."`: write a bounded metacognitive report.
- `cargo run -- executive-status`: show recent executive decisions and safety state.

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
- `cargo run -- benchmark autonomy`: run the autonomous worker benchmark.
- `cargo run -- benchmark artifacts`: run the artifact-pack benchmark.
- `cargo run -- benchmark advanced-autonomy`: run launch-kit, technical-report, product-spec, learning-pack, export, audit, doctor, and regression checks.
- `cargo run -- benchmark conversation`: test deterministic conversation modes, quality scoring, and safety filtering.
- `cargo run -- benchmark gui-smoke`: verify GUI assets and AppApi-ready state.
- `cargo run -- benchmark creative`: create and validate a creative production package.
- `cargo run -- benchmark executive`: record an executive decision and self-model update.
- `cargo run -- benchmark compare`: compare benchmark history.
