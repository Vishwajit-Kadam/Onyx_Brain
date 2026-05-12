# Onyx Brain v0.9

Onyx Brain is a miniature sparse cognitive operating system. It copies useful brain principles, not biological atoms: virtual neurons, synaptic routing, gated activation, memory hierarchy, expert regions, energy budgeting, plasticity, and sleep-like consolidation.

It is not AGI. It is not conscious. It is not a biological brain simulation. It is an experimental architecture for routing small tasks through a disk-backed cognitive runtime.

## Philosophy

The whole brain exists as potential. Only the useful part becomes active.

Onyx Brain keeps neurons, synapses, memories, indexes, logs, and project state on disk as JSON. A task only loads a small active working set: the task, working memory, active virtual neurons, nearby synapses, selected memories, selected experts, a tiny routing cache, and a short execution log.

## Architecture

```text
Input
  |
  v
Receptors / Classifier
  |
  v
Router Gatekeeper + Energy Budget
  |
  +--> disk-backed virtual neurons
  +--> nearby disk-backed synapses
  +--> indexed disk-backed memories
  |
  v
Sparse Expert Regions + Expert Confidence
  |
  +--> LanguageExpert
  +--> CodeExpert
  +--> ReasoningExpert
  +--> ToolUseExpert
  |
  v
Safe Agency + Tools
  |
  +--> sandbox filesystem
  +--> allowlisted terminal commands
  +--> Rust project creator
  |
  v
Self-Review + Route Trace + Learning + Consolidation
```

## v0.2 Features

- Indexed memory search with `data/indexes/memory_tags.json` and `data/indexes/memory_keywords.json`.
- Weighted memory scoring using keyword overlap, tag overlap, task type fit, importance, linked active neurons, recency, and access count.
- Procedural memories are preferred for code and file-operation tasks when relevant.
- Route traces are saved for every `think` run.
- Expert confidence stats are tracked in `data/indexes/expert_stats.json`.
- `inspect` now reports top synapses, used neurons, important memories, recent tasks, average energy, and consolidation state.
- Checkpoints include the current goal, planned steps, completed steps, failed steps, and final status.
- Brain output includes a deterministic self-review.

## v0.3 Features

Onyx Brain v0.3 adds a miniature autonomous project worker. It can take a larger project goal, decompose it into deterministic subtasks, execute those tasks inside the sandbox, checkpoint after every task, run diagnostics, retry simple safe fixes, and write a final project report.

New project-worker pieces:

- `agency/task_queue.rs`: disk-backed task queue stored at `data/projects/{goal_id}/task_queue.json`
- `agency/decomposer.rs`: rule-based project goal decomposition
- `agency/project_state.rs`: disk-backed project state stored at `data/projects/{goal_id}/project_state.json`
- `agency/retry.rs`: bounded safe retry logic
- `tools/code_editor.rs`: sandbox-only UTF-8 code editing
- `tools/diagnostics.rs`: simple Cargo output diagnostics
- `cargo run -- project ...`: autonomous project execution command

The worker is still deliberately small. It does not call an LLM, does not use a GPU, does not access the network, and does not execute unrestricted shell commands.

## v0.4 Features

Onyx Brain v0.4 upgrades the project worker into a self-improving coding worker for sandbox Rust projects. It can discover existing projects, modify them later, run checks/tests, diagnose common Rust failures, retry bounded safe fixes, evaluate its own result, and extract reusable procedural skills.

New v0.4 pieces:

- Project registry at `data/projects/project_registry.json`
- Goal parser with `CreateProject`, `ModifyProject`, `InspectProject`, `ResumeProject`, and `Unknown`
- Feature extraction for calculator operations, tests, README, CLI, modules, and functions
- Existing project modification through `cargo run -- project "..."`
- `cargo run -- projects` to list registered projects
- `cargo run -- project-inspect <name>` to inspect a known project
- `cargo run -- resume <goal_id|latest>` to continue queued work
- Safer code editor helpers with timestamped backups
- Improved diagnostics for missing functions, modules, type mismatches, test failures, dependency errors, and divide-by-zero risk
- Self-evaluation scores for correctness, test coverage, completeness, energy efficiency, and overall result
- Procedural skill extraction after successful work

## v0.5 Features

Onyx Brain v0.5 adds active skill reuse, memory hygiene, and benchmark reporting.

New v0.5 pieces:

- `learning/skill_reuse.rs`: finds relevant procedural skills and tracks reuse success.
- `memory/dedup.rs`: archives duplicate project memories and procedural skills.
- `cargo run -- memory-inspect`: shows memory counts, duplicates, reusable skills, stale memories, and index size.
- `cargo run -- memory-dedup`: archives duplicate memories and writes a report.
- `cargo run -- benchmark basic`: runs a deterministic project-create/modify/inspect/check/test/consolidate/dedup benchmark.
- Project outputs list reused skills.
- Route traces can record reused skills and skill application results.
- Self-evaluation includes skill reuse and memory hygiene scores.
- `inspect` reports memory hygiene warnings and suggests `memory-dedup` when needed.

Skill reuse prefers procedural memories tagged with `rust`, `project`, `skill`, `workflow`, `tests`, `readme`, `calculator`, or `cli`.

## v0.6 Features

Onyx Brain v0.6 adds goal memory, priority scheduling, automatic maintenance, benchmark comparison, and a brain-status dashboard.

New v0.6 pieces:

- `agency/goal_memory.rs`: disk-backed goal memories in `data/goals/{goal_id}.json`
- `data/indexes/goal_index.json`: lightweight goal index
- `energy/priority_scheduler.rs`: rule-based priority scoring
- `cargo run -- goal "..."`
- `cargo run -- goals`
- `cargo run -- brain-status`
- `cargo run -- cleanup-backups`
- `cargo run -- benchmark compare`
- `cargo run -- maintain`
- automatic lightweight dedup after successful project runs
- benchmark history at `data/indexes/benchmark_history.json`
- clearer separation between registered projects and historical project memories
- selective skill reuse that prefers generic skills over unrelated project workflows
- more realistic self-evaluation scoring

## v0.7 Features

Onyx Brain v0.7 adds adaptive efficiency. Repeated successful work can now be profiled, summarized into reusable habits, cached as successful plans, and used to reduce budget fan-out on familiar tasks while preserving safety checks.

New v0.7 pieces:

- `energy/performance_profiler.rs`: saves performance profiles in `data/logs/performance_profiles/` and summaries in `data/indexes/performance_index.json`
- `routing/route_efficiency.rs`: tracks route success, runtime, energy, and efficiency in `data/indexes/route_efficiency.json`
- `energy::AdaptiveBudgetManager`: reduces or expands task budgets conservatively based on habit/cache/failure signals
- `learning/habits.rs`: forms and reuses disk-backed habits in `data/habits/`
- `agency/plan_cache.rs`: caches successful plan signatures in `data/cache/plans/`
- `cargo run -- optimize`: analyzes profiles/routes/skills and writes an optimization report
- `cargo run -- habits`: lists formed habits and confidence
- `cargo run -- routes`: shows efficient, weak, and failure-prone routes
- `cargo run -- cache-inspect`: shows cached plans, hit rate, and estimated runtime saved
- Benchmark reports now include irrelevant skills, habits, cache hits, adaptive budget decisions, and route efficiency
- Self-evaluation now includes habit reuse, plan cache, route efficiency, and irrelevant skill penalty signals

Adaptive efficiency stays rule-based. It does not skip Cargo validation, sandbox checks, retry limits, or terminal allowlists.

## v0.8 Features

Onyx Brain v0.8 adds live habits, runtime-aware reporting, safe fast paths, and a project template cache. The goal is to explain and reduce repeated-task overhead while preserving validation and sandbox guarantees.

New v0.8 pieces:

- `learning/live_habits.rs`: strengthens or creates matching habits after successful project/goal work.
- `energy/performance_profiler.rs`: performance profiles now include runtime breakdowns for brain, tools, Cargo, filesystem, reporting, and maintenance.
- `agency/fast_path.rs`: uses high-confidence habit/cache matches to shorten redundant planning and scanning without skipping safety, state, trace, report, or Cargo validation.
- `agency/template_cache.rs`: stores deterministic Rust project templates in `data/cache/templates/` with an index at `data/indexes/template_cache_index.json`.
- `tools/cargo_policy.rs`: explains when `cargo check` and `cargo test` are required, and allows README-only changes to skip Cargo safely.
- `utils/environment.rs`: warns when the workspace path looks like OneDrive/cloud sync, contains spaces, or is unusually long.
- `learning/auto_optimize.rs`: gives optimization hints after profile buildup, missed habits, duplicate memories, or irrelevant skill reuse.
- `cargo run -- template-cache-inspect`: shows cached templates.
- `cargo run -- inspect --summary`: prints a concise state summary.
- `cargo run -- brain-status --summary`: prints a compact dashboard.

Benchmark reports now include template cache hits and runtime diagnosis, with compare output distinguishing brain improvements from tool/Cargo-bound total runtime. Project and goal outputs include live habit updates, fast-path decisions, Cargo validation policy, runtime breakdown, template/cache usage, adaptive budget, and optimization hints.

v0.8 remains deterministic and local. Fast paths never bypass sandbox path checks, state saves, route traces, final reports, or Cargo validation for Rust code edits.

## v0.9 Features

Onyx Brain v0.9 adds reliability infrastructure for long-running autonomous work. Every risky action should become recoverable, inspectable, and resumable while staying disk-backed and sandboxed.

New v0.9 pieces:

- `agency/action_journal.rs`: disk-backed journal entries in `data/journal/` with a lightweight index.
- `agency/snapshot.rs`: project snapshots in `data/snapshots/` for safe restore.
- `agency/rollback.rs`: conservative rollback from rollback-capable journal entries.
- `tools/transactional_edit.rs`: atomic-ish file edits with backups, temp files, transaction logs, and journal entries.
- `storage/state_recovery.rs`: `doctor` and `doctor --repair` for missing/corrupt state and safe archival.
- `agency/session.rs`: resumable work sessions in `data/sessions/`.
- `agency/recovery.rs`: deterministic recovery plans for failed tasks.
- `agency/reliability.rs`: reliability scoring for reports and dashboards.
- `agency/worker_mode.rs`: bounded worker-mode simulation.
- `testing/regression_guard.rs`: internal `regression-check` command.

New commands:

- `cargo run -- journal`
- `cargo run -- journal --session latest`
- `cargo run -- snapshots`
- `cargo run -- snapshot-create <project> --reason "manual checkpoint"`
- `cargo run -- snapshot-restore <snapshot_id>`
- `cargo run -- rollback latest`
- `cargo run -- rollback --project <project> latest`
- `cargo run -- transactions`
- `cargo run -- doctor`
- `cargo run -- doctor --repair`
- `cargo run -- recover latest`
- `cargo run -- recover --project <project> latest`
- `cargo run -- sessions`
- `cargo run -- session-start "Build calculator improvements"`
- `cargo run -- session-status latest`
- `cargo run -- session-end latest`
- `cargo run -- session-resume latest`
- `cargo run -- worker "Create and improve a Rust calculator project called worker_calc"`
- `cargo run -- regression-check`
- `cargo run -- benchmark reliability`

Project reports now also write `final_report.json` beside `final_report.md`, and include session id, journal/snapshot summaries, rollback readiness, reliability score, recovery plan, runtime breakdown, fast path, cache/habit/template usage, and Cargo policy.

## Run

```bash
cargo run -- init
cargo run -- memory-add --type procedural --title "Create Rust Project" --tags "rust,cargo,project" --content "Create Cargo.toml, create src/main.rs, run cargo check."
cargo run -- think "Create a Rust CLI project called hello_cli"
cargo run -- inspect
cargo run -- consolidate
```

Run an autonomous project:

```bash
cargo run -- project "Create a Rust CLI calculator project called calc_cli with add and subtract functions, tests, and README"
cargo run -- project "Modify the calc_cli project to add multiply and divide functions with tests"
cargo run -- project-inspect calc_cli
cargo run -- projects
cargo run -- resume latest
cargo run -- memory-inspect
cargo run -- memory-dedup
cargo run -- benchmark basic
cargo run -- benchmark compare
cargo run -- goal "Create a Rust CLI calculator project called goal_calc with tests and README"
cargo run -- goals
cargo run -- brain-status
cargo run -- brain-status --summary
cargo run -- optimize
cargo run -- habits
cargo run -- routes
cargo run -- cache-inspect
cargo run -- template-cache-inspect
cargo run -- journal
cargo run -- snapshots
cargo run -- transactions
cargo run -- doctor
cargo run -- regression-check
cargo run -- benchmark reliability
cargo run -- worker "Create and improve a Rust calculator project called worker_calc"
cargo run -- cleanup-backups
cargo run -- maintain
cargo run -- inspect --summary
```

Add semantic memory manually:

```bash
cargo run -- memory-add --type semantic --title "Rust Ownership" --tags "rust,ownership,memory" --content "Ownership is Rust's system for memory safety without garbage collection."
```

## Memory Scoring

Memory candidates are found from the tag and keyword indexes before loading memory files. Each candidate is scored with:

```text
keyword_overlap * 0.35
+ tag_overlap * 0.20
+ task_type_match * 0.15
+ importance * 0.15
+ linked_neuron_bonus * 0.10
+ recency_bonus * 0.03
+ access_count_bonus * 0.02
```

Only the top memories allowed by the energy budget are returned. Accessed memories are updated on disk with `last_accessed_at` and `access_count`.

## Inspect

`cargo run -- inspect` prints:

- number of neurons, synapses, memories, and logs on disk
- top 10 strongest synapses
- top 10 most used neurons
- top 10 most important memories
- last 5 route-traced tasks
- average energy estimate
- last consolidation time
- registered project count
- last modified project
- top extracted procedural skills
- average project self-evaluation score
- failed/blocked task count
- memory hygiene warnings
- route efficiency top routes
- habit summary
- plan cache summary
- template cache summary through `template-cache-inspect`
- slowest recent command types
- runtime breakdown and adaptive budget summaries
- adaptive budget summary
- sandbox path

## Route Trace

Every task log is a `RouteTrace` containing task input, task type, activated neurons, activated synapses, selected experts, selected memories, tool actions, success/failure, energy estimate, runtime, and learning updates.

## Procedural Memory Example

```bash
cargo run -- memory-add --type procedural --title "Create Rust Project" --tags "rust,cargo,project" --content "Steps: create Cargo.toml, create src/main.rs, run cargo check."
cargo run -- think "Create a Rust CLI project called hello_cli"
```

For code tasks, procedural memories about Rust project creation score higher than equally similar semantic memories because they match the task type.

## Autonomous Project Worker

The project worker follows a deterministic loop:

1. Create a goal id.
2. Extract or generate a project name.
3. Decompose the prompt into a queue of subtasks.
4. Save the task queue and project state to disk.
5. Execute each task using sandbox-safe tools.
6. Save state after every task.
7. Run `cargo check` and `cargo test` through the terminal allowlist.
8. Parse command output with diagnostics.
9. Retry simple safe fixes up to the task attempt limit.
10. Save project memory and a final report.

For code project goals, decomposition includes understanding the goal, creating directories, writing `Cargo.toml`, writing `src/main.rs`, writing `src/lib.rs`, writing tests and README when requested, running Cargo checks, inspecting the result, and creating a final report.

## Diagnostics And Retries

Diagnostics classify command output as:

- `CargoCheckPassed`
- `CargoTestPassed`
- `SyntaxError`
- `MissingFunction`
- `MissingFile`
- `MissingModule`
- `TypeMismatch`
- `TestFailure`
- `DependencyError`
- `DivideByZeroRisk`
- `UnknownError`

Retry logic is intentionally bounded. Each queued task has `max_attempts`, defaulting to 2. The retry path only applies simple template repairs through `CodeEditorTool` and only inside the sandbox project.

## Self-Evaluation

Project outputs include:

- `correctness_score`
- `test_coverage_score`
- `completeness_score`
- `energy_efficiency_score`
- `skill_reuse_score`
- `memory_hygiene_score`
- `habit_reuse_score`
- `plan_cache_score`
- `route_efficiency_score`
- `irrelevant_skill_penalty`
- `overall_score`

Scores are deterministic. Passing `cargo check` and `cargo test` boosts correctness, requested functions in code boost completeness, requested README updates boost completeness, and low retry counts boost energy efficiency. Habit/cache/route signals improve scores when they are relevant and reduce scores when unrelated project workflows are reused unnecessarily.

## Skill Extraction

Successful project work creates procedural skill memories with tags such as `rust`, `project`, `skill`, and `workflow`. Examples include:

- Create Rust CLI project
- Add calculator operation to Rust library
- Add Rust unit tests
- Update README for Rust project
- Run cargo check and cargo test

Duplicate skill titles are skipped so repeated successful runs strengthen the architecture without flooding memory.

## Memory Hygiene

`memory-inspect` reports total memory counts, archived memories, duplicate groups, top reusable skills, stale memories, and index size. `memory-dedup` preserves the newest/most important memory in each duplicate group and moves duplicates to `data/memories/archive/`; it never permanently deletes them.

Consolidation calls deduplication internally and can create shortcut synapses for repeated skill workflows.

## Benchmark Mode

`cargo run -- benchmark basic` runs a deterministic smoke suite: create `bench_calc`, modify it, inspect it, run `cargo check` and `cargo test`, consolidate, and deduplicate memory. Reports are saved as `data/logs/benchmark_basic_{timestamp}.json`.

`cargo run -- benchmark compare` reads `data/indexes/benchmark_history.json` and prints the last score, best score, average score, runtime trend, energy trend, skill reuse quality trend, habit usage trend, cache hit rate trend, route efficiency trend, and memory hygiene trend. v0.8 compare can explain cases like `brain improving, total runtime tool-bound` or `cache improved but cargo time increased`. If history is too short, trends report `insufficient history`.

## Goal Memory

`cargo run -- goal "..."`

This creates a goal memory, parses intent, schedules and executes the goal through the project worker, links the resulting project and reused skills, then updates the goal as completed, failed, or blocked.

`cargo run -- goals` lists goals by priority and status.

## Brain Status

`cargo run -- brain-status` prints a compact dashboard:

- version
- neuron and synapse counts
- registered projects
- historical project memories
- active/completed/blocked goals
- memories by type
- duplicate memory groups
- top skills by reuse
- latest benchmark score
- average project self-evaluation
- memory hygiene score
- recommended maintenance actions
- performance profile count and average runtime
- average route efficiency
- habits and top habit summaries
- plan cache entries and hit rate
- adaptive budget estimated savings
- optimization recommendations
- environment overhead notes, including OneDrive/cloud-sync warnings
- average brain/tool/Cargo runtime for recent profiles
- reliability summary with journal, snapshot, session, doctor, rollback, and recovery signals

## Maintenance

`cargo run -- maintain` runs memory deduplication, backup cleanup, consolidation, and benchmark comparison summary. Backup cleanup only touches `.bak.` files under the sandbox and keeps the latest three backups per source file.

`cargo run -- optimize` reads performance profiles and route traces, forms or strengthens habits from repeated successes, updates route efficiency, lightly penalizes unrelated old project workflows, and writes `data/logs/optimization_report_{timestamp}.json`. It does not edit project files.

Normal project and goal runs only print optimization hints. `cargo run -- maintain` runs the mutating maintenance path and refreshes lightweight optimization signals.

## Safety Limitations

Tools are intentionally narrow:

- Filesystem operations are sandbox-only.
- Paths reject traversal such as `..`.
- Terminal commands are allowlisted only: `cargo --version`, `rustc --version`, `cargo check`, and `cargo test`.
- Commands run without shell expansion.
- There is no network access in the runtime.
- There are no LLM API calls, GPU dependencies, neural-network crates, database servers, or unsafe Rust.
- Inspect still reads small JSON summaries from disk for top lists; v0.8 is not a database engine.
- Project execution is template-driven and rule-based; it is not a general software engineer.
- Adaptive budgets are conservative hints, not permission to skip safety checks.
- Habits and plan cache are deterministic workflow compression, not learned code synthesis.
- Template cache is limited to deterministic Rust CLI/library/README templates.
- Runtime breakdowns are approximate timers intended for diagnosis, not precise profiling.
- Rollback is conservative and refuses unsafe paths.
- Corrupt JSON is archived before replacement; v0.9 does not permanently delete corrupt state.
- Worker mode is a bounded synchronous simulation, not a daemon or background process.
- Retries are finite and conservative.
- Modification support is currently strongest for simple Rust calculator-style projects.
- No LLM/API/GPU/network support exists yet.
- Benchmarks are deterministic smoke tests, not scientific cognitive evaluations.
- Goal scheduling is rule-based and local; it is not a full planner.

## Memory Model

- Working memory: task-local and RAM-only.
- Episodic memory: past task summaries on disk.
- Semantic memory: facts and concepts on disk.
- Procedural memory: workflows and skills on disk.
- Project memory: checkpoints and long-running task progress on disk.

## Roadmap

- More compact summary indexes for inspect.
- Better duplicate-memory merging during consolidation.
- Configurable energy budgets.
- More safe tools with explicit permissions.
- Optional durable project histories.
- Larger expert library while preserving sparse activation.
