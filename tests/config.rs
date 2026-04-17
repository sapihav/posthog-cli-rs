use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use posthog_cli_rs::config::{
    load_config_from, require_config_from, save_global_config_to, Config, PartialConfig,
};

// Env var tests race with each other — serialize them behind a single mutex.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Acquire the env-mutation lock, tolerating poison so a panicking test
/// doesn't cascade into PoisonError for every subsequent test.
fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn clear_posthog_env() {
    std::env::remove_var("POSTHOG_API_KEY");
    std::env::remove_var("POSTHOG_PROJECT_ID");
    std::env::remove_var("POSTHOG_HOST");
}

fn empty_paths() -> (PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    (
        tmp.path().join("nonexistent-global.json"),
        tmp.path().join("nonexistent-local.json"),
    )
}

#[test]
fn load_returns_defaults_when_nothing_is_set() {
    let _g = lock_env();
    clear_posthog_env();
    let (gp, lp) = empty_paths();

    let config = load_config_from(&gp, &lp);
    assert_eq!(config.api_key, "");
    assert_eq!(config.project_id, "");
    assert_eq!(config.host, "https://us.posthog.com");
}

#[test]
fn env_vars_win_over_global_and_local() {
    let _g = lock_env();
    clear_posthog_env();
    std::env::set_var("POSTHOG_API_KEY", "phx_env_key");
    std::env::set_var("POSTHOG_PROJECT_ID", "env_project");
    std::env::set_var("POSTHOG_HOST", "https://eu.posthog.com");

    let (gp, lp) = empty_paths();
    let config = load_config_from(&gp, &lp);

    assert_eq!(config.api_key, "phx_env_key");
    assert_eq!(config.project_id, "env_project");
    assert_eq!(config.host, "https://eu.posthog.com");

    clear_posthog_env();
}

#[test]
fn local_config_only_sets_project_id() {
    let _g = lock_env();
    clear_posthog_env();

    let tmp = tempfile::tempdir().unwrap();
    let global = tmp.path().join("global.json");
    let local = tmp.path().join(".posthog.json");

    fs::write(
        &global,
        r#"{"apiKey":"phx_global","projectId":"global_pid","host":"https://us.posthog.com"}"#,
    )
    .unwrap();
    fs::write(
        &local,
        r#"{"apiKey":"phx_MALICIOUS","projectId":"local_pid","host":"https://attacker.com"}"#,
    )
    .unwrap();

    let config = load_config_from(&global, &local);

    // apiKey and host come from global only
    assert_eq!(config.api_key, "phx_global");
    assert_eq!(config.host, "https://us.posthog.com");
    // projectId is honored from local (takes priority over global when set)
    assert_eq!(config.project_id, "local_pid");
}

#[test]
fn global_config_roundtrip() {
    let _g = lock_env();
    clear_posthog_env();

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.json");

    let saved = save_global_config_to(
        &path,
        PartialConfig {
            api_key: Some("phx_saved".into()),
            project_id: Some("999".into()),
            host: Some("https://us.posthog.com".into()),
        },
    )
    .expect("save ok");

    assert_eq!(saved.api_key, "phx_saved");
    assert_eq!(saved.project_id, "999");
    assert_eq!(saved.host, "https://us.posthog.com");

    let raw = fs::read_to_string(&path).unwrap();
    let parsed: Config = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed.api_key, "phx_saved");
    assert_eq!(parsed.project_id, "999");
    assert_eq!(parsed.host, "https://us.posthog.com");
    // File should end with a trailing newline (mirrors TS)
    assert!(raw.ends_with('\n'));
}

#[test]
fn save_merges_with_existing_values() {
    let _g = lock_env();
    clear_posthog_env();

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.json");

    save_global_config_to(
        &path,
        PartialConfig {
            api_key: Some("phx_first".into()),
            project_id: Some("111".into()),
            host: Some("https://us.posthog.com".into()),
        },
    )
    .unwrap();

    // Now only update project_id; api_key and host should persist
    let merged = save_global_config_to(
        &path,
        PartialConfig {
            api_key: None,
            project_id: Some("222".into()),
            host: None,
        },
    )
    .unwrap();

    assert_eq!(merged.api_key, "phx_first");
    assert_eq!(merged.project_id, "222");
    assert_eq!(merged.host, "https://us.posthog.com");
}

#[test]
fn require_config_returns_ok_when_both_env_vars_set() {
    let _g = lock_env();
    clear_posthog_env();
    std::env::set_var("POSTHOG_API_KEY", "phx_required");
    std::env::set_var("POSTHOG_PROJECT_ID", "12345");

    let (gp, lp) = empty_paths();
    let config = require_config_from(&gp, &lp).expect("both set, should be Ok");
    assert_eq!(config.api_key, "phx_required");
    assert_eq!(config.project_id, "12345");

    clear_posthog_env();
}

#[test]
fn require_config_errors_when_api_key_missing() {
    let _g = lock_env();
    clear_posthog_env();
    std::env::set_var("POSTHOG_PROJECT_ID", "12345");

    let (gp, lp) = empty_paths();
    let err = require_config_from(&gp, &lp).expect_err("should error");
    assert_eq!(err.code, posthog_cli_rs::errors::ErrorCode::AuthMissing);
    assert!(err.message.contains("No API key configured"));

    clear_posthog_env();
}

#[test]
fn require_config_errors_when_project_id_missing() {
    let _g = lock_env();
    clear_posthog_env();
    std::env::set_var("POSTHOG_API_KEY", "phx_test");

    let (gp, lp) = empty_paths();
    let err = require_config_from(&gp, &lp).expect_err("should error");
    assert_eq!(err.code, posthog_cli_rs::errors::ErrorCode::AuthMissing);
    assert!(err.message.contains("No project ID configured"));

    clear_posthog_env();
}

#[test]
#[cfg(unix)]
fn saved_config_file_is_chmod_600_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let _g = lock_env();
    clear_posthog_env();

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.json");

    save_global_config_to(
        &path,
        PartialConfig {
            api_key: Some("phx_secret".into()),
            project_id: None,
            host: None,
        },
    )
    .unwrap();

    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "expected 0o600, got {:o}", mode);
}

#[test]
fn save_rejects_invalid_host() {
    let _g = lock_env();
    clear_posthog_env();

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config.json");

    let err = save_global_config_to(
        &path,
        PartialConfig {
            api_key: None,
            project_id: None,
            host: Some("https://attacker.com".into()),
        },
    )
    .expect_err("should reject");

    assert_eq!(err.code, posthog_cli_rs::errors::ErrorCode::Validation);
    assert!(err.message.contains("Invalid host"));
}
