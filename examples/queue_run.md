# Queue Run Example

```bash
cargo run -- init
cargo run -- queue-run "Create a learning pack about brain-inspired AI || Create a documentation pack for Onyx Brain commands || Run doctor"
```

Queue mode runs goals sequentially in the current command. It is not a background daemon and stops if a severe safety issue occurs.
