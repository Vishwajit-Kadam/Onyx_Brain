# Native GUI

Onyx Brain v0.0.4 includes a Rust-native `eframe/egui` desktop GUI.

Run it with:

```bash
cargo run -- gui
```

The GUI is a safe front-end over `AppApi`. It preserves the existing CLI and does not use Tauri, HTML, CSS, JavaScript, unrestricted shell execution, unrestricted file access, default network access, or hidden background workers.

## Functional UI wiring

Working views:

- Home / New task: sends chat, launches bounded autonomy for build/create/plan/generate prompts, and launches creative planning for creative/video/movie prompts.
- Chat: appends user and Onyx messages using the existing deterministic conversation layer.
- Search: searches local tasks, artifacts, sessions, memories, and projects only.
- Library: lists generated artifacts, packs, and sessions with local filtering and grid/list layout.
- Projects and Tasks: show backend projects/goals plus safe GUI-created task entries.
- Settings: persists theme and personality defaults in `data/config/gui_settings.json`.
- Safety and System: run Doctor, Regression Check, Maintain, status refresh, and workspace inspection.
- Command palette: `Ctrl+K` opens quick commands.

Intentionally disabled:

- Scheduled background jobs: disabled in v0.0.4 because Onyx does not run hidden background work.
- External connectors: disabled in v0.0.4.
- Favorites: marked as not implemented.

See `docs/gui_functionality_audit.md` for the full control-by-control audit.
