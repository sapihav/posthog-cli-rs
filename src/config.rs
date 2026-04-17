//! Config read/write. Priority: env vars > local `.posthog.json` (projectId only) > `~/.config/posthog/config.json`.
//!
//! Security: local `.posthog.json` can only set `projectId`. `apiKey` and `host`
//! are restricted to env vars and global config to prevent credential theft via
//! malicious repositories.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::errors::{ErrorCode, PostHogError};

const DEFAULT_HOST: &str = "https://us.posthog.com";
const LOCAL_CONFIG_NAME: &str = ".posthog.json";

fn allowed_hosts() -> &'static HashSet<&'static str> {
    static HOSTS: OnceLock<HashSet<&'static str>> = OnceLock::new();
    HOSTS.get_or_init(|| {
        let mut s = HashSet::new();
        s.insert("https://us.posthog.com");
        s.insert("https://eu.posthog.com");
        s
    })
}

pub fn global_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("posthog")
}

pub fn global_config_path() -> PathBuf {
    global_config_dir().join("config.json")
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub api_key: String,
    pub project_id: String,
    pub host: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

fn read_json_file(path: &Path) -> PartialConfig {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.is_empty())
}

/// Low-level: load config given explicit paths. Used by tests.
pub fn load_config_from(global_path: &Path, local_path: &Path) -> Config {
    let global = read_json_file(global_path);
    let local = read_json_file(local_path);

    let api_key = non_empty(std::env::var("POSTHOG_API_KEY").ok())
        .or_else(|| non_empty(global.api_key.clone()))
        .unwrap_or_default();
    let project_id = non_empty(std::env::var("POSTHOG_PROJECT_ID").ok())
        .or_else(|| non_empty(local.project_id.clone()))
        .or_else(|| non_empty(global.project_id.clone()))
        .unwrap_or_default();
    let host = non_empty(std::env::var("POSTHOG_HOST").ok())
        .or_else(|| non_empty(global.host.clone()))
        .unwrap_or_else(|| DEFAULT_HOST.to_string());

    Config { api_key, project_id, host }
}

/// Resolve config using env > ./.posthog.json > ~/.config/posthog/config.json.
pub fn load_config() -> Config {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    load_config_from(&global_config_path(), &cwd.join(LOCAL_CONFIG_NAME))
}

/// Low-level: validate the resolved config given explicit paths. Used by tests.
pub fn require_config_from(global_path: &Path, local_path: &Path) -> Result<Config, PostHogError> {
    let c = load_config_from(global_path, local_path);
    if c.api_key.is_empty() {
        return Err(PostHogError::new("No API key configured.", ErrorCode::AuthMissing).with_hint(
            "Run `posthog login` or `posthog config set --api-key <key>`, or set POSTHOG_API_KEY.",
        ));
    }
    if c.project_id.is_empty() {
        return Err(
            PostHogError::new("No project ID configured.", ErrorCode::AuthMissing).with_hint(
                "Run `posthog login` or `posthog config set --project-id <id>`, or set POSTHOG_PROJECT_ID.",
            ),
        );
    }
    Ok(c)
}

/// Exported for use by commands that need config. Used by the API client in M2.
#[allow(dead_code)]
pub fn require_config() -> Result<Config, PostHogError> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    require_config_from(&global_config_path(), &cwd.join(LOCAL_CONFIG_NAME))
}

fn strip_trailing_slash(s: &str) -> &str {
    s.strip_suffix('/').unwrap_or(s)
}

/// Low-level: save config to an explicit path. Used by tests.
pub fn save_global_config_to(path: &Path, partial: PartialConfig) -> Result<Config, PostHogError> {
    if let Some(host) = &partial.host {
        if !allowed_hosts().contains(strip_trailing_slash(host)) {
            return Err(PostHogError::new(
                format!(
                    "Invalid host \"{}\". Allowed: https://us.posthog.com, https://eu.posthog.com",
                    host
                ),
                ErrorCode::Validation,
            ));
        }
    }

    let existing = read_json_file(path);
    let merged = Config {
        api_key: non_empty(partial.api_key)
            .or_else(|| non_empty(existing.api_key))
            .unwrap_or_default(),
        project_id: non_empty(partial.project_id)
            .or_else(|| non_empty(existing.project_id))
            .unwrap_or_default(),
        host: non_empty(partial.host)
            .or_else(|| non_empty(existing.host))
            .unwrap_or_else(|| DEFAULT_HOST.to_string()),
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            PostHogError::new(
                format!("Failed to create config dir: {}", e),
                ErrorCode::ApiError,
            )
        })?;
    }
    let mut json = serde_json::to_string_pretty(&merged).expect("serializable");
    json.push('\n');
    fs::write(path, json).map_err(|e| {
        PostHogError::new(format!("Failed to write config: {}", e), ErrorCode::ApiError)
    })?;
    // The config file contains a plaintext API key — on unix, restrict to owner r/w.
    // Best-effort: if chmod fails (e.g., on a filesystem without unix perms), swallow
    // rather than surfacing a misleading "save failed" error.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(merged)
}

pub fn save_global_config(partial: PartialConfig) -> Result<Config, PostHogError> {
    save_global_config_to(&global_config_path(), partial)
}
