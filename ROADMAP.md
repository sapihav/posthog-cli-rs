# Roadmap

Forward-looking plan for `posthog-cli-rs`. North star: feel like a frictionless extension of PostHog's official tooling, so PostHog itself doesn't need to invest. Near-term focus: **AI-agent UX polish** (the differentiator vs the MCP).

> Format: each milestone is one PR, ~150–300 lines of app code, in strict order. Future agents picking this up should ship them sequentially, not batch them.

---

## Rust port status

This crate is a Rust port of the TypeScript [posthog-cli](https://github.com/sapihav/posthog-cli) (currently at v0.1.4 on npm). The TS project has shipped milestones M1 and M2 and has M3 in progress. The Rust port restarts the milestone ladder from scratch — each Rust milestone below must be shipped as its own PR before the next starts.

- **M0 — Scaffold** ✅ shipped. Project skeleton, docs ported from TS, `cargo build` green.
- **M1 — Config + login** ✅ shipped. `config set/show`, interactive `login` with browser open + org/project fetch, structured `PostHogError`, stdout-JSON / stderr-error output helpers.
- **M2 — API client + feature flags** ⏳ next.
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

---

## MCP parity plan (M6 → M16) — mirrors TS `posthog-cli` M4 → M14

After M5 reaches TS v0.1.4 + M3 parity, the Rust crate must mirror the TS CLI's MCP-parity roadmap milestone-for-milestone. Same ordering, same acceptance criteria, same JSON output shapes — only the implementation language differs.

**Hard rule:** no Rust milestone may ship ahead of the corresponding TS milestone being merged and stable. If a Rust contributor wants to unblock a later milestone, the fix is to ship the TS version first.

Rust milestone → TS milestone map:

| Rust | TS | Scope |
|---|---|---|
| **M6** | M4 | Contract flags hardening (`--quiet`, `--verbose`, `--out`, `--limit`, stdin `-`, env-only auth mode) |
| **M7** | M5 | Feature flags full parity (copy, dependents, status, blast-radius, evaluation-reasons, scheduled changes) |
| **M8** | M6 | HogQL runner v2 (`query run --params`, `query nl`, `query saved list/get/run`) |
| **M9** | M7 | Experiments full CRUD (create, update, delete, archive, reset, resume, ship-variant) |
| **M10** | M8 | Insights + dashboards write surface (create/update/delete + add-insight + reorder-tiles + run) |
| **M11** | M9 | Persons + cohorts (including static cohort add/remove) |
| **M12** | M10 | Surveys (CRUD + per-survey + global stats) |
| **M13** | M11 | Error tracking (issues list/get/update/merge/split, rules, query) |
| **M14** | M12 | Taxonomy (actions, annotations, event-definitions, property-definitions) |
| **M15** | M13 | Session replays + playlists |
| **M16** | M14 | Query wrappers (funnel/trends/lifecycle/retention/paths/stickiness) |

For the detailed scope and acceptance criteria of each Rust milestone, consult the corresponding section in `/Users/vlads/src/clis/posthog-cli/ROADMAP.md` — **this crate treats the TS ROADMAP as the spec**. When the TS spec changes, this file's map must be updated in the same PR.

### Parity test

Starting at M6, every milestone PR must include a `tests/parity_<domain>.rs` integration test that:
1. Runs a fixed scenario via the TS CLI (`npx posthog@0.1.4 …` or pinned version) and captures its JSON output.
2. Runs the same scenario via the Rust CLI and captures its JSON output.
3. Asserts equality (modulo `elapsed_ms` and any documented Rust-specific fields).

The TS CLI is the reference implementation; the Rust crate is the mirror.

---

## Long tail MCP parity (stretch)

Same deferred list as TS ROADMAP §"Deferred — long tail MCP parity". Only tackle when concrete demand surfaces:

1. LLM analytics + evaluations (12 tools)
2. Data warehouse views + endpoints (16 tools)
3. CDP functions + templates (8 tools)
4. Notebooks, alerts, subscriptions, proxies, integrations, workflows, conversations, roles, org/project switching, early-access features, prompts (~50 tools combined)
5. `docs-search` and `entity-search` — standalone, low-effort, high-value; ship any time.

---

## Deferred (non-parity infrastructure)

1. **Trust signals** — GitHub Actions CI (build + test on stable + beta), `CHANGELOG.md`, `CONTRIBUTING.md`, issue/PR templates, semver discipline, crates.io publish automation.
2. **Multi-project switching** — `posthog use <project>` and equivalent for orgs.
3. **Discoverability** — community post in `posthog/posthog`, docs PR mentioning the CLI under tooling, crates.io badges, crates.io publish.
4. **Claude Code integration kit** — slash commands, CLAUDE.md snippet, recipes. *Optional, opt-in only.*
5. **Self-telemetry** (opt-in, off by default) — emit anonymous usage events to PostHog itself. Eat your own dogfood; builds the case to PostHog that real people use it.

---

## Principles

- **One milestone = one PR**, ~500 lines of app code max. Refuse to batch; review and merge each before starting the next.
- **JSON contract is sacred:** stdout always valid JSON; stderr structured (after M5); never break documented shapes without a major version bump.
- **YAGNI:** no plugin system, no TUI, no nice-to-have features without an explicit ask.

---

## References

- TS original: https://github.com/sapihav/posthog-cli
- [Rewrite Your CLI for AI Agents — Justin Poehnelt](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/)
- [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)
- [Linux CLI apps should have a --json flag](https://thomashunter.name/posts/2012-06-06-linux-cli-apps-should-have-a-json-flag)
