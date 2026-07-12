---
spec: standup.spec.md
---

## Test Plan

### Unit Tests

- Time-window conversion and ISO/local date formatting.
- Repository labels, tilde expansion, prompt construction, and Git parsing.

### Integration Tests

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build --release`
- Verify help without invoking GitHub or AI.
