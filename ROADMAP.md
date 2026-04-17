# Roadmap

Forward-looking plan for `posthog-cli-rs`. North star: feel like a frictionless extension of PostHog's official tooling, so PostHog itself doesn't need to invest. Near-term focus: **AI-agent UX polish** (the differentiator vs the MCP).

> Format: each milestone is one PR, ~150–300 lines of app code, in strict order. Future agents picking this up should ship them sequentially, not batch them.

---

## Rust port status

This crate is a Rust port of the TypeScript [posthog-cli](https://github.com/sapihav/poshog-cli) (currently at v0.1.4 on npm). The TS project has shipped milestones M1 and M2 and has M3 in progress. The Rust port restarts the milestone ladder from scratch — each Rust milestone below must be shipped as its own PR before the next starts.

- **M0 — Scaffold** ✅ shipped. Project skeleton, docs ported from TS, `cargo build` green.
- **M1 — Config + login** ⏳ next.
- **M2 — API client + feature flags** — pending.
- **M3 — Experiments, insights, dashboards, query** — pending.
- **M4 — Self-describing CLI (`schema` + `--fields`)** — pending.
- **M5 — Structured errors + `--dry-run`** — pending.

After M5 the Rust crate reaches parity with TS v0.1.4 + M3 (the currently in-progress TS milestone).

---

## Status

- Not yet published to crates.io. Target crate name: `posthog-cli-rs`. Binary: `posthog`.
- Official `@posthog/cli` (Rust, by PostHog Inc) is largely abandoned and installs as `posthog-cli` on crates.io. To avoid collision we publish as `posthog-cli-rs`; the installed binary is still `posthog`.

---

## Milestone 1 — Config + login

Lay the foundation for all subsequent commands: config read/write, structured output, and the interactive login flow.

- `src/config.rs` — env > local `.posthog.json` (projectId only) > `~/.config/posthog/config.json`
- `src/output.rs` — `output_json`, `output_error`, `OutputOptions` (pretty, fields — fields wired up in M4)
- `src/errors.rs` — `PostHogError` enum, error codes, `classify_http_status`
- `src/commands/config.rs` — `posthog config set / show`
- `src/commands/login.rs` — region pick → browser open → masked key paste → org/project fetch → save global config

**Files:** `src/config.rs`, `src/output.rs`, `src/errors.rs`, `src/commands/{config,login}.rs`, `src/main.rs` (wire up), `tests/{config,login,output}.rs`

**Verify:** `cargo run -- login` walks through region/key/project and writes `~/.config/posthog/config.json`; `cargo run -- config show` prints masked config as JSON.

---

## Milestone 2 — API client + feature flags

Implement the shared HTTP client and the first resource (feature flags, the highest-leverage one for AI agents).

- `src/client.rs` — `PostHogClient` with Bearer auth, 429 retry w/ exponential backoff (max 3), generic `list`/`list_all`/`get`/`create`/`update`/`delete`, HogQL `query`
- `src/commands/flags.rs` — `list`, `get`, `create`, `update`, `enable`, `disable`, `delete` (with key-or-id resolution)

**Files:** `src/client.rs`, `src/commands/flags.rs`, `src/main.rs`, `tests/client.rs`, `tests/flags.rs`

**Verify:** against a real project, `flags list` returns array; `flags create --key x --name X --rollout 50`, `enable x`, `delete x` round-trips.

---

## Milestone 3 — Experiments, insights, dashboards, query

Fill out the read/mutate surface for the remaining resources using the M2 client.

- `src/commands/experiments.rs` — `list`, `get`, `results`, `launch`, `pause`, `end`
- `src/commands/insights.rs` — `list`, `get`
- `src/commands/dashboards.rs` — `list`, `get`
- `src/commands/query.rs` — `posthog query "<hogql>"`

**Verify:** `posthog query "SELECT event, count() FROM events GROUP BY event LIMIT 5"` returns rows; `experiments results <id>` returns payload.

---

## Milestone 4 — Self-describing CLI (`schema` + `--fields`)

Make the CLI introspectable at runtime so agents don't have to scrape `--help`.

- `posthog schema` — emit full command tree as JSON (commands, options, arguments, output shapes)
- `--help --json` at every level — same data scoped to the current subcommand
- `--fields <a,b,c>` global flag for list/get commands — post-filters response objects to only the listed keys
- Pointer to `posthog schema` from `posthog --help`
- `OUTPUT.md` stays in sync with `OUTPUT_SHAPES` in `src/schema.rs`

**Verify:** `posthog schema | jq '.commands | length'` returns the command count; `flags list --fields key,active` returns objects with only those two keys.

---

## Milestone 5 — Structured errors + `--dry-run`

Make failures machine-readable and add a safety rail for mutations.

- stderr emits structured JSON: `{ "error": { "message", "code", "hint?", "docs_url?" } }`
- Error codes: `AUTH_MISSING`, `AUTH_INVALID`, `NOT_FOUND`, `RATE_LIMITED`, `API_ERROR`, `VALIDATION`
- `--dry-run` on every mutating command — prints the planned API request, no network call, exit 0

**Verify:** bad key emits `{"error":{"code":"AUTH_INVALID",...}}`; `posthog flags create --key x --name X --dry-run` prints request payload, exit 0.

---

## Deferred (next rounds, in rough order)

1. **Resource coverage parity with the MCP** — `persons`, `cohorts`, `surveys`, `error tracking`, `annotations`, `actions`, `event definitions`, `session recordings`. Each is small (one resource per PR) and uses the existing generic CRUD client.
2. **Trust signals** — GitHub Actions CI (build + test on stable + beta), `CHANGELOG.md`, `CONTRIBUTING.md`, issue/PR templates, semver discipline, crates.io publish automation.
3. **Multi-project switching** — `posthog use <project>` and equivalent for orgs.
4. **Discoverability** — community post in `posthog/posthog`, docs PR mentioning the CLI under tooling, crates.io badges, crates.io publish.
5. **Claude Code integration kit** — slash commands, CLAUDE.md snippet, recipes. *Optional, opt-in only.* Revisit after M1–M5 stabilize the agent surface.
6. **Self-telemetry** (opt-in, off by default) — emit anonymous usage events to PostHog itself. Eat your own dogfood; builds the case to PostHog that real people use it.

---

## Principles

- **One milestone = one PR**, ~500 lines of app code max. Refuse to batch; review and merge each before starting the next.
- **JSON contract is sacred:** stdout always valid JSON; stderr structured (after M5); never break documented shapes without a major version bump.
- **YAGNI:** no plugin system, no TUI, no nice-to-have features without an explicit ask.

---

## References

- TS original: https://github.com/sapihav/poshog-cli
- [Rewrite Your CLI for AI Agents — Justin Poehnelt](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/)
- [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)
- [Linux CLI apps should have a --json flag](https://thomashunter.name/posts/2012-06-06-linux-cli-apps-should-have-a-json-flag)
