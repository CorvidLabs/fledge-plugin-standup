---
change: CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-governance-for-the-standup-fledge-plugin
artifact: context
---

# Context

Standup is a cross-platform Rust Fledge plugin that aggregates Git activity from the current repository, multiple repositories, or GitHub-wide history. It can emit deterministic raw output or delegate narration to Fledge Ask. The repository already validates Linux, macOS, and Windows builds and publishes documentation through Pages, but it does not yet use the SpecSync 5 lifecycle or the unified Trust gate.
