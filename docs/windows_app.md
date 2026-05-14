# Windows Desktop App

Onyx Brain builds a native Windows desktop executable through Rust and `eframe`/`egui`.

Run during development:

```bash
cargo run -- gui
```

Build release:

```bash
cargo build --release
```

Release executable:

```text
target/release/onyx_brain.exe
```

The official GUI path is `cargo run -- gui`.

Window defaults:

- Title: `Onyx Brain`
- Initial size: `1280x800`
- Minimum size: `1000x700`
- Theme: dark

Manual verification:

- Launch `cargo run -- gui`.
- Confirm a native desktop window opens.
- Confirm the native window opens directly.
- Switch between sidebar views.
- Send a chat message with the Chat view.
- Open Safety and System views and confirm status panels render.
- Build `cargo build --release` and confirm `target/release/onyx_brain.exe` exists.
