---
change: CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-governance-for-the-standup-fledge-plugin
artifact: testing
---

# Testing

Run `specsync check --strict --require-coverage 100 --force`, `specsync agents status`, `fledge trust doctor`, and `fledge lanes run verify`. The native lane must pass Rustfmt, Clippy with warnings denied, all 27 tests, a release build, and the compiled plugin help smoke test. Hosted checks must retain the existing Linux, macOS, and Windows coverage.
