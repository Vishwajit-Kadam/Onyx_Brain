# Conversation

The v0.0.3 conversation layer is deterministic and disk-backed. It uses mode-specific response templates, intent detection, response quality checks, and safety filters.

```bash
cargo run -- chat
cargo run -- chat "Hello Onyx, what can you do?"
```

Interactive chat supports `/help`, `/mode <name>`, `/summary`, `/save`, and `/exit`.

Sessions are stored in `data/conversations/`. Summaries are stored in `data/conversation_memory/`. Transcript exports are written under `sandbox/exports/conversations/`.

This layer is not an LLM and does not claim real understanding.
