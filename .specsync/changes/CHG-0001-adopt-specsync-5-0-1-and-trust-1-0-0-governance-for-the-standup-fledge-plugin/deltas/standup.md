## MODIFIED
### SPEC SECTION Change Log
| Version | Date | Changes |
|---------|------|---------|
| 1 | 2026-07-12 | Document existing local, multi-repository, GitHub-wide, raw, and narrated standup behavior for SpecSync 5 adoption. |
| 2 | 2026-07-13 | Reconciled existing prompt, scope, and date documentation with stable requirement IDs for SpecSync 5.0.1 governance; runtime behavior is unchanged. |

### REQUIREMENT REQ-standup-001
Single-repository scope SHALL aggregate the selected time window and author, with optional diff statistics.

Acceptance Criteria
- Existing unit tests verify the prompt includes the selected scope and window, includes author context when supplied, and includes diff statistics only when present.

