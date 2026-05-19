# Windows App

On Windows, launch the native app from the repository root:

```powershell
cargo run -- gui
```

Build the executable:

```powershell
cargo build --release
```

The release binary is written to:

```text
target/release/onyx_brain.exe
```

## Functional UI wiring

The Windows GUI uses `eframe/egui` and calls the safe Rust `AppApi` layer. It supports chat, bounded autonomy, creative planning, local search, library browsing, settings persistence, safety tools, system status, and a `Ctrl+K` command palette.

Safety notes:

- No default network access is introduced.
- No unrestricted shell execution is introduced.
- Scheduled background jobs are disabled in v0.0.4.
- Connectors are disabled in v0.0.4.
- Errors are shown with friendly summaries and Doctor guidance when local state repair may help.
