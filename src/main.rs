use clap::{CommandFactory, Parser, Subcommand};
use posthog_cli_rs::{commands, output::OutputOptions};

/// Unofficial PostHog CLI — manage PostHog projects from the terminal.
#[derive(Parser, Debug)]
#[command(
    name = "posthog",
    version,
    about = "Unofficial PostHog CLI — manage PostHog projects from the terminal",
    after_help = "For agent/tooling use, run `posthog schema` or append `--help --json` to any command\nfor a machine-readable description of the CLI surface."
)]
struct Cli {
    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pretty: bool,
    /// Comma-separated list of fields to keep in object outputs (e.g. --fields key,active)
    #[arg(long, global = true)]
    fields: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Interactive setup — authenticate and select a project
    Login,
    /// Manage CLI configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Set global config values
    Set(commands::config::SetArgs),
    /// Show current effective config
    Show,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();
    let opts = OutputOptions {
        pretty: cli.pretty,
        fields: cli.fields.clone(),
    };

    match cli.command {
        None => {
            let mut cmd = Cli::command();
            cmd.print_help().expect("help");
            println!();
        }
        Some(Command::Login) => commands::login::run_login(&opts).await,
        Some(Command::Config { command }) => match command {
            ConfigCommand::Set(args) => commands::config::run_set(args, &opts),
            ConfigCommand::Show => commands::config::run_show(&opts),
        },
    }
}
