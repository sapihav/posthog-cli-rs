# posthog-cli-rs

PostHog CLI (`posthog` binary) — manage PostHog projects from the terminal. Rust port of the TypeScript [posthog-cli](https://github.com/sapihav/poshog-cli).
MCPs are not a good fit for AI agent tooling — a CLI is more predictable and composable.

## Sources of truth

- `DESIGN.md` — full spec: stack, commands, API client contract, auth.
- `ROADMAP.md` — current milestone status and what ships next. **Read this before starting new work.**
- `OUTPUT.md` — per-command JSON output shapes (mirror of `OUTPUT_SHAPES` in `src/schema.rs`).

## Dev commands

```
cargo build             # compile
cargo run -- <cmd>      # run without installing
cargo test              # run tests
cargo install --path .  # install `posthog` binary locally
```

Runtime introspection: `cargo run -- schema` returns the full command tree as JSON. Prefer this over re-reading source when answering "what does command X return?".

## Auth

`POSTHOG_API_KEY` and `POSTHOG_PROJECT_ID` env vars, or run `posthog config set`.
API key format: `phx_...` — personal API keys only (not project tokens).

## Architecture

```
src/
  main.rs         entry point, clap setup
  client.rs       API client — all HTTP calls go here
  config.rs       config read/write (~/.config/posthog/config.json)
  output.rs       stdout/stderr helpers
  errors.rs       PostHogError enum, error codes
  schema.rs       OUTPUT_SHAPES + schema command
  commands/       one file per subcommand group
```

## Output contract

- **stdout**: always valid JSON
- **stderr**: human-readable errors + exit 1
- `--pretty`: indented JSON for humans

## Conventions

- **Commits**: conventional prefixes — `feat:`, `fix:`, `chore:` (match existing `git log`).
- **One milestone = one PR**, ≤500 LoC app code. Refuse to batch milestones; push back once, comply only if user insists.
- **When changing a command's JSON output**, update **both** `OUTPUT_SHAPES` in `src/schema.rs` AND `OUTPUT.md`. They must not drift.
- **Dogfood the CLI**: for ad-hoc verification during dev, prefer `cargo run -- <cmd>` over the PostHog MCP — this CLI is the agent-facing surface we're building.
- **Parity with TS original**: when in doubt about behaviour, consult the TS source at `/Users/vlads/src/poshog-cli` (if available) or https://github.com/sapihav/poshog-cli. JSON shapes, error codes, and flag semantics must match.

## When to delegate to subagents

Selective, not default. The codebase is small enough to fit in main context; subagent output comes back as summaries and loses fidelity.

- **`Explore`** — open-ended "where/how" searches across the repo. Protects main context on wide investigation.
- **`code-reviewer`** — **mandatory before opening a PR for a milestone.** Independent eyes catch scope creep; aligns with the ≤500 LoC rule.
- **`senior-backend-engineer`** — multi-file milestones touching ≥3 files (e.g., adding a new resource: client + command + schema + OUTPUT.md + tests).
- **Skip subagents for**: typo fixes, one-line changes, renames, trivial test tweaks, single-file edits — main agent is faster and the diff is ground truth.
