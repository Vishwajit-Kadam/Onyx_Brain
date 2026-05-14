# Native GUI

Onyx Brain's official GUI is a Rust-native `eframe`/`egui` desktop app.

```bash
cargo run -- gui
```

Build the Windows executable with:

```bash
cargo build --release
```

The executable is:

```text
target/release/onyx_brain.exe
```

No JavaScript toolchain or local web server is required for the native GUI.

The app opens a native desktop window titled `Onyx Brain` with a dark shell, left sidebar, top status bar, main view area, and inspector panel. Implemented views include Chat, Autonomy, Creative Studio, Tasks, Artifacts, Memory, Safety, System, and Settings.

GUI actions call the Rust `AppApi` boundary. The interface preserves bounded autonomy, sandbox-first wording, allowlisted commands, and recovery/doctor checks. It does not expose a control to disable safety.

Legacy web GUI files have been removed from the official project path.
