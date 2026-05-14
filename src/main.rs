use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use onyx_brain::{
    agency::AutonomyLevel,
    conversation::{ConversationMode, PersonalityProfile},
    memory::MemoryType,
    Brain, ONYX_VERSION,
};

#[derive(Debug, Parser)]
#[command(name = "onyx_brain", version, about = "Miniature sparse cognitive OS")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    Gui,
    Think {
        input: String,
    },
    Project {
        input: String,
    },
    Goal {
        input: String,
    },
    Goals,
    Projects,
    ProjectInspect {
        project_name: String,
    },
    Resume {
        goal_id: String,
    },
    MemoryInspect,
    MemoryDedup,
    Benchmark {
        name: String,
    },
    Optimize,
    Habits,
    Routes,
    CacheInspect,
    TemplateCacheInspect,
    Autonomize {
        #[arg(long, value_enum, default_value_t = AutonomyLevelArg::Standard)]
        level: AutonomyLevelArg,
        input: String,
    },
    Auto {
        #[arg(long, value_enum, default_value_t = AutonomyLevelArg::Standard)]
        level: AutonomyLevelArg,
        input: String,
    },
    AutonomyPolicy,
    Artifacts,
    ArtifactInspect {
        selector: String,
    },
    ArtifactPacks,
    Packs,
    ArtifactPackInspect {
        selector: String,
    },
    PackInspect {
        selector: String,
    },
    ReviewArtifacts {
        selector: String,
    },
    RepairArtifacts {
        selector: String,
    },
    Workspaces,
    WorkspaceInspect {
        selector: String,
    },
    Recipes,
    RecipeInspect {
        selector: String,
    },
    AutonomyStatus,
    AutoStatus,
    ExportPackage {
        selector: String,
    },
    Export {
        selector: String,
    },
    ExportInspect {
        selector: String,
    },
    Exports,
    SessionReport {
        selector: String,
    },
    Report {
        selector: String,
    },
    TaskGraph {
        selector: String,
    },
    Reflections,
    ImproveRecipes,
    Capabilities,
    Trace {
        selector: String,
    },
    AutonomyHistory,
    CleanupAutonomy,
    QueueRun {
        input: String,
    },
    Creative {
        input: String,
    },
    SelfModel,
    Attention,
    Metacognition {
        input: String,
    },
    ExecutiveStatus,
    Chat {
        input: Option<String>,
    },
    Modes,
    Mode {
        #[arg(value_enum)]
        mode: ConversationModeArg,
        #[arg(long)]
        show_quality: bool,
        input: String,
    },
    Personality {
        #[command(subcommand)]
        command: Option<PersonalityCommand>,
    },
    ConversationMemory,
    PromptLibrary,
    Transcript {
        selector: String,
    },
    TranscriptExport {
        selector: String,
    },
    Journal {
        #[arg(long)]
        session: Option<String>,
    },
    Snapshots,
    SnapshotCreate {
        project_name: String,
        #[arg(long)]
        reason: String,
    },
    SnapshotRestore {
        snapshot_id: String,
    },
    Rollback {
        #[arg(long)]
        project: Option<String>,
        selector: String,
    },
    Transactions,
    Doctor {
        #[arg(long)]
        repair: bool,
    },
    Recover {
        #[arg(long)]
        project: Option<String>,
        selector: String,
    },
    Sessions,
    SessionStart {
        title: String,
    },
    SessionStatus {
        selector: String,
    },
    SessionEnd {
        selector: String,
    },
    SessionResume {
        selector: String,
    },
    Worker {
        input: String,
    },
    RegressionCheck,
    CleanupBackups,
    BrainStatus {
        #[arg(long)]
        summary: bool,
    },
    Maintain,
    Consolidate,
    Inspect {
        #[arg(long)]
        summary: bool,
    },
    MemoryAdd {
        #[arg(long = "type", value_enum)]
        memory_type: MemoryTypeArg,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        tags: String,
        #[arg(long)]
        content: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum MemoryTypeArg {
    Working,
    Episodic,
    Semantic,
    Procedural,
    Project,
}

#[derive(Debug, Clone, ValueEnum)]
enum AutonomyLevelArg {
    Assisted,
    Standard,
    High,
    FullBounded,
    ReviewOnly,
    RepairOnly,
    Studio,
    Executive,
}

#[derive(Debug, Clone, ValueEnum)]
enum ConversationModeArg {
    Standard,
    Debate,
    Teacher,
    Socratic,
    Critic,
    Planner,
    Architect,
    Debugger,
    ResearchOutline,
    Creative,
    Summarizer,
    SafetyReview,
    ProductManager,
    Coach,
}

impl From<ConversationModeArg> for ConversationMode {
    fn from(value: ConversationModeArg) -> Self {
        match value {
            ConversationModeArg::Standard => ConversationMode::Standard,
            ConversationModeArg::Debate => ConversationMode::Debate,
            ConversationModeArg::Teacher => ConversationMode::Teacher,
            ConversationModeArg::Socratic => ConversationMode::Socratic,
            ConversationModeArg::Critic => ConversationMode::Critic,
            ConversationModeArg::Planner => ConversationMode::Planner,
            ConversationModeArg::Architect => ConversationMode::Architect,
            ConversationModeArg::Debugger => ConversationMode::Debugger,
            ConversationModeArg::ResearchOutline => ConversationMode::ResearchOutline,
            ConversationModeArg::Creative => ConversationMode::Creative,
            ConversationModeArg::Summarizer => ConversationMode::Summarizer,
            ConversationModeArg::SafetyReview => ConversationMode::SafetyReview,
            ConversationModeArg::ProductManager => ConversationMode::ProductManager,
            ConversationModeArg::Coach => ConversationMode::Coach,
        }
    }
}

#[derive(Debug, Subcommand)]
enum PersonalityCommand {
    Set {
        #[arg(value_enum)]
        profile: PersonalityProfileArg,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum PersonalityProfileArg {
    Balanced,
    Friendly,
    Technical,
    Concise,
    Mentor,
    DebateCoach,
    Productive,
}

impl From<PersonalityProfileArg> for PersonalityProfile {
    fn from(value: PersonalityProfileArg) -> Self {
        match value {
            PersonalityProfileArg::Balanced => PersonalityProfile::Balanced,
            PersonalityProfileArg::Friendly => PersonalityProfile::Friendly,
            PersonalityProfileArg::Technical => PersonalityProfile::Technical,
            PersonalityProfileArg::Concise => PersonalityProfile::Concise,
            PersonalityProfileArg::Mentor => PersonalityProfile::Mentor,
            PersonalityProfileArg::DebateCoach => PersonalityProfile::DebateCoach,
            PersonalityProfileArg::Productive => PersonalityProfile::Productive,
        }
    }
}

impl From<AutonomyLevelArg> for AutonomyLevel {
    fn from(value: AutonomyLevelArg) -> Self {
        match value {
            AutonomyLevelArg::Assisted => AutonomyLevel::Assisted,
            AutonomyLevelArg::Standard => AutonomyLevel::Standard,
            AutonomyLevelArg::High => AutonomyLevel::High,
            AutonomyLevelArg::FullBounded => AutonomyLevel::FullBounded,
            AutonomyLevelArg::ReviewOnly => AutonomyLevel::ReviewOnly,
            AutonomyLevelArg::RepairOnly => AutonomyLevel::RepairOnly,
            AutonomyLevelArg::Studio => AutonomyLevel::Studio,
            AutonomyLevelArg::Executive => AutonomyLevel::Executive,
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();
    if cli.command.is_none() {
        onyx_brain::gui::run_native_gui()?;
        return Ok(());
    }
    let root = std::env::current_dir()?;
    let brain = Brain::new(root);
    let command = cli.command.unwrap_or(Command::Gui);
    match command {
        Command::Init => {
            brain.init()?;
            println!("Onyx Brain {ONYX_VERSION} initialized");
        }
        Command::Gui => {
            onyx_brain::gui::run_native_gui()?;
        }
        Command::Think { input } => {
            let output = brain.think(input)?;
            println!("Onyx Brain {ONYX_VERSION}");
            println!("Task: {}", output.task);
            println!("Task Type: {:?}", output.task_type);
            println!(
                "Activated neurons: {} / budget {}",
                output.activated_neurons.len(),
                match output.task_type {
                    onyx_brain::core::TaskType::Chat => 16,
                    onyx_brain::core::TaskType::Planning => 24,
                    onyx_brain::core::TaskType::Unknown => 12,
                    _ => 32,
                }
            );
            println!(
                "Loaded synapses: {}",
                output.energy_report.loaded_synapse_count
            );
            println!("Memories used: {}", output.used_memories.len());
            println!("Experts used: {}", output.activated_experts.join(", "));
            println!("Tools used: {}", output.tool_actions.join("; "));
            println!("Result: {}", output.result);
            println!(
                "Self-review: {}",
                if output.self_review.success {
                    "pass"
                } else {
                    "review needed"
                }
            );
            println!(
                "Energy estimate: {} units",
                output.energy_report.estimated_cost_units
            );
            println!("Summary: {}", output.answer);
            println!(
                "RAM-minimal note: active state was task-local; durable neurons, synapses, memories, and logs stayed disk-backed."
            );
        }
        Command::Project { input } => {
            let output = brain.run_project(input)?;
            println!("Onyx Brain {ONYX_VERSION} project worker");
            println!("Intent: {:?}", output.intent);
            println!("Project name: {}", output.project_name);
            println!(
                "Features requested: {}",
                output.features_requested.join(", ")
            );
            println!("Tasks completed: {}", output.tasks_completed);
            println!("Tasks failed: {}", output.tasks_failed);
            println!("Files created: {}", output.files_created.len());
            println!("Files modified: {}", output.files_modified.len());
            println!("Cargo check result: {}", output.cargo_check_result);
            println!("Cargo test result: {}", output.cargo_test_result);
            println!("Retries used: {}", output.retries_used);
            println!("Self-evaluation:");
            println!(
                "  correctness_score: {:.2}",
                output.self_evaluation.correctness_score
            );
            println!(
                "  test_coverage_score: {:.2}",
                output.self_evaluation.test_coverage_score
            );
            println!(
                "  completeness_score: {:.2}",
                output.self_evaluation.completeness_score
            );
            println!(
                "  energy_efficiency_score: {:.2}",
                output.self_evaluation.energy_efficiency_score
            );
            println!(
                "  skill_reuse_score: {:.2}",
                output.self_evaluation.skill_reuse_score
            );
            println!(
                "  memory_hygiene_score: {:.2}",
                output.self_evaluation.memory_hygiene_score
            );
            println!(
                "  habit_reuse_score: {:.2}",
                output.self_evaluation.habit_reuse_score
            );
            println!(
                "  plan_cache_score: {:.2}",
                output.self_evaluation.plan_cache_score
            );
            println!(
                "  route_efficiency_score: {:.2}",
                output.self_evaluation.route_efficiency_score
            );
            println!(
                "  irrelevant_skill_penalty: {:.2}",
                output.self_evaluation.irrelevant_skill_penalty
            );
            println!(
                "  overall_score: {:.2}",
                output.self_evaluation.overall_score
            );
            println!("Skills reused:");
            for skill in &output.reused_skills {
                println!("  - {}", skill.title);
            }
            if output.habits_used.is_empty() {
                println!("Habits used: none");
            } else {
                println!("Habits used:");
                for habit in &output.habits_used {
                    println!("  - {}", habit.title);
                }
            }
            println!(
                "Plan cache: {}",
                output
                    .plan_cache_match
                    .as_ref()
                    .map(|cache| format!("hit {} ({:.2})", cache.cache_id, cache.similarity_score))
                    .unwrap_or_else(|| "miss".to_string())
            );
            println!(
                "Template cache: {}",
                output
                    .template_cache_used
                    .clone()
                    .unwrap_or_else(|| "not used".to_string())
            );
            println!(
                "Fast path: {}",
                if output.fast_path_decision.used_fast_path {
                    "used"
                } else {
                    "not used"
                }
            );
            println!("Reason: {}", output.fast_path_decision.reason);
            println!(
                "Preserved safety steps: {}",
                output.fast_path_decision.preserved_steps.join(", ")
            );
            println!("Cargo validation policy:");
            println!(
                "  cargo check: {}",
                if output.cargo_validation_policy.run_cargo_check {
                    "yes"
                } else {
                    "no"
                }
            );
            println!(
                "  cargo test: {}",
                if output.cargo_validation_policy.run_cargo_test {
                    "yes"
                } else {
                    "no"
                }
            );
            println!("  reason: {}", output.cargo_validation_policy.reason);
            println!("Runtime breakdown:");
            println!("  brain: {} ms", output.runtime_breakdown.brain_runtime_ms);
            println!("  tools: {} ms", output.runtime_breakdown.tool_runtime_ms);
            println!(
                "  cargo: {} ms",
                output.runtime_breakdown.cargo_runtime_ms()
            );
            println!(
                "  filesystem: {} ms",
                output.runtime_breakdown.filesystem_runtime_ms
            );
            println!(
                "  reporting: {} ms",
                output.runtime_breakdown.reporting_runtime_ms
            );
            println!("Live habit update:");
            println!(
                "  habit_created: {}",
                if output.live_habit_update.habit_created {
                    "yes"
                } else {
                    "no"
                }
            );
            println!(
                "  habit_strengthened: {}",
                if output.live_habit_update.habit_strengthened {
                    "yes"
                } else {
                    "no"
                }
            );
            println!(
                "  habit: {}",
                output
                    .live_habit_update
                    .habit_id
                    .clone()
                    .unwrap_or_else(|| "none".to_string())
            );
            println!("  reason: {}", output.live_habit_update.reason);
            println!(
                "Adaptive budget: {:?}",
                output.adaptive_budget.decision_type
            );
            println!("Reason: {}", output.adaptive_budget.reason);
            println!(
                "Estimated savings: {:.0}%",
                output.adaptive_budget.estimated_savings * 100.0
            );
            println!(
                "Optimization hint: {} ({})",
                output.optimization_hint.reason, output.optimization_hint.recommended_command
            );
            println!("Final status: {}", output.final_status);
            println!("Project report path: {}", output.project_report_path);
            println!(
                "JSON report path: {}",
                output
                    .json_report_path
                    .clone()
                    .unwrap_or_else(|| "none".to_string())
            );
            println!(
                "Session id: {}",
                output
                    .session_id
                    .clone()
                    .unwrap_or_else(|| "none".to_string())
            );
            println!("Journal entries: {}", output.journal_summary.len());
            println!("Snapshots: {}", output.snapshot_summary.len());
            println!("Rollback readiness: {:.2}", output.rollback_readiness);
            println!("Reliability score: {:.2}", output.reliability_score.overall);
            println!("RAM-minimal note: {}", output.ram_minimal_note);
        }
        Command::Projects => {
            println!("Onyx Brain {ONYX_VERSION} projects");
            for project in brain.projects()? {
                println!(
                    "{} | {} | {} | {} | {}",
                    project.project_name,
                    project.status,
                    project.root_path,
                    project.updated_at,
                    project.goal_id
                );
            }
        }
        Command::Goal { input } => {
            let output = brain.execute_goal(input.clone())?;
            println!("Onyx Brain {ONYX_VERSION} goal execution");
            println!("Goal: {input}");
            println!("Goal status: {:?}", output.goal_status);
            println!(
                "Project: {}",
                output.project_name.unwrap_or_else(|| "none".to_string())
            );
            println!("Skills reused: {}", output.skills_reused.join(", "));
            println!(
                "Habits used: {}",
                if output.habits_used.is_empty() {
                    "none".to_string()
                } else {
                    output.habits_used.join(", ")
                }
            );
            println!(
                "Adaptive budget: {:?}",
                output.adaptive_budget.decision_type
            );
            println!("Reason: {}", output.adaptive_budget.reason);
            println!(
                "Estimated savings: {:.0}%",
                output.adaptive_budget.estimated_savings * 100.0
            );
            println!(
                "Fast path: {}",
                if output.fast_path_decision.used_fast_path {
                    "used"
                } else {
                    "not used"
                }
            );
            println!(
                "Cargo validation: check={}, test={}, reason={}",
                output.cargo_validation_policy.run_cargo_check,
                output.cargo_validation_policy.run_cargo_test,
                output.cargo_validation_policy.reason
            );
            println!(
                "Runtime breakdown: brain {} ms, tools {} ms, cargo {} ms",
                output.runtime_breakdown.brain_runtime_ms,
                output.runtime_breakdown.tool_runtime_ms,
                output.runtime_breakdown.cargo_runtime_ms()
            );
            println!(
                "Live habit update: created={}, strengthened={}, reason={}",
                output.live_habit_update.habit_created,
                output.live_habit_update.habit_strengthened,
                output.live_habit_update.reason
            );
            println!(
                "Optimization hint: {} ({})",
                output.optimization_hint.reason, output.optimization_hint.recommended_command
            );
            println!("Energy estimate: {}", output.energy_estimate);
            println!(
                "Self-evaluation: {:.2}",
                output.self_evaluation.overall_score
            );
            println!("Reliability score: {:.2}", output.reliability_score.overall);
            println!("Goal memory: {}", output.goal_memory_path);
        }
        Command::Goals => {
            println!("Onyx Brain {ONYX_VERSION} goals");
            for goal in brain.goals()? {
                println!(
                    "{} | {:?} | {:?} | {:?} | {}",
                    goal.goal_id, goal.status, goal.priority, goal.project_name, goal.title
                );
            }
        }
        Command::ProjectInspect { project_name } => {
            let output = brain.project_inspect(&project_name)?;
            println!("Onyx Brain {ONYX_VERSION} project inspect");
            println!("Project: {}", output.project_name);
            println!("Root: {}", output.root_path);
            println!("Status: {}", output.status);
            println!(
                "Last report: {}",
                output.last_report.unwrap_or_else(|| "none".to_string())
            );
            println!("Files:");
            for file in &output.files {
                println!("  - {file}");
            }
            println!("Memories:");
            for memory in &output.memories {
                println!("  - {memory}");
            }
            println!("Task queue:");
            for task in &output.task_queue_status {
                println!("  - {task}");
            }
            println!("Recent errors:");
            for error in &output.recent_errors {
                println!("  - {error}");
            }
            if let Some(evaluation) = output.self_evaluation {
                println!("Self-evaluation overall: {:.2}", evaluation.overall_score);
            }
        }
        Command::Resume { goal_id } => {
            let output = brain.resume_project(&goal_id)?;
            println!("Onyx Brain {ONYX_VERSION} resume");
            println!("Project name: {}", output.project_name);
            println!("Final status: {}", output.final_status);
            println!("Project report path: {}", output.project_report_path);
        }
        Command::MemoryInspect => {
            let report = brain.memory_inspect()?;
            println!("Onyx Brain {ONYX_VERSION} memory inspect");
            println!("Total memories: {}", report.total_memories);
            println!("Semantic memories: {}", report.semantic_memories);
            println!("Procedural memories: {}", report.procedural_memories);
            println!("Project memories: {}", report.project_memories);
            println!("Archived memories: {}", report.archived_memories);
            println!("Duplicate groups: {}", report.duplicate_groups);
            println!("Top reusable skills:");
            for skill in &report.top_reusable_skills {
                println!("  - {skill}");
            }
            println!("Stale memories: {}", report.stale_memories);
            println!("Memory index size: {}", report.memory_index_size);
            println!("Recommendation: {}", report.recommendation);
        }
        Command::MemoryDedup => {
            let report = brain.memory_dedup()?;
            println!("Onyx Brain {ONYX_VERSION} memory dedup");
            println!("Duplicate groups: {}", report.duplicate_groups);
            println!("Memories archived: {}", report.memories_archived);
            println!("Report: {}", report.report_path);
        }
        Command::Benchmark { name } => {
            if name == "compare" {
                let report = brain.benchmark_compare()?;
                println!("Onyx Brain {ONYX_VERSION} benchmark compare");
                println!("Last score: {:?}", report.last_score);
                println!("Best score: {:?}", report.best_score);
                println!("Average score: {:.2}", report.average_score);
                println!("Runtime trend: {}", report.runtime_trend);
                println!("Energy trend: {}", report.energy_trend);
                println!("Skill reuse trend: {}", report.skill_reuse_trend);
                println!(
                    "Skill reuse quality trend: {}",
                    report.skill_reuse_quality_trend
                );
                println!("Habit usage trend: {}", report.habit_usage_trend);
                println!("Cache hit rate trend: {}", report.cache_hit_rate_trend);
                println!("Route efficiency trend: {}", report.route_efficiency_trend);
                println!("Memory hygiene trend: {}", report.memory_hygiene_trend);
            } else if name == "reliability" {
                let report = brain.benchmark_reliability()?;
                println!("Onyx Brain {ONYX_VERSION} reliability benchmark");
                println!("Tasks run: {}", report.tasks_run);
                println!("Tasks successful: {}", report.tasks_successful);
                println!("Rollback success: {}", report.rollback_success);
                println!(
                    "Snapshot restore success: {}",
                    report.snapshot_restore_success
                );
                println!("Doctor critical issues: {}", report.doctor_critical_issues);
                println!(
                    "Regression check passed: {}",
                    report.regression_check_passed
                );
                println!("Reliability score: {:.2}", report.reliability_score);
                println!("Runtime ms: {}", report.runtime_ms);
                println!("Report: {}", report.report_path);
            } else if name == "autonomy" {
                let report = brain.benchmark_autonomy()?;
                println!("Onyx Brain {ONYX_VERSION} autonomy benchmark");
                println!("Tasks run: {}", report.tasks_run);
                println!("Tasks successful: {}", report.tasks_successful);
                println!("Artifacts created: {}", report.artifacts_created);
                println!("Validation pass rate: {:.2}", report.validation_pass_rate);
                println!("Repairs performed: {}", report.repairs_performed);
                println!("Safety stops: {}", report.safety_stops);
                println!("Reliability score: {:.2}", report.reliability_score);
                println!("Autonomy score: {:.2}", report.autonomy_score);
                println!(
                    "Artifact completion rate: {:.2}",
                    report.artifact_completion_rate
                );
                println!("Revision success rate: {:.2}", report.revision_success_rate);
                println!("Average quality score: {:.2}", report.average_quality_score);
                println!("Assumptions recorded: {}", report.assumptions_recorded);
                println!("Limitations recorded: {}", report.limitations_recorded);
                println!("Recipe reuse count: {}", report.recipe_reuse_count);
                println!("Workspace health: {:.2}", report.workspace_health);
                println!("Runtime ms: {}", report.runtime_ms);
                println!("Report: {}", report.report_path);
            } else if name == "artifacts" {
                let report = brain.benchmark_artifacts()?;
                println!("Onyx Brain {ONYX_VERSION} artifacts benchmark");
                println!("Tasks run: {}", report.tasks_run);
                println!("Tasks successful: {}", report.tasks_successful);
                println!("Artifacts created: {}", report.artifacts_created);
                println!("Completeness score: {:.2}", report.artifact_completion_rate);
                println!("Quality score: {:.2}", report.average_quality_score);
                println!("Repair count: {}", report.repairs_performed);
                println!(
                    "Report card grade: {}",
                    grade_from_score(report.autonomy_score)
                );
                println!("Report: {}", report.report_path);
            } else if name == "advanced-autonomy" {
                let report = brain.benchmark_advanced_autonomy()?;
                println!("Onyx Brain {ONYX_VERSION} advanced autonomy benchmark");
                println!("Tasks run: {}", report.tasks_run);
                println!("Tasks successful: {}", report.tasks_successful);
                println!("Artifacts created: {}", report.artifacts_created);
                println!("Average quality score: {:.2}", report.average_quality_score);
                println!("Consistency score: {:.2}", report.artifact_completion_rate);
                println!(
                    "Report card grade: {}",
                    grade_from_score(report.autonomy_score)
                );
                println!("Export success rate: {:.2}", report.workspace_health);
                println!("Safety stops: {}", report.safety_stops);
                println!("Runtime ms: {}", report.runtime_ms);
                println!("Report: {}", report.report_path);
            } else if name == "conversation" {
                let report = brain.benchmark_conversation()?;
                println!("Onyx Brain {ONYX_VERSION} conversation benchmark");
                println!("Modes tested: {}", report.modes_tested);
                println!("Responses generated: {}", report.responses_generated);
                println!("Average quality: {:.2}", report.average_quality);
                println!("Safety pass rate: {:.2}", report.safety_pass_rate);
                println!("Runtime ms: {}", report.runtime_ms);
                println!("Failures: {}", report.failures.len());
                println!("Report: {}", report.report_path);
            } else if name == "creative" {
                let report = brain.benchmark_creative()?;
                println!("Onyx Brain {ONYX_VERSION} creative benchmark");
                println!("Tasks run: {}", report.tasks_run);
                println!("Tasks successful: {}", report.tasks_successful);
                println!("Artifacts created: {}", report.artifacts_created);
                println!("Validation passed: {}", report.validation_passed);
                println!("Runtime ms: {}", report.runtime_ms);
                println!("Report: {}", report.report_path);
            } else if name == "executive" {
                let report = brain.benchmark_executive()?;
                println!("Onyx Brain {ONYX_VERSION} executive benchmark");
                println!("Decisions recorded: {}", report.decisions_recorded);
                println!("Self-model updated: {}", report.self_model_updated);
                println!("Safety checked: {}", report.safety_checked);
                println!("Runtime ms: {}", report.runtime_ms);
                println!("Report: {}", report.report_path);
            } else if name == "gui-smoke" {
                let report = brain.benchmark_gui_smoke()?;
                println!("Onyx Brain {ONYX_VERSION} GUI smoke benchmark");
                println!(
                    "GUI status: {}",
                    if report.launched {
                        "ready"
                    } else {
                        "missing assets"
                    }
                );
                println!("Views: {}", report.views.len());
                println!("Assets: {}", report.asset_path);
            } else {
                let report = brain.benchmark(&name)?;
                println!("Onyx Brain {ONYX_VERSION} benchmark");
                println!("Benchmark: {}", report.benchmark_name);
                println!("Tasks run: {}", report.tasks_run);
                println!("Tasks successful: {}", report.tasks_successful);
                println!("Tasks failed: {}", report.tasks_failed);
                println!("Total runtime ms: {}", report.total_runtime_ms);
                println!("Reused skills: {}", report.reused_skills_count);
                println!("Irrelevant skills used: {}", report.irrelevant_skills_used);
                println!("Habits used: {}", report.habits_used);
                println!("Cache hits: {}", report.cache_hits);
                println!("Template cache hits: {}", report.template_cache_hits);
                println!(
                    "Adaptive budget decisions: {}",
                    report.adaptive_budget_decisions
                );
                println!(
                    "Average route efficiency: {:.2}",
                    report.average_route_efficiency
                );
                println!("Memories created: {}", report.memories_created);
                println!("Memories archived: {}", report.memories_archived);
                println!("Final score: {:.2}", report.final_score);
                println!("Runtime diagnosis:");
                println!(
                    "  main bottleneck: {}",
                    report.runtime_diagnosis.main_runtime_source
                );
                println!(
                    "  brain: {:.0}%",
                    report.runtime_diagnosis.brain_runtime_percent * 100.0
                );
                println!(
                    "  tools: {:.0}%",
                    report.runtime_diagnosis.tool_runtime_percent * 100.0
                );
                println!(
                    "  cargo: {:.0}%",
                    report.runtime_diagnosis.cargo_runtime_percent * 100.0
                );
                println!(
                    "  recommendation: {}",
                    report.runtime_diagnosis.recommendation
                );
                println!("Report: {}", report.report_path);
            }
        }
        Command::Optimize => {
            let report = brain.optimize()?;
            println!("Onyx Brain {ONYX_VERSION} optimization");
            println!("Profiles analyzed: {}", report.profiles_analyzed);
            println!("Habits created: {}", report.habits_created);
            println!("Habits strengthened: {}", report.habits_strengthened);
            println!("Routes optimized: {}", report.routes_optimized);
            println!(
                "Low-efficiency routes penalized: {}",
                report.low_efficiency_routes_penalized
            );
            println!(
                "Irrelevant skills penalized: {}",
                report.irrelevant_skills_penalized
            );
            println!("Recommendations: {}", report.recommendations.join("; "));
            println!("Report: {}", report.report_path);
        }
        Command::Habits => {
            let habits = brain.habits()?;
            println!("Onyx Brain {ONYX_VERSION} habits");
            if habits.is_empty() {
                println!(
                    "No habits formed yet. Run repeated successful tasks and then `cargo run -- optimize`."
                );
            }
            for habit in habits {
                println!(
                    "{} | confidence {:.2} | success {} failure {} | runtime {:.0}ms | energy {:.1}",
                    habit.title,
                    habit.confidence,
                    habit.success_count,
                    habit.failure_count,
                    habit.average_runtime_ms,
                    habit.average_energy
                );
                println!("  triggers: {}", habit.trigger_patterns.join(", "));
                println!("  skills: {}", habit.preferred_skills.join(", "));
                println!("  tools: {}", habit.preferred_tools.join(", "));
            }
        }
        Command::Routes => {
            let routes = brain.routes()?;
            println!("Onyx Brain {ONYX_VERSION} routes");
            println!("Routes tracked: {}", routes.route_count);
            println!("Average efficiency: {:.2}", routes.average_efficiency);
            println!("Most efficient routes:");
            for row in &routes.top_routes {
                println!("  - {row}");
            }
            println!("Least efficient routes:");
            for row in &routes.least_efficient_routes {
                println!("  - {row}");
            }
            println!("High failure routes:");
            for row in &routes.high_failure_routes {
                println!("  - {row}");
            }
        }
        Command::CacheInspect => {
            let cache = brain.cache_inspect()?;
            println!("Onyx Brain {ONYX_VERSION} plan cache");
            println!("Entries: {}", cache.entries);
            println!("Top cached plans:");
            for row in &cache.top_cached_plans {
                println!("  - {row}");
            }
            println!(
                "Estimated runtime saved: {:.0} ms",
                cache.estimated_runtime_saved
            );
            println!("Cache hit rate: {:.2}", cache.cache_hit_rate);
            println!("Failed cached plans: {}", cache.failed_cached_plans);
        }
        Command::TemplateCacheInspect => {
            let cache = brain.template_cache_inspect()?;
            println!("Onyx Brain {ONYX_VERSION} template cache");
            println!("Entries: {}", cache.entries);
            if cache.top_templates.is_empty() {
                println!("No templates cached yet. Run a successful Rust project creation first.");
            }
            println!("Top templates:");
            for row in &cache.top_templates {
                println!("  - {row}");
            }
            println!(
                "Estimated runtime saved: {:.0} ms",
                cache.estimated_runtime_saved
            );
            println!("Cache hit rate: {:.2}", cache.cache_hit_rate);
        }
        Command::Autonomize { level, input } | Command::Auto { level, input } => {
            let output = brain.autonomize(input.clone(), level.into())?;
            println!("Onyx Brain {ONYX_VERSION} autonomous worker");
            println!("Goal: {input}");
            println!("Session: {}", output.session_id);
            println!("Goal id: {}", output.goal_id);
            println!("Status: {:?}", output.status);
            println!("Tasks planned: {}", output.tasks_planned);
            println!("Tasks completed: {}", output.tasks_completed);
            println!("Tasks failed: {}", output.tasks_failed);
            println!("Artifacts created: {}", output.artifacts_created.len());
            for path in &output.artifacts_created {
                println!("  - {path}");
            }
            println!("Validation passed: {}", output.validation_passed);
            println!("Reliability score: {:.2}", output.reliability_score);
            println!("Autonomy score: {:.2}", output.autonomy_score);
            println!("Recovery actions: {}", output.recovery_actions.join("; "));
            println!("Final report: {}", output.final_report_path);
            println!(
                "Bounded autonomy note: no network, no unrestricted shell, sandboxed writes, finite task/retry limits."
            );
        }
        Command::AutonomyPolicy => {
            let policy = brain.autonomy_policy()?;
            println!("Onyx Brain {ONYX_VERSION} autonomy policy");
            println!("Summary: {}", policy.summary);
            println!("Max session minutes: {}", policy.limits.max_session_minutes);
            println!("Max tasks: {}", policy.limits.max_tasks);
            println!("Max phases: {}", policy.limits.max_phases);
            println!(
                "Max retries per task: {}",
                policy.limits.max_retries_per_task
            );
            println!("Max tool actions: {}", policy.limits.max_tool_actions);
            println!(
                "Network allowed by default: {}",
                policy.limits.network_allowed
            );
            println!(
                "Unrestricted shell allowed: {}",
                policy.limits.unrestricted_shell_allowed
            );
            println!("Safety rules: {}", policy.safety_rules.join("; "));
        }
        Command::Artifacts => {
            let overview = brain.artifacts()?;
            println!("Onyx Brain {ONYX_VERSION} artifacts");
            println!("Artifacts: {}", overview.count);
            for artifact in overview.artifacts {
                println!(
                    "{} | {:?} | {:.2}",
                    artifact.session_id, artifact.artifact_type, artifact.validation_score
                );
                println!("  {}", artifact.path);
            }
        }
        Command::ArtifactInspect { selector } => {
            let report = brain.artifact_inspect(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} artifact inspect");
            println!("Session: {}", report.session_id);
            println!("Validation passed: {}", report.validation_passed);
            println!("Validation score: {:.2}", report.validation_score);
            println!("Manifest: {:?}", report.manifest_path);
            println!("Report: {:?}", report.report_path);
            println!("Files:");
            for file in report.files {
                println!("  - {file}");
            }
        }
        Command::ArtifactPacks | Command::Packs => {
            let overview = brain.artifact_packs()?;
            println!("Onyx Brain {ONYX_VERSION} artifact packs");
            println!("Packs: {}", overview.count);
            for pack in overview.packs {
                println!(
                    "{} | {} | {:.2}",
                    pack.session_id, pack.title, pack.validation_score
                );
                println!("  {}", pack.manifest_path);
            }
        }
        Command::ArtifactPackInspect { selector } | Command::PackInspect { selector } => {
            let report = brain.artifact_pack_inspect(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} artifact pack inspect");
            println!("Pack: {}", report.pack_title);
            println!("Manifest: {}", report.manifest_path);
            println!("Artifacts:");
            for artifact in report.artifacts {
                println!("  - {artifact}");
            }
            println!("Dependency graph:");
            for edge in report.dependency_graph {
                println!("  - {edge}");
            }
            println!(
                "Failed or missing artifacts: {}",
                report.failed_or_missing_artifacts.len()
            );
        }
        Command::ReviewArtifacts { selector } => {
            let report = brain.review_artifacts(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} artifact review");
            println!("Session: {}", report.session_id);
            println!("Overall score: {:.2}", report.overall_score);
            println!("Issues: {}", report.issues.len());
            for issue in report.issues {
                println!("  - {:?}: {}", issue.severity, issue.message);
            }
            println!("Report: {}", report.report_path);
        }
        Command::RepairArtifacts { selector } => {
            let report = brain.repair_artifacts(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} artifact repair");
            println!("Session: {}", report.session_id);
            println!("Overall score: {:.2}", report.overall_score);
            println!("Remaining issues: {}", report.issues.len());
            println!("Report: {}", report.report_path);
        }
        Command::Workspaces => {
            let overview = brain.workspaces()?;
            println!("Onyx Brain {ONYX_VERSION} workspaces");
            println!("Workspaces: {}", overview.count);
            for workspace in overview.workspaces {
                println!("  - {workspace}");
            }
        }
        Command::WorkspaceInspect { selector } => {
            let report = brain.workspace_inspect(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} workspace inspect");
            println!("Workspace: {}", report.workspace_id);
            println!("Session: {}", report.session_id);
            println!("Root: {}", report.root_path);
            println!("Status: {}", report.status);
            for file in report.files {
                println!("  - {file}");
            }
        }
        Command::Recipes => {
            println!("Onyx Brain {ONYX_VERSION} recipes");
            for recipe in brain.recipes()? {
                println!(
                    "{} | {:.2} | {}",
                    recipe.recipe_id, recipe.confidence, recipe.title
                );
            }
        }
        Command::RecipeInspect { selector } => {
            let recipe = brain.recipe_inspect(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} recipe inspect");
            println!("Recipe: {}", recipe.title);
            println!("Triggers: {}", recipe.trigger_keywords.join(", "));
            println!("Phases: {}", recipe.phase_templates.join(", "));
            println!("Validation: {}", recipe.validation_rules.join(", "));
        }
        Command::AutonomyStatus | Command::AutoStatus => {
            let status = brain.autonomy_status()?;
            println!("Onyx Brain {ONYX_VERSION} autonomy status");
            println!("Autonomous sessions: {}", status.autonomous_sessions);
            println!("Artifact packs: {}", status.artifact_packs);
            println!(
                "Average autonomy score: {:.2}",
                status.average_autonomy_score
            );
            println!("Average quality score: {:.2}", status.average_quality_score);
            println!("Repairs performed: {}", status.repairs_performed);
            println!("Safety stops: {}", status.safety_stops);
            println!("Top recipes:");
            for recipe in status.top_recipes {
                println!("  - {recipe}");
            }
            println!("Recommendations: {}", status.recommendations.join("; "));
        }
        Command::ExportPackage { selector } | Command::Export { selector } => {
            let report = brain.export_package(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} export package");
            println!("Session: {}", report.session_id);
            println!("Export path: {}", report.export_path);
            println!("Files exported: {}", report.files_exported);
        }
        Command::ExportInspect { selector } => {
            let report = brain.export_inspect(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} export inspect");
            println!("Export: {}", report.export_path);
            println!("Files:");
            for file in report.files {
                println!("  - {file}");
            }
        }
        Command::Exports => {
            let overview = brain.exports()?;
            println!("Onyx Brain {ONYX_VERSION} exports");
            println!("Exports: {}", overview.count);
            for export in overview.exports {
                println!("  - {export}");
            }
        }
        Command::SessionReport { selector } | Command::Report { selector } => {
            let report = brain.session_report(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} session report");
            println!("Session: {}", report.session_id);
            println!("Goal: {}", report.goal);
            println!("Status: {}", report.status);
            println!("Tasks completed: {}", report.tasks_completed);
            println!("Artifacts created: {}", report.artifacts_created.len());
            println!("Validation score: {:.2}", report.validation_score);
            println!("Repairs performed: {}", report.repairs_performed);
            println!("Reliability score: {:.2}", report.reliability_score);
            println!("Final report: {}", report.final_report_path);
            println!("Session report: {}", report.markdown_report_path);
        }
        Command::TaskGraph { selector } => {
            let graph = brain.task_graph(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} task graph");
            println!("Graph: {}", graph.graph_id);
            println!("Status: {:?}", graph.status);
            println!("Tasks:");
            for task in graph.nodes {
                println!(
                    "  - {} | {:?} | {:?}",
                    task.task_id, task.task_type, task.status
                );
            }
            println!("Dependencies:");
            for edge in graph.edges {
                println!(
                    "  - {} -> {} ({})",
                    edge.from_task_id, edge.to_task_id, edge.relation
                );
            }
        }
        Command::Reflections => {
            println!("Onyx Brain {ONYX_VERSION} reflections");
            for reflection in brain.reflections()? {
                println!(
                    "{} | {} | confidence {:.2}",
                    reflection.session_id, reflection.goal_type, reflection.confidence
                );
            }
        }
        Command::ImproveRecipes => {
            let report = brain.improve_recipes()?;
            println!("Onyx Brain {ONYX_VERSION} recipe improvement");
            println!("Recipes reviewed: {}", report.recipes_reviewed);
            println!("Recipes improved: {}", report.recipes_improved);
            println!("Report: {}", report.report_path);
        }
        Command::Capabilities => {
            let matrix = brain.capabilities()?;
            println!("Onyx Brain {ONYX_VERSION} capabilities");
            println!("Can:");
            for row in matrix.can_do {
                println!("  - {row}");
            }
            println!("Cannot:");
            for row in matrix.cannot_do {
                println!("  - {row}");
            }
            println!("Safety boundaries:");
            for row in matrix.safety_boundaries {
                println!("  - {row}");
            }
        }
        Command::Trace { selector } => {
            let trace = brain.trace(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} execution trace");
            println!("Session: {}", trace.session_id);
            println!("Goal: {}", trace.goal);
            for event in trace.events {
                println!(
                    "{} | {} | {} | {} | {}",
                    event.timestamp, event.phase, event.action, event.status, event.output_summary
                );
            }
        }
        Command::AutonomyHistory => {
            let history = brain.autonomy_history()?;
            println!("Onyx Brain {ONYX_VERSION} autonomy history");
            println!("Rows: {}", history.count);
            for row in history.rows {
                println!("  - {row}");
            }
        }
        Command::CleanupAutonomy => {
            let report = brain.cleanup_autonomy()?;
            println!("Onyx Brain {ONYX_VERSION} autonomy cleanup");
            println!("Temp dirs checked: {}", report.temp_dirs_checked);
            println!("Temp files removed: {}", report.temp_files_removed);
            println!("Report: {}", report.report_path);
        }
        Command::QueueRun { input } => {
            let report = brain.queue_run(&input)?;
            println!("Onyx Brain {ONYX_VERSION} queue run");
            println!("Goals total: {}", report.goals_total);
            println!("Goals completed: {}", report.goals_completed);
            println!("Goals failed: {}", report.goals_failed);
            println!("Safety stops: {}", report.safety_stops);
            println!("Artifact packs created: {}", report.artifact_packs_created);
            println!("Report: {}", report.report_path);
        }
        Command::Creative { input } => {
            let report = brain.creative(&input)?;
            println!("Onyx Brain {ONYX_VERSION} creative production studio");
            println!("Session: {}", report.session_id);
            println!("Project: {}", report.title);
            println!("Workspace: {}", report.workspace_path);
            println!("Artifacts created: {}", report.artifacts_created.len());
            println!("Validation passed: {}", report.validation_passed);
            if let Some(caution) = report.originality_caution {
                println!("Originality caution: {caution}");
            }
            println!("Final report: {}", report.final_report_path);
        }
        Command::SelfModel => {
            let model = brain.self_model()?;
            println!("Onyx Brain {ONYX_VERSION} self-model");
            println!("Name: {}", model.name);
            println!("Mode: {}", model.current_mode);
            println!("Confidence: {:.2}", model.confidence_state.score);
            println!("Limitations: {}", model.limitations.join("; "));
        }
        Command::Attention => {
            let attention = brain.attention()?;
            println!("Onyx Brain {ONYX_VERSION} attention state");
            println!("Active goal: {:?}", attention.active_goal);
            println!("Active task: {:?}", attention.active_task);
            println!("Focus score: {:.2}", attention.focus_score);
        }
        Command::Metacognition { input } => {
            let report = brain.metacognition(&input)?;
            println!("Onyx Brain {ONYX_VERSION} metacognition");
            println!("Doing: {}", report.what_i_am_doing);
            println!("Why: {}", report.why_i_am_doing_it);
            println!("Known: {}", report.what_i_know.join("; "));
            println!("Unknown: {}", report.what_i_do_not_know.join("; "));
            println!("Next best action: {}", report.next_best_action);
            println!("Confidence: {:.2}", report.confidence);
        }
        Command::ExecutiveStatus => {
            let status = brain.executive_status()?;
            println!("Onyx Brain {ONYX_VERSION} executive status");
            println!("Active goal: {:?}", status.active_goal);
            println!("Active task: {:?}", status.active_task);
            println!("Confidence: {:.2}", status.confidence);
            println!("Safety: {}", status.safety_state);
            println!("Recent decisions: {}", status.recent_decisions.join("; "));
        }
        Command::Chat { input } => {
            if let Some(input) = input {
                let output = brain.chat_once(&input)?;
                println!("Onyx Brain {ONYX_VERSION} chat");
                println!("Mode: {}", output.mode);
                println!("{}", output.response);
            } else {
                brain.chat_loop()?;
            }
        }
        Command::Modes => {
            println!("Onyx Brain {ONYX_VERSION} conversation modes");
            for mode in brain.modes() {
                println!("{}: {}", mode.name, mode.description);
            }
        }
        Command::Mode {
            mode,
            show_quality,
            input,
        } => {
            let output = brain.run_mode(mode.into(), &input, show_quality)?;
            println!("Onyx Brain {ONYX_VERSION} mode {}", output.mode);
            println!("{}", output.response);
        }
        Command::Personality { command } => match command {
            Some(PersonalityCommand::Set { profile }) => {
                let profile = brain.set_personality(profile.into())?;
                println!("Onyx Brain {ONYX_VERSION} personality set: {:?}", profile);
            }
            None => {
                println!(
                    "Onyx Brain {ONYX_VERSION} personality: {:?}",
                    brain.personality()?
                );
            }
        },
        Command::ConversationMemory => {
            println!("Onyx Brain {ONYX_VERSION} conversation memory");
            for row in brain.conversation_memory()? {
                println!("{} | {} | {:.2}", row.topic, row.summary, row.importance);
            }
        }
        Command::PromptLibrary => {
            println!("Onyx Brain {ONYX_VERSION} prompt library");
            for pattern in brain.prompt_library() {
                println!(
                    "{} | {:?} | triggers: {}",
                    pattern.name,
                    pattern.mode,
                    pattern.trigger_keywords.join(", ")
                );
            }
        }
        Command::Transcript { selector } => {
            let transcript = brain.transcript(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} transcript");
            println!("Session: {}", transcript.session_id);
            println!("Summary: {}", transcript.summary);
            for message in transcript.messages {
                println!(
                    "{:?}: {}",
                    message.role,
                    message.content.lines().next().unwrap_or("")
                );
            }
        }
        Command::TranscriptExport { selector } => {
            let report = brain.transcript_export(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} transcript export");
            println!("Session: {}", report.session_id);
            println!("Export: {}", report.export_path);
            println!("Files: {}", report.files_written.len());
        }
        Command::Journal { session } => {
            println!("Onyx Brain {ONYX_VERSION} journal");
            for entry in brain.journal(session)? {
                println!(
                    "{} | {:?} | {:?} | project {:?} | rollback {}",
                    entry.created_at,
                    entry.action_type,
                    entry.status,
                    entry.project_id,
                    entry.rollback_available
                );
            }
        }
        Command::Snapshots => {
            let report = brain.snapshots()?;
            println!("Onyx Brain {ONYX_VERSION} snapshots");
            println!("Snapshots: {}", report.count);
            for row in report.snapshots {
                println!("  - {row}");
            }
        }
        Command::SnapshotCreate {
            project_name,
            reason,
        } => {
            let snapshot = brain.snapshot_create(&project_name, &reason)?;
            println!("Onyx Brain {ONYX_VERSION} snapshot create");
            println!("Project: {}", snapshot.project_name);
            println!("Snapshot: {}", snapshot.snapshot_id);
            println!("Files: {}", snapshot.files.len());
            println!("Bytes: {}", snapshot.total_bytes);
        }
        Command::SnapshotRestore { snapshot_id } => {
            let report = brain.snapshot_restore(&snapshot_id)?;
            println!("Onyx Brain {ONYX_VERSION} snapshot restore");
            println!("Snapshot: {}", report.snapshot_id);
            println!("Project: {}", report.project_name);
            println!("Files restored: {}", report.files_restored);
            println!("Status: {}", report.status);
        }
        Command::Rollback { project, selector } => {
            if selector != "latest" {
                return Err(anyhow::anyhow!("only rollback latest is supported"));
            }
            let report = brain.rollback_latest(project.as_deref())?;
            println!("Onyx Brain {ONYX_VERSION} rollback");
            println!("Rollback: {}", report.rollback_id);
            println!("Target entry: {}", report.target_entry_id);
            println!("Files restored: {}", report.files_restored);
            println!("Status: {}", report.status);
            println!("Report: {}", report.report_path);
        }
        Command::Transactions => {
            let report = brain.transactions()?;
            println!("Onyx Brain {ONYX_VERSION} transactions");
            println!("Transactions: {}", report.count);
            for row in report.transactions {
                println!("  - {row}");
            }
        }
        Command::Doctor { repair } => {
            let report = brain.doctor(repair)?;
            println!("Onyx Brain {ONYX_VERSION} doctor");
            println!("Issues found: {}", report.issues_found);
            println!("Critical: {}", report.critical);
            println!("Warnings: {}", report.warnings);
            println!("Repair available: {}", report.repair_available);
            println!("Repaired: {}", report.repaired);
            println!("Recommendation: {}", report.recommendation);
            println!("Report: {}", report.report_path);
        }
        Command::Recover { project, selector } => {
            if selector != "latest" {
                return Err(anyhow::anyhow!("only recover latest is supported"));
            }
            let result = brain.recover_latest(project.as_deref())?;
            println!("Onyx Brain {ONYX_VERSION} recover");
            println!("Failure kind: {:?}", result.plan.failure_kind);
            println!("Safe to auto-run: {}", result.plan.safe_to_auto_run);
            println!("Requires review: {}", result.plan.requires_user_review);
            println!("Executed: {}", result.executed);
            println!("Status: {}", result.status);
            println!("Steps: {}", result.plan.suggested_steps.join("; "));
        }
        Command::Sessions => {
            println!("Onyx Brain {ONYX_VERSION} sessions");
            for session in brain.sessions()? {
                println!(
                    "{} | {:?} | {} | {:?}",
                    session.session_id, session.status, session.title, session.ended_at
                );
            }
        }
        Command::SessionStart { title } => {
            let session = brain.session_start(title)?;
            println!("Onyx Brain {ONYX_VERSION} session start");
            println!("Session: {}", session.session_id);
        }
        Command::SessionStatus { selector } => {
            let session = brain.session_status(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} session status");
            println!("Session: {}", session.session_id);
            println!("Status: {:?}", session.status);
            println!("Summary: {}", session.summary);
        }
        Command::SessionEnd { selector } => {
            let session = brain.session_end(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} session end");
            println!("Session: {}", session.session_id);
            println!("Status: {:?}", session.status);
        }
        Command::SessionResume { selector } => {
            let session = brain.session_resume(&selector)?;
            println!("Onyx Brain {ONYX_VERSION} session resume");
            println!("Session: {}", session.session_id);
            println!("Status: {:?}", session.status);
        }
        Command::Worker { input } => {
            let output = brain.worker(input.clone())?;
            println!("Onyx Brain {ONYX_VERSION} worker mode");
            println!("Session: {}", output.session_id);
            println!("Goal: {}", output.goal);
            println!("Phases completed: {}", output.phases_completed);
            println!("Tasks completed: {}", output.tasks_completed);
            println!("Failures: {}", output.failures.join("; "));
            println!("Recovery actions: {}", output.recovery_actions.join("; "));
            println!("Final report: {}", output.final_report);
        }
        Command::RegressionCheck => {
            let report = brain.regression_check()?;
            println!("Onyx Brain {ONYX_VERSION} regression check");
            println!("Checks passed: {}", report.checks_passed);
            println!("Checks failed: {}", report.checks_failed);
            println!("Status: {}", report.status);
            for failure in report.failures {
                println!("  - {failure}");
            }
        }
        Command::CleanupBackups => {
            let report = brain.cleanup_backups()?;
            println!("Onyx Brain {ONYX_VERSION} backup cleanup");
            println!("Backups seen: {}", report.backups_seen);
            println!("Backups cleaned: {}", report.backups_removed);
            println!("Report: {}", report.report_path);
        }
        Command::BrainStatus { summary } => {
            if summary {
                let status = brain.brain_status_summary()?;
                println!("Onyx Brain status summary");
                println!("Version: {}", status.version);
                println!("Neurons: {}", status.neurons);
                println!("Synapses: {}", status.synapses);
                println!("Memories: {}", status.memories);
                println!("Registered projects: {}", status.registered_projects);
                println!(
                    "Goals active/completed/blocked: {}/{}/{}",
                    status.goals_active, status.goals_completed, status.goals_blocked
                );
                println!("Memory hygiene: {}", status.memory_hygiene);
                println!("Habits: {}", status.habits_count);
                println!("Plan cache entries: {}", status.cache_entries);
                println!("Last benchmark score: {:?}", status.last_benchmark_score);
                println!(
                    "Average route efficiency: {:.2}",
                    status.average_route_efficiency
                );
                println!("Recommended action: {}", status.recommended_action);
                println!("Reliability: {}", status.reliability_summary);
                println!("Conversation: {}", status.conversation_summary);
                println!("Executive: {}", status.executive_summary);
                if !status.environment_notes.is_empty() {
                    println!("Environment notes: {}", status.environment_notes.join("; "));
                }
                return Ok(());
            }
            let status = brain.brain_status()?;
            println!("Onyx Brain status");
            println!("Version: {}", status.version);
            println!("Neurons: {}", status.neurons);
            println!("Synapses: {}", status.synapses);
            println!(
                "Active registered projects: {}",
                status.active_registered_projects
            );
            println!(
                "Historical project memories: {}",
                status.historical_project_memories
            );
            println!(
                "Goals active/completed/blocked: {}/{}/{}",
                status.goals_active, status.goals_completed, status.goals_blocked
            );
            println!("Memories by type: {}", status.memories_by_type.join(", "));
            println!(
                "Duplicate memory groups: {}",
                status.duplicate_memory_groups
            );
            println!("Top skills by reuse:");
            for skill in &status.top_skills_by_reuse {
                println!("  - {skill}");
            }
            println!("Benchmark last score: {:?}", status.benchmark_last_score);
            println!(
                "Average project self-evaluation: {:.2}",
                status.average_project_self_evaluation
            );
            println!("Memory hygiene score: {:.2}", status.memory_hygiene_score);
            println!("Performance:");
            println!("  profiles: {}", status.performance_profile_count);
            println!(
                "  avg runtime last 5: {:.0} ms",
                status.average_runtime_last_5
            );
            println!(
                "  avg route efficiency: {:.2}",
                status.average_route_efficiency
            );
            println!(
                "  avg brain/tool/cargo last 5: {:.0}/{:.0}/{:.0} ms",
                status.average_brain_runtime_last_5,
                status.average_tool_runtime_last_5,
                status.average_cargo_runtime_last_5
            );
            println!("Habits:");
            println!("  habits: {}", status.habits_count);
            for habit in &status.top_habits {
                println!("  - {habit}");
            }
            println!("Plan cache:");
            println!("  entries: {}", status.plan_cache_entries);
            println!("  cache hit rate: {:.2}", status.cache_hit_rate);
            println!("Adaptive budget:");
            println!(
                "  estimated savings: {:.0}%",
                status.adaptive_budget_savings_estimate * 100.0
            );
            println!(
                "Optimization recommendations: {}",
                status.optimization_recommendations.join(", ")
            );
            println!("Reliability:");
            println!("  journal entries: {}", status.journal_entries_count);
            println!("  snapshots: {}", status.snapshots_count);
            println!("  active sessions: {}", status.active_sessions_count);
            println!("  latest session: {:?}", status.latest_session);
            println!("  doctor: {}", status.doctor_health_summary);
            println!("  rollback readiness: {:.2}", status.rollback_readiness);
            println!(
                "  reliability score: {:.2}",
                status.reliability_score.overall
            );
            println!(
                "  recovery recommendations: {}",
                status.recovery_recommendations.join(", ")
            );
            println!("Autonomy:");
            println!(
                "  autonomous sessions: {}",
                status.autonomous_sessions_count
            );
            println!("  artifacts: {}", status.artifacts_count);
            println!("  last autonomy score: {:.2}", status.last_autonomy_score);
            println!(
                "  last validation score: {:.2}",
                status.last_validation_score
            );
            println!("  safety stops: {}", status.safety_stops_count);
            println!("  repairs performed: {}", status.repairs_performed);
            println!("  policy: {}", status.autonomy_policy_summary);
            println!("Conversation:");
            println!("  sessions: {}", status.conversation_sessions_count);
            println!("  recent mode: {:?}", status.recent_conversation_mode);
            println!("  memories: {}", status.conversation_memory_count);
            println!("  personality: {}", status.current_personality);
            println!(
                "  average response quality: {:.2}",
                status.average_response_quality
            );
            println!(
                "  conversation benchmark score: {:?}",
                status.conversation_benchmark_score
            );
            println!("Executive / Creative / GUI:");
            println!("  GUI status: {}", status.gui_status);
            println!("  creative projects: {}", status.creative_projects_count);
            println!(
                "  executive decisions: {}",
                status.executive_decisions_count
            );
            println!("Environment:");
            println!(
                "  Cloud sync path: {}",
                if status.environment.is_onedrive_path {
                    "yes"
                } else {
                    "no"
                }
            );
            println!(
                "  Path spaces: {}",
                if status.environment.path_has_spaces {
                    "yes"
                } else {
                    "no"
                }
            );
            for note in &status.environment.potential_overhead_notes {
                println!("  Note: {note}");
            }
            println!(
                "Recommended maintenance actions: {}",
                status.recommended_maintenance_actions.join(", ")
            );
        }
        Command::Maintain => {
            let (dedup, backups, consolidation, compare) = brain.maintain()?;
            println!("Maintenance completed");
            println!("Memories archived: {}", dedup.memories_archived);
            println!("Backups cleaned: {}", backups.backups_removed);
            println!("Routes strengthened: {}", consolidation.strengthened_routes);
            println!("Benchmark trend: {}", compare.runtime_trend);
            println!("Recommendations: check brain-status");
        }
        Command::Consolidate => {
            let report = brain.consolidate()?;
            println!("Onyx Brain {ONYX_VERSION} consolidation");
            println!("Logs seen: {}", report.logs_seen);
            println!("Strengthened routes: {}", report.strengthened_routes);
            println!("Pruned synapses: {}", report.pruned_synapses);
            println!(
                "Shortcut synapses created: {}",
                report.shortcut_synapses_created
            );
            println!("Report: {}", report.report_path);
        }
        Command::Inspect {
            summary: summary_mode,
        } => {
            if summary_mode {
                let summary = brain.inspect_summary()?;
                println!("Onyx Brain {ONYX_VERSION} state summary");
                println!("Neurons: {}", summary.neurons);
                println!("Synapses: {}", summary.synapses);
                println!("Memories: {}", summary.memories);
                println!("Logs: {}", summary.logs);
                println!("Registered projects: {}", summary.registered_projects);
                println!("Goals: {}", summary.goals);
                println!("Memory hygiene: {}", summary.memory_hygiene);
                println!("Habits: {}", summary.habits_count);
                println!("Plan cache entries: {}", summary.cache_entries);
                println!("Last benchmark score: {:?}", summary.last_benchmark_score);
                println!(
                    "Average route efficiency: {:.2}",
                    summary.average_route_efficiency
                );
                println!("Recommended action: {}", summary.recommended_action);
                println!("Reliability: {}", summary.reliability_summary);
                println!("Conversation: {}", summary.conversation_summary);
                println!("Executive: {}", summary.executive_summary);
                return Ok(());
            }
            let summary = brain.inspect()?;
            println!("Onyx Brain {ONYX_VERSION} state");
            println!("Neurons: {}", summary.neurons);
            println!("Synapses: {}", summary.synapses);
            println!("Memories: {}", summary.memories);
            println!("Logs: {}", summary.logs);
            println!("Top strongest synapses:");
            for row in &summary.top_strongest_synapses {
                println!("  - {row}");
            }
            println!("Top used neurons:");
            for row in &summary.top_used_neurons {
                println!("  - {row}");
            }
            println!("Top important memories:");
            for row in &summary.top_important_memories {
                println!("  - {row}");
            }
            println!("Last tasks:");
            for row in &summary.last_tasks {
                println!("  - {row}");
            }
            println!(
                "Average energy estimate: {:.2}",
                summary.average_energy_estimate
            );
            println!(
                "Last consolidation: {}",
                summary
                    .last_consolidation_time
                    .unwrap_or_else(|| "never".to_string())
            );
            println!("Project state history:");
            for row in &summary.known_projects {
                println!("  - {row}");
            }
            println!("Failed project tasks:");
            for row in &summary.failed_tasks {
                println!("  - {row}");
            }
            println!("Project retry counts:");
            for row in &summary.retry_counts {
                println!("  - {row}");
            }
            println!(
                "Last project report: {}",
                summary
                    .last_project_report_path
                    .unwrap_or_else(|| "none".to_string())
            );
            println!("Registered projects: {}", summary.registered_project_count);
            println!(
                "Historical project memories: {}",
                summary.historical_project_memories
            );
            println!(
                "Archived project memories: {}",
                summary.archived_project_memories
            );
            println!(
                "Duplicate project memories: {}",
                summary.duplicate_project_memories
            );
            println!(
                "Last modified project: {}",
                summary
                    .last_modified_project
                    .unwrap_or_else(|| "none".to_string())
            );
            println!("Top extracted procedural skills:");
            for row in &summary.top_extracted_skills {
                println!("  - {row}");
            }
            println!(
                "Average project self-evaluation score: {:.2}",
                summary.average_project_self_evaluation_score
            );
            println!(
                "Failed/blocked project tasks: {}",
                summary.failed_or_blocked_task_count
            );
            println!("Memory hygiene:");
            println!(
                "  duplicate project memories: {}",
                summary.memory_hygiene.duplicate_project_memories
            );
            println!(
                "  duplicate procedural skills: {}",
                summary.memory_hygiene.duplicate_procedural_skills
            );
            println!(
                "  recommendation: {}",
                summary.memory_hygiene.recommendation
            );
            println!("Route efficiency top 10:");
            for row in &summary.route_efficiency_top {
                println!("  - {row}");
            }
            println!("Habit summary:");
            for row in &summary.habit_summary {
                println!("  - {row}");
            }
            println!("Cache summary:");
            for row in &summary.cache_summary {
                println!("  - {row}");
            }
            println!("Slowest recent tasks:");
            for row in &summary.slowest_recent_tasks {
                println!("  - {row}");
            }
            println!("Average runtime: {:.0} ms", summary.average_runtime_ms);
            println!("Average energy: {:.2}", summary.average_energy);
            println!(
                "Adaptive budget summary: {}",
                summary.adaptive_budget_summary
            );
            println!("Latest journal entries:");
            for row in &summary.latest_journal_entries {
                println!("  - {row}");
            }
            println!("Latest snapshots:");
            for row in &summary.latest_snapshots {
                println!("  - {row}");
            }
            println!("Recent sessions:");
            for row in &summary.recent_sessions {
                println!("  - {row}");
            }
            println!("Recovery reports:");
            for row in &summary.recovery_reports {
                println!("  - {row}");
            }
            println!("Doctor: {}", summary.doctor_health_summary);
            println!("Transactions:");
            for row in &summary.transaction_summary {
                println!("  - {row}");
            }
            println!("Reliability: {}", summary.reliability_summary);
            println!("Sandbox: {}", summary.sandbox);
        }
        Command::MemoryAdd {
            memory_type,
            title,
            tags,
            content,
        } => {
            let tags = tags
                .split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            let memory = brain.add_memory(memory_type.into(), title, tags, content)?;
            println!("Added memory: {}", memory.id);
        }
    }
    Ok(())
}

impl From<MemoryTypeArg> for MemoryType {
    fn from(value: MemoryTypeArg) -> Self {
        match value {
            MemoryTypeArg::Working => MemoryType::Working,
            MemoryTypeArg::Episodic => MemoryType::Episodic,
            MemoryTypeArg::Semantic => MemoryType::Semantic,
            MemoryTypeArg::Procedural => MemoryType::Procedural,
            MemoryTypeArg::Project => MemoryType::Project,
        }
    }
}

fn grade_from_score(score: f32) -> &'static str {
    if score >= 0.9 {
        "A"
    } else if score >= 0.8 {
        "B"
    } else if score >= 0.7 {
        "C"
    } else if score >= 0.6 {
        "D"
    } else {
        "F"
    }
}
