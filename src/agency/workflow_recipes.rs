use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::{
    agency::{DeliverableKind, GoalType},
    storage::{load_json, save_json, DiskStore},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRecipe {
    pub recipe_id: String,
    pub title: String,
    pub trigger_keywords: Vec<String>,
    pub goal_type: GoalType,
    pub deliverable_kinds: Vec<DeliverableKind>,
    pub phase_templates: Vec<String>,
    pub validation_rules: Vec<String>,
    pub artifact_templates: Vec<String>,
    pub success_count: u64,
    pub failure_count: u64,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeIndex {
    #[serde(default)]
    pub recipes: Vec<WorkflowRecipe>,
}

pub fn recipe_index_path(store: &DiskStore) -> std::path::PathBuf {
    store.paths.indexes.join("recipe_index.json")
}

pub fn ensure_default_recipes(store: &DiskStore) -> Result<RecipeIndex> {
    fs::create_dir_all(store.paths.data.join("recipes"))?;
    let path = recipe_index_path(store);
    if path.exists() {
        return load_json(&path);
    }
    let recipes = RecipeIndex {
        recipes: vec![
            recipe(
                "recipe_launch_kit",
                "Launch Kit",
                vec![
                    "launch kit",
                    "startup pack",
                    "product launch",
                    "open-source launch",
                    "pitch deck",
                ],
            ),
            recipe(
                "recipe_technical_report",
                "Technical Report Pack",
                vec!["technical report", "architecture report", "system design"],
            ),
            recipe(
                "recipe_product_spec",
                "Product Spec Pack",
                vec!["product spec", "prd", "requirements"],
            ),
            recipe(
                "recipe_presentation_pack",
                "Presentation Pack",
                vec!["presentation", "slides"],
            ),
            recipe(
                "recipe_learning_pack",
                "Learning Pack",
                vec!["learning pack", "study guide", "quiz", "glossary"],
            ),
            recipe(
                "recipe_project_proposal",
                "Project Proposal",
                vec!["proposal", "roadmap", "risks", "budget"],
            ),
            recipe(
                "recipe_rust_project",
                "Rust Project Worker",
                vec!["rust project", "cargo", "cli"],
            ),
            recipe(
                "recipe_documentation_pack",
                "Documentation Pack",
                vec!["documentation", "user guide", "faq"],
            ),
            recipe(
                "recipe_benchmark_report",
                "Benchmark and Report",
                vec!["benchmark", "report"],
            ),
        ],
    };
    for recipe in &recipes.recipes {
        save_json(
            &store
                .paths
                .data
                .join("recipes")
                .join(format!("{}.json", recipe.recipe_id)),
            recipe,
        )?;
    }
    save_json(&path, &recipes)?;
    Ok(recipes)
}

pub fn recipes(store: &DiskStore) -> Result<Vec<WorkflowRecipe>> {
    Ok(ensure_default_recipes(store)?.recipes)
}

pub fn recipe_inspect(store: &DiskStore, selector: &str) -> Result<WorkflowRecipe> {
    let recipes = ensure_default_recipes(store)?.recipes;
    if selector.eq_ignore_ascii_case("latest") {
        return recipes
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no recipes found"));
    }
    recipes
        .into_iter()
        .find(|recipe| recipe.recipe_id == selector || recipe.title.eq_ignore_ascii_case(selector))
        .ok_or_else(|| anyhow::anyhow!("recipe not found"))
}

pub fn match_recipe(store: &DiskStore, prompt: &str) -> Result<Option<WorkflowRecipe>> {
    let lower = prompt.to_lowercase();
    Ok(ensure_default_recipes(store)?
        .recipes
        .into_iter()
        .find(|recipe| {
            recipe
                .trigger_keywords
                .iter()
                .any(|keyword| lower.contains(keyword))
        }))
}

fn recipe(id: &str, title: &str, keywords: Vec<&str>) -> WorkflowRecipe {
    WorkflowRecipe {
        recipe_id: id.to_string(),
        title: title.to_string(),
        trigger_keywords: keywords.into_iter().map(str::to_string).collect(),
        goal_type: GoalType::Mixed,
        deliverable_kinds: vec![DeliverableKind::Report],
        phase_templates: vec![
            "understand".to_string(),
            "generate".to_string(),
            "validate".to_string(),
            "report".to_string(),
        ],
        validation_rules: vec![
            "required artifacts exist".to_string(),
            "final report exists".to_string(),
        ],
        artifact_templates: vec!["markdown".to_string()],
        success_count: 0,
        failure_count: 0,
        confidence: 0.75,
    }
}
