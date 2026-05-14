# Conversation Modes

Use `cargo run -- modes` to list modes.

- `standard`: concise structured help.
- `debate`: two sides, counterarguments, common ground, verdict.
- `teacher`: explanation, examples, mini exercise, recap.
- `socratic`: guiding questions and hints.
- `critic`: strengths, weaknesses, risks, improvements.
- `planner`: phases, tasks, dependencies, risks.
- `architect`: modules, data flow, storage, safety, tradeoffs.
- `debugger`: safe diagnostic steps and allowlisted Cargo commands.
- `research-outline`: research questions, source types, verification notes, citation placeholders.
- `creative`, `summarizer`, `safety-review`, `product-manager`, and `coach`: structured deterministic helper modes.

Modes shape the response format. They do not enable web access, LLM calls, unrestricted shell execution, or outside-sandbox file access.
