---
spec: standup.spec.md
---

## Context

This Rust replacement removes the previous Bash script's Python and jq requirements while keeping the same Fledge Ask composition model.

## Related Modules

- Git and GitHub CLI commit history.
- Fledge Ask and AI configuration.

## Design Decisions

- Support raw output so aggregation remains useful and testable without AI.
- Use sorted repository grouping for reproducible prompts.
