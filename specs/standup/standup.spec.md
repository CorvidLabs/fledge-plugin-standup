---
module: standup
version: 1
status: active
files:
  - src/main.rs

db_tables: []
depends_on: []
---

# Standup

## Purpose

Aggregate recent Git activity from one repository, multiple local repositories, or GitHub-wide search and produce raw logs or an AI-narrated Markdown standup through Fledge Ask.

## Public API

| Surface | Behavior |
|---------|----------|
| single repository | Summarize the current Git repository and optionally include diff statistics. |
| multiple repositories | Aggregate explicit or auto-discovered local Git repositories. |
| GitHub-wide | Search authenticated GitHub-visible commits and group them by repository. |
| raw | Print the aggregated log without invoking AI. |
| narrated | Build the standup prompt and delegate it to Fledge Ask. |

## Invariants

1. Scope-selection flags for explicit repositories, repository directory, and GitHub-wide search are mutually exclusive.
2. Author filtering applies consistently to local Git history and the selected GitHub author.
3. Diff statistics are included only for a single local repository.
4. Multi-repository output is deterministically grouped and labeled.
5. GitHub commits are grouped by full repository name and dates are converted to local time.
6. Empty activity exits successfully without invoking AI.
7. Raw mode never invokes Fledge Ask.
8. Narrated mode delegates through `fledge ask --no-spec-index` and propagates its exit status.

## Behavioral Examples

```
Given recent commits in the selected scope
When the developer requests a narrated standup
Then the plugin aggregates deterministic commit context and delegates a Markdown standup prompt to Fledge Ask
```

## Error Cases

| Error | When | Behavior |
|-------|------|----------|
| Not a Git repository | Single scope runs outside Git | Explain alternate scope flags and exit non-zero. |
| Invalid repository directory | Auto-discovery path is not a directory | Report the invalid path. |
| No repositories | Explicit or discovered multi-scope is empty | Report that no repositories can be scanned. |
| GitHub CLI unavailable | GitHub-wide mode cannot find gh | Report authentication/tool prerequisite. |
| Unsupported since value | GitHub mode cannot convert the time window | Request a supported relative or ISO date. |
| Fledge Ask unavailable | Narration cannot launch | Surface the delegated command failure. |

## Dependencies

- Rust 1.89 or later
- Git for local scopes
- Authenticated GitHub CLI for GitHub-wide scope
- Fledge Ask and a configured AI provider for narration

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1 | 2026-07-12 | Document existing local, multi-repository, GitHub-wide, raw, and narrated standup behavior for SpecSync 5 adoption. |
