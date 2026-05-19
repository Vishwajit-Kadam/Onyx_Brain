# Changelog

## v0.0.4

### Functional UI wiring

- Added a Rust-native `eframe/egui` GUI launched with `cargo run -- gui`.
- Added typed `AppApi` modules for friendly errors, GUI-facing models, and safe action wiring.
- Added persistent GUI settings at `data/config/gui_settings.json`, including Light/Dark/Auto theme mode and default personality.
- Added local GUI search across tasks, sessions, artifacts, artifact packs, memories, and projects.
- Wired Home, Chat, Autonomy, Creative Studio, Library, Projects, Tasks, Artifacts, Memory, Safety, System, Settings, and `Ctrl+K` command palette controls.
- Added clear disabled states for Scheduled background jobs, external connectors, favorites, and deep artifact inspection gaps.
- Added GUI/AppApi tests for settings recovery, search, status, safety, task creation, chat, artifacts, and Doctor.
