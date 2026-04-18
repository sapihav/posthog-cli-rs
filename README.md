# posthog-cli-rs

> **Disclaimer:** This is an unofficial, community-built CLI tool. It is not affiliated with, endorsed by, or supported by [PostHog Inc](https://posthog.com). It interacts with PostHog through their public API. Use at your own risk.

PostHog CLI (Rust port) — manage PostHog projects from the terminal. JSON output by default, designed for scripting and AI agent tooling.

This is a Rust port of [posthog-cli](https://github.com/sapihav/posthog-cli) (TypeScript). Feature surface, JSON output shapes, and command-line interface are kept 1:1 with the TS original. Install via Cargo instead of npm.

## Install

**One-line install (recommended)** — no Rust toolchain required:

```bash
curl -sSL https://raw.githubusercontent.com/sapihav/posthog-cli-rs/main/install.sh | bash
```

Downloads the latest release for your OS/arch, verifies SHA-256, installs `posthog` to `/usr/local/bin`. Override with `INSTALL_DIR=$HOME/.local/bin`. Requires `curl` + `jq`.

Note: the installed binary collides with the official abandoned `@posthog/cli` if that's on your PATH. This one wins if `/usr/local/bin` is ahead of `~/.cargo/bin`.

**From source** (requires Rust 1.70+):

```bash
git clone https://github.com/sapihav/posthog-cli-rs
cd posthog-cli-rs
cargo install --path .
```

The binary is installed as `posthog`. Crates.io publish is planned — see `ROADMAP.md`.

## Setup

```bash
posthog login
```

This will walk you through region selection, open your browser to create an API key, and let you pick a project.

Alternatively, use environment variables:

```bash
export POSTHOG_API_KEY=phx_...
export POSTHOG_PROJECT_ID=12345
```

## Commands

```
posthog login
posthog config set [--api-key ...] [--project-id ...] [--host ...]
posthog config show
```

More commands (`flags`, `experiments`, `insights`, `dashboards`, `query`) are planned — see `ROADMAP.md`.

## Examples

```bash
# Interactive setup
posthog login

# Show current config (apiKey is masked)
posthog config show --pretty
```

## Output

- **stdout**: Always valid JSON (for piping/scripting)
- **stderr**: Human-readable errors, exit code 1
- `--pretty`: Indented JSON for humans

## License

MIT
