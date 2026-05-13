# Task Graphs

Onyx Brain v0.0.2 can represent autonomous work as a dependency-aware task graph. The graph is disk-backed under the session folder and indexed lightly.

Use:

```bash
cargo run -- task-graph latest
```

Task graphs are synchronous and bounded. They do not create background workers and do not bypass autonomy limits.
