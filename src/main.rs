use clap::{CommandFactory, Parser};

/// Unofficial PostHog CLI — manage PostHog projects from the terminal.
///
/// Rust port of the TypeScript posthog-cli. See ROADMAP.md for milestone
/// status; commands land in M1 and later.
#[derive(Parser, Debug)]
#[command(
    name = "posthog",
    version,
    about = "Unofficial PostHog CLI — manage PostHog projects from the terminal",
    after_help = "For agent/tooling use, run `posthog schema` or append `--help --json` to any command\nfor a machine-readable description of the CLI surface."
)]
struct Cli {}

fn main() {
    // M0 scaffold: no subcommands wired up yet. Print help when invoked with
    // no arguments so the binary is self-describing even before M1 lands.
    if std::env::args().len() <= 1 {
        let mut cmd = Cli::command();
        cmd.print_help().expect("failed to print help");
        println!();
        return;
    }
    Cli::parse();
}
