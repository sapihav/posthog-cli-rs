//! `posthog config set` / `posthog config show`

use clap::Args;
use serde_json::json;

use crate::config::{load_config, save_global_config, PartialConfig};
use crate::errors::{ErrorCode, PostHogError};
use crate::output::{output_error, output_json, OutputOptions};

#[derive(Args, Debug)]
pub struct SetArgs {
    /// PostHog personal API key (phx_...)
    #[arg(long = "api-key")]
    pub api_key: Option<String>,
    /// PostHog project ID
    #[arg(long = "project-id")]
    pub project_id: Option<String>,
    /// PostHog host: us.posthog.com (default) or eu.posthog.com
    #[arg(long)]
    pub host: Option<String>,
}

pub fn run_set(args: SetArgs, opts: &OutputOptions) {
    if args.api_key.is_none() && args.project_id.is_none() && args.host.is_none() {
        output_error(&PostHogError::new(
            "Provide at least one of --api-key, --project-id, or --host.",
            ErrorCode::Validation,
        ));
    }
    match save_global_config(PartialConfig {
        api_key: args.api_key,
        project_id: args.project_id,
        host: args.host,
    }) {
        Ok(config) => output_json(&config, opts),
        Err(e) => output_error(&e),
    }
}

/// Format key as `phx_xxx...yyyy`. Mirrors TS `slice(0, 7) + "..." + slice(-4)`.
/// Char-based (not byte-based) so it can't panic on multibyte input — API keys
/// are ASCII in practice but the helper is defensive.
pub fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return "(not set)".to_string();
    }
    let char_count = key.chars().count();
    let head: String = key.chars().take(7).collect();
    let tail: String = key.chars().skip(char_count.saturating_sub(4)).collect();
    format!("{}...{}", head, tail)
}

pub fn run_show(opts: &OutputOptions) {
    let config = load_config();
    let display = json!({
        "apiKey": mask_api_key(&config.api_key),
        "projectId": config.project_id,
        "host": config.host,
    });
    output_json(&display, opts);
}
