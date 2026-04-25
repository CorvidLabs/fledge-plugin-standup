# fledge-plugin-standup

Generate a paste-ready Markdown standup post from your recent git commits, narrated by whatever LLM you've configured via [`fledge ai use`](https://github.com/CorvidLabs/fledge).

```
$ fledge standup --since "yesterday"
## Yesterday
- Re-absorbed templates-remote and doctor plugins back into core (#260)
- Synced the Nix flake and Homebrew formula with the current version (#259)
- Trimmed `DEFAULT_PLUGINS` from 5 entries to 3

## Today
- (inferred) Cut the v0.15.2 release with the cleanup
- (inferred) Backfill any docs that still reference the dropped plugins

## Blockers
None.
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

## Usage

```bash
fledge standup                              # last 24h, current user
fledge standup --since "1 week ago"         # weekly recap
fledge standup --since yesterday
fledge standup --since "2026-04-01"
fledge standup --author "Leif"              # someone else's standup
fledge standup --include-diff               # richer narration (slower)
fledge standup --raw                        # plain git log, no LLM
```

### Provider override per call

```bash
fledge standup -- --provider ollama --model qwen3-coder:480b-cloud
```

Anything after `--` is forwarded to `fledge ask`.

### Debugging

```bash
fledge standup --show-prompt                # prints the prompt that was sent
```

## How it works

~120 lines of bash that:

1. Pulls commits via `git log --since=<window> [--author=<name>]`
2. Builds a structured prompt with three required sections (`## Yesterday`, `## Today`, `## Blockers`)
3. Pipes it through `fledge ask --no-spec-index "$PROMPT"` so it inherits your provider, model, timeouts, and rate-limit config

The model is told to:

- Write past-tense bullets for "Yesterday" using the commit subjects (stripping `feat:`/`fix:` prefixes)
- Infer 1–3 "Today" items and mark them with `(inferred)` so you can edit
- Leave "Blockers" as `None` unless the log explicitly mentions one

## Caveats

- Output quality is bounded by your LLM. Local Ollama models work but cloud models give cleaner narration.
- `--include-diff` adds shortstat lines per commit. Big windows can hit token limits — use `--since` to scope.
- Two daily standups in a row from the same window will produce slightly different output (LLMs gonna LLM). The structure is stable; the prose isn't.

## License

MIT
