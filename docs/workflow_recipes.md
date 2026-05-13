# Workflow Recipes

Workflow recipes are deterministic planning hints for bounded autonomous work. They help Onyx Brain choose a familiar phase structure without adding network access, LLM calls, or unrestricted tools.

Default recipes include:

- Presentation Pack
- Learning Pack
- Project Proposal
- Rust Project Worker
- Documentation Pack
- Benchmark and Report

Recipes are stored in:

```text
data/recipes/
data/indexes/recipe_index.json
```

Use:

```bash
cargo run -- recipes
cargo run -- recipe-inspect latest
```

Recipes are advisory. Validation, safety checks, journaling, and reports still run.
