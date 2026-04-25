# Changelog

## [v0.3.1] - 2026-04-25

### Fixed

- Each commit line now carries its own date (`YYYY-MM-DD` in local time) and the prompt tells the model what "today" is, so commits made today actually land under `## Today` instead of being lumped under `## Yesterday`.
- Tightened the "Blockers" rule to only trigger on `wip:` / `blocked:` / `revert:` prefixes (no speculation).

## [v0.3.0] - 2026-04-25

### Changed

- Rewritten in Rust. Drops the runtime dependencies on `python3` (date math now in-process via the `time` crate) and `jq` (JSON parsing via `serde_json`). `git` is always required; `gh` is still required for `--gh`.
- `fledge plugins install` auto-detects `Cargo.toml` and runs `cargo build --release` — no separate toolchain steps.
- Binary moved from `bin/standup` (bash) to `target/release/fledge-standup`.

## [v0.2.0] - 2026-04-25

### Added

- `--repos a,b,c`, `--repo-dir <path>`, and `--gh` modes for multi-project standups (bbece2d).

### Fixed

- `--me` matches by `git config user.email` (stable across squash-merges) before falling back to `user.name` (ce8e46c).

## [v0.1.1] - 2026-04-25

### Fixed

- Drop default author filter; add `--me` opt-in; hint other authors on empty result (802a1a3).
- Handle empty passthrough array under `set -u` (d63f551).
