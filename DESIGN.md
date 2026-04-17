# PostHog CLI (`posthog`) — Design Document

## Context
PostHog provides a web UI, SDK, API, and MCP server but no proper CLI. The existing official `@posthog/cli` (Rust) is very limited (login, query, sourcemap only). MCPs in general are not a good fit for AI agent tooling — terminal CLI tools are more predictable, composable, and debuggable for agents like Claude Code. Goal: a Rust CLI (`posthog`) that Claude Code can invoke via bash to manage PostHog projects — JSON output by default, core operations only.

This project is a Rust port of the TypeScript [posthog-cli](https://github.com/sapihav/poshog-cli). Feature surface and JSON output shapes are kept 1:1 with the TS original; only the implementation stack differs.

---

## Stack

- **Language**: Rust (edition 2021)
- **CLI framework**: `clap` v4 (derive macros)
- **HTTP**: `reqwest` (`rustls-tls`, `json`, `gzip`) — no openssl build dependency
- **Async runtime**: `tokio` (`rt-multi-thread`, `macros`)
- **JSON**: `serde` + `serde_json`
- **Errors**: `thiserror` — typed error enum with structured payloads
- **Config paths**: `dirs`
- **Browser open**: `open`
- **Masked input**: `rpassword`
- **Binary name**: `posthog`
- **Crate name**: `posthog-cli-rs`

---

## Project Structure

```
posthog-cli-rs/
├── Cargo.toml
├── rustfmt.toml
├── src/
│   ├── main.rs              # Entry: clap root, subcommand wiring
│   ├── client.rs            # PostHog API client (auth, fetch, retry on 429)
│   ├── config.rs            # Config read/write (~/.config/posthog/config.json)
│   ├── output.rs            # stdout JSON / stderr errors helper
│   ├── errors.rs            # PostHogError enum, error codes, classify_http_status
│   ├── schema.rs            # OUTPUT_SHAPES + schema command
│   └── commands/
│       ├── mod.rs
│       ├── login.rs         # posthog login (interactive)
│       ├── config.rs        # posthog config set / show
│       ├── flags.rs         # posthog flags *
│       ├── experiments.rs   # posthog experiments *
│       ├── insights.rs      # posthog insights *
│       ├── dashboards.rs    # posthog dashboards *
│       └── query.rs         # posthog query <hogql>
└── tests/                   # integration tests
```

---

## Auth & Config

Priority (highest first):
1. Env vars: `POSTHOG_API_KEY`, `POSTHOG_PROJECT_ID`, `POSTHOG_HOST`
2. Local project config: `.posthog.json` in cwd (only `projectId` is honoured — `apiKey` and `host` are ignored for security, to prevent credential theft via malicious repos)
3. Global config: `~/.config/posthog/config.json`

Config shape:
```json
{ "apiKey": "phx_...", "projectId": "12345", "host": "https://us.posthog.com" }
```

Allowed hosts: `https://us.posthog.com`, `https://eu.posthog.com`.

---

## Commands (MVP Scope)

```
posthog login

posthog config set --api-key <key> --project-id <id> [--host <url>]
posthog config show

posthog flags list [--search <text>] [--active] [--all]
posthog flags get <key-or-id>
posthog flags create --key <key> --name <name> [--rollout <0-100>]
posthog flags update <key-or-id> [options]
posthog flags enable <key-or-id>
posthog flags disable <key-or-id>
posthog flags delete <key-or-id>

posthog experiments list [--status draft|running|complete]
posthog experiments get <id>
posthog experiments results <id>
posthog experiments launch <id>
posthog experiments pause <id>
posthog experiments end <id>

posthog insights list [--search <text>]
posthog insights get <id>

posthog dashboards list
posthog dashboards get <id>

posthog query "<hogql>"

posthog schema
```

### Global flags

- `--pretty` — indented JSON output
- `--json` — with `--help`, emits the CLI schema for the current subcommand
- `--fields <csv>` — post-filter object/array-of-object outputs to the listed keys
- `--dry-run` — on mutating commands: print the planned API request as JSON and exit without sending it

---

## API Client (`src/client.rs`)

- Base URL: `{host}/api/projects/{projectId}/` (or `/api/environments/{projectId}/` for newer endpoints)
- Auth header: `Authorization: Bearer {apiKey}`
- Retry on 429 with exponential backoff (max 3 retries)
- Return typed errors; all caught in command layer and printed to stderr as structured JSON with exit code 1
- Pagination: auto-fetch next page when `--all` flag passed (default: first page, limit 100)

---

## Output Conventions

- **stdout**: Always valid JSON (either object or array)
- **stderr**: Structured JSON error `{"error":{"message","code","hint?","docs_url?"}}` + exit 1
- No pretty-printing by default (compact JSON for AI parsing)
- `--pretty` flag for human-readable indented output

Error codes: `AUTH_MISSING`, `AUTH_INVALID`, `NOT_FOUND`, `RATE_LIMITED`, `API_ERROR`, `VALIDATION`.

---

## Implementation Milestones

See `ROADMAP.md` for current status. Broad sequence:

1. **Scaffold** — `Cargo.toml`, `src/main.rs` entry, ported docs, `.claude/settings.json`
2. **Config + login** — `src/config.rs` + `posthog config set/show` + `posthog login` (interactive, browser open, masked input)
3. **API client + feature flags** — `src/client.rs` with auth, fetch, 429 retry, pagination; `posthog flags` CRUD (highest priority for Claude Code)
4. **Experiments** — list, get, results, launch, pause, end
5. **Insights + dashboards** — read-only list/get
6. **HogQL query** — `posthog query "<sql>"` → raw results JSON
7. **Schema + `--fields`** — self-describing CLI
8. **Structured errors + `--dry-run`** — machine-readable failures + mutation safety rail

---

## Verification

1. `posthog config set --api-key phx_... --project-id 123` → writes config, shows it back
2. `posthog flags list` → returns JSON array of flags
3. `posthog flags enable my-flag` → returns updated flag JSON
4. `posthog query "SELECT event, count() FROM events GROUP BY event LIMIT 10"` → returns rows JSON
5. `posthog experiments results 42` → returns experiment results JSON
6. Claude Code can run `posthog flags list | jq '.[].key'` to extract flag keys

---

## Key PostHog API Endpoints

- Feature flags: `GET/POST /api/projects/:id/feature_flags/`
- Flag detail: `GET/PATCH/DELETE /api/projects/:id/feature_flags/:flagId/`
- Experiments: `GET/POST /api/projects/:id/experiments/`
- Experiment results: `GET /api/projects/:id/experiments/:id/results/`
- Insights: `GET /api/environments/:id/insights/`
- Dashboards: `GET /api/environments/:id/dashboards/`
- Query (HogQL): `POST /api/environments/:id/query/`

Note: some endpoints use `environments` (newer) vs `projects` (legacy) — handle both.
