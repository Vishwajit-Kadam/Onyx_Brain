use onyx_brain::{
    agency::{
        list_goals, load_goal, parse_goal, register_project, GoalMemoryItem, GoalStatus,
        IntentKind, ProjectRecord,
    },
    core::Priority,
    energy::PriorityScheduler,
    learning::{SkillKind, SkillReuseEngine},
    memory::{hygiene::cleanup_backups, MemoryItem, MemoryType},
    storage::DiskStore,
    Brain,
};

fn procedural(id: &str, title: &str, tags: &[&str]) -> MemoryItem {
    let mut memory = MemoryItem::new(
        id,
        MemoryType::Procedural,
        title,
        "Reusable workflow.",
        tags.iter().map(|tag| (*tag).to_string()).collect(),
        vec![],
    );
    memory.importance = 0.9;
    memory
}

#[test]
fn goal_memory_saves_and_loads() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let now = chrono::Utc::now();
    let goal = GoalMemoryItem {
        goal_id: "g1".to_string(),
        title: "demo".to_string(),
        original_prompt: "create demo".to_string(),
        parsed_intent: IntentKind::CreateProject,
        project_name: Some("demo".to_string()),
        status: GoalStatus::Active,
        priority: Priority::Normal,
        created_at: now,
        updated_at: now,
        completed_at: None,
        linked_project_id: None,
        linked_memories: Vec::new(),
        linked_skills: Vec::new(),
        success_score: 0.0,
        energy_spent: 0,
        notes: Vec::new(),
    };
    onyx_brain::agency::save_goal(&store, &goal).expect("save");
    assert_eq!(load_goal(&store, "g1").expect("load").title, "demo");
}

#[test]
fn goal_command_creates_completed_goal_and_lists_it() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let output = brain
        .execute_goal(
            "Create a Rust CLI calculator project called goal_calc with tests and README"
                .to_string(),
        )
        .expect("goal");
    assert_eq!(output.goal_status, GoalStatus::Completed);
    let goals = brain.goals().expect("goals");
    assert!(goals.iter().any(|goal| goal.goal_id == output.goal_id));
}

#[test]
fn priority_scheduler_orders_high_above_low() {
    let now = chrono::Utc::now();
    let make = |id: &str, priority| GoalMemoryItem {
        goal_id: id.to_string(),
        title: id.to_string(),
        original_prompt: id.to_string(),
        parsed_intent: IntentKind::CreateProject,
        project_name: None,
        status: GoalStatus::Active,
        priority,
        created_at: now,
        updated_at: now,
        completed_at: None,
        linked_project_id: None,
        linked_memories: Vec::new(),
        linked_skills: Vec::new(),
        success_score: 0.0,
        energy_spent: 0,
        notes: Vec::new(),
    };
    let ordered = PriorityScheduler::order_goals(vec![
        make("low", Priority::Low),
        make("high", Priority::High),
    ]);
    assert_eq!(ordered[0].goal_id, "high");
}

#[test]
fn projects_command_source_is_registry_only_and_inspect_separates_memories() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let now = chrono::Utc::now();
    register_project(
        brain.store(),
        ProjectRecord {
            goal_id: "registered".to_string(),
            project_name: "registered_project".to_string(),
            root_path: "sandbox/projects/registered_project".to_string(),
            status: "Completed".to_string(),
            created_at: now,
            updated_at: now,
            last_report_path: None,
            tags: vec![],
            summary: String::new(),
        },
    )
    .expect("register");
    brain
        .store()
        .save_memory(&MemoryItem::new(
            "project_memory_old",
            MemoryType::Project,
            "Project old_project",
            "historical",
            vec!["project".to_string()],
            vec![],
        ))
        .expect("memory");
    assert_eq!(brain.projects().expect("projects").len(), 1);
    let inspect = brain.inspect().expect("inspect");
    assert!(inspect.historical_project_memories >= 1);
}

#[test]
fn cleanup_backups_keeps_latest_three() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    let dir = temp.path().join("sandbox/projects/demo/src");
    std::fs::create_dir_all(&dir).expect("dirs");
    for idx in 0..5 {
        std::fs::write(dir.join(format!("lib.rs.bak.{idx}")), "x").expect("write");
    }
    cleanup_backups(&store, 3).expect("cleanup");
    let left = std::fs::read_dir(&dir)
        .expect("list")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".bak."))
        .count();
    assert_eq!(left, 3);
}

#[test]
fn benchmark_history_and_compare_work() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain.benchmark("basic").expect("benchmark");
    let compare = brain.benchmark_compare().expect("compare");
    assert!(compare.last_score.is_some());
}

#[test]
fn brain_status_and_maintain_do_not_crash() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    let status = brain.brain_status().expect("status");
    assert_eq!(status.version, "v0.0.1");
    let _ = brain.maintain().expect("maintain");
}

#[test]
fn skill_reuse_prefers_generic_over_unrelated_workflow() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = DiskStore::new(temp.path());
    store.ensure_layout().expect("layout");
    store
        .save_memory(&procedural(
            "workflow_bench",
            "Workflow for bench_calc",
            &["rust", "project", "workflow", "skill"],
        ))
        .expect("workflow");
    store
        .save_memory(&procedural(
            "skill_generic",
            "Create Rust CLI project",
            &["rust", "project", "workflow", "skill", "cli"],
        ))
        .expect("generic");
    let parsed = parse_goal("Create a Rust CLI project called fresh_calc");
    let matches = SkillReuseEngine::find_relevant_skills(
        &store,
        &parsed,
        &[],
        &onyx_brain::energy::EnergyBudget::default(),
    )
    .expect("matches");
    assert!(matches
        .iter()
        .any(|skill| matches!(skill.skill_kind, SkillKind::GenericSkill)));
    assert!(!matches
        .iter()
        .any(|skill| skill.title.contains("bench_calc")));
}

#[test]
fn memory_hygiene_score_improves_after_dedup_and_goal_links_project_skills() {
    let temp = tempfile::tempdir().expect("tempdir");
    let brain = Brain::new(temp.path());
    brain.init().expect("init");
    brain
        .store()
        .save_memory(&MemoryItem::new(
            "project_a",
            MemoryType::Project,
            "Project dup",
            "a",
            vec!["project".to_string()],
            vec![],
        ))
        .expect("a");
    brain
        .store()
        .save_memory(&MemoryItem::new(
            "project_b",
            MemoryType::Project,
            "Project dup",
            "b",
            vec!["project".to_string()],
            vec![],
        ))
        .expect("b");
    let before = brain
        .inspect()
        .expect("before")
        .memory_hygiene
        .duplicate_groups;
    brain.memory_dedup().expect("dedup");
    let after = brain
        .inspect()
        .expect("after")
        .memory_hygiene
        .duplicate_groups;
    assert!(after <= before);

    let output = brain
        .execute_goal("Create a Rust CLI project called linked_goal with README".to_string())
        .expect("goal");
    let goal = list_goals(brain.store())
        .expect("goals")
        .into_iter()
        .find(|goal| goal.goal_id == output.goal_id)
        .expect("goal saved");
    assert!(goal.linked_project_id.is_some());
}
