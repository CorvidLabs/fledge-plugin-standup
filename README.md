# fledge-plugin-standup

Generate a paste-ready Markdown standup post from your recent git commits, narrated by whatever LLM you've configured via [`fledge ai use`](https://github.com/CorvidLabs/fledge). Works on a single repo, a list of repos, an entire dev directory, or every repo on GitHub you've touched.

```
$ fledge standup --gh --since "1 week ago"
## Yesterday
- Prepared v0.15.3 release including FLEDGE_PLUGIN_DIR support and plugin.toml as a version source (fledge)
- Re-absorbed templates-remote and doctor plugins back into core (fledge)
- Built standup and roast plugins from scratch (fledge-plugin-standup, fledge-plugin-roast)
- Hand-authored 33 spec stubs and tightened entitlements (Mono)

## Today
- (inferred) Cut v0.15.3 and watch the post-release formula PR land

## Blockers
None
```

## Install

```bash
fledge plugins install CorvidLabs/fledge-plugin-standup
```

Make sure an AI provider is configured:

```bash
fledge ai use                       # interactive picker
# or
fledge ai use ollama gpt-oss:120b-cloud
```

## Modes

### Single repo (default)

```bash
fledge standup                              # last 24h, everyone in the repo
fledge standup --me                         # just your commits (matches by email)
fledge standup --since "1 week ago" --me
fledge standup --author "Leif"              # someone else
```

`--me` matches by `git config user.email` first (stable across squash-merges), falling back to `user.name`.

### Multiple repos

```bash
# Explicit list
fledge standup --repos ~/dev/foo,~/dev/bar,~/dev/fledge --me --since "1 week ago"

# Auto-discover one level deep
fledge standup --repo-dir ~/Development --me --since "1 week ago"
```

Output groups commits under `## <repo-name>` headers. The LLM annotates each "Yesterday" bullet with the repo it belongs to: `- Bumped tokei to library mode (fledge-plugin-metrics)`.

### GitHub-wide

```bash
# Every commit you pushed anywhere on GitHub (requires `gh auth`)
fledge standup --gh --since "1 week ago"

# A specific GitHub user (e.g. you on a different machine, a teammate)
fledge standup --gh --gh-user 0xLeif --since yesterday
```

`--gh` shells out to `gh search commits` so it doesn't need local clones. Sees only commits you pushed to GitHub-visible repos. Requires `gh`, `jq`, and `python3` (for date math). Only commits visible to your authenticated user count, so this won't surface private-repo work you don't have access to.

### Output controls

```bash
fledge standup --raw                        # plain aggregated log, skip the LLM
fledge standup --include-diff               # adds shortstat per commit (single-repo only)
fledge standup --show-prompt                # debug — prints what would be sent
```

### Provider override

```bash
fledge standup --me -- --provider ollama --model qwen3-coder:480b-cloud
```

Anything after `--` is forwarded to `fledge ask`.

## How it works

~280 lines of bash that:

1. Resolves the active mode (single / `--repos` / `--repo-dir` / `--gh`) and aggregates commits into a single log with `## <repo>` headers when multi-repo
2. Builds a structured prompt with three required sections (`## Yesterday`, `## Today`, `## Blockers`)
3. Pipes it through `fledge ask --no-spec-index "$PROMPT"` so it inherits your provider, model, timeouts, and rate-limit config

The model is told to:

- Write past-tense bullets for "Yesterday" using the commit subjects (stripping `feat:`/`fix:` prefixes)
- Annotate multi-repo bullets with the repo name in parentheses
- Infer 1–3 "Today" items, prefixed with `(inferred)` so you can edit
- Leave "Blockers" as `None` unless the log explicitly mentions one

## Caveats

- `--gh` is bounded to 200 commits per call (the `gh search commits` `--limit` cap). For long windows, narrow with `--gh-user` or `--since`.
- `--include-diff` is single-repo only. Multi-repo diff stats would blow the token budget.
- Output quality scales with model — cloud models produce cleaner narration than local ones.
- Two standups for the same window produce structurally identical but stylistically different prose. That's fine.

## License

MIT
