---
spec: standup.spec.md
---

## User Stories

- As a developer, I want a concise standup assembled from my recent activity across the repositories I select.

## Acceptance Criteria

### REQ-standup-001

Single-repository scope SHALL aggregate the selected time window and author, with optional diff statistics.

Acceptance Criteria
- Existing unit tests verify the prompt includes the selected scope and window, includes author context when supplied, and includes diff statistics only when present.

### REQ-standup-002

Multi-repository scope SHALL validate repositories, skip invalid entries with diagnostics, and group non-empty logs deterministically.

### REQ-standup-003

GitHub-wide scope SHALL use authenticated commit search, supported date conversion, author selection, and repository grouping.

### REQ-standup-004

Raw mode SHALL print aggregated activity without AI, while narrated mode SHALL delegate through Fledge Ask with spec indexing disabled.

### REQ-standup-005

Empty activity SHALL exit successfully without invoking AI, and prerequisite failures SHALL be explicit.

## Constraints

- GitHub search is limited by authentication, visibility, and the committed result limit.

## Out of Scope

- Calendar scheduling, issue/project aggregation, automatic posting, and AI provider configuration.
