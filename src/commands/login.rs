//! `posthog login` — interactive setup.

use std::io::{self, IsTerminal, Write};

use serde::Deserialize;
use serde_json::json;

use crate::commands::config::mask_api_key;
use crate::config::{global_config_path, save_global_config, PartialConfig};
use crate::errors::{ErrorCode, PostHogError};
use crate::output::{output_error, output_json, OutputOptions};

// `io::Error` and `reqwest::Error` → `PostHogError` via `?` come from the
// `From` impls in `errors.rs`. Use `?` directly rather than a local helper.

pub const HOST_US: &str = "https://us.posthog.com";
pub const HOST_EU: &str = "https://eu.posthog.com";

/// Map "1"/"2" to the corresponding host URL. Mirrors TS `HOSTS`.
pub fn host_for(choice: &str) -> Option<&'static str> {
    match choice {
        "1" => Some(HOST_US),
        "2" => Some(HOST_EU),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct Organization {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct ApiList<T> {
    results: Vec<T>,
}

/// Fetch all projects an API key can see across its organizations.
/// Returns `Ok(None)` if the orgs endpoint returns any non-success status,
/// which signals a project-scoped key that cannot enumerate projects.
///
/// Parity-preserving TODO: TS returns `null` for ALL non-success statuses on
/// `/api/organizations/`, so a transient 5xx would also be treated as "scoped
/// key" here. Consider pinning to 401/403 only once M5 introduces a richer
/// error path — see ROADMAP.
pub async fn fetch_projects(host: &str, api_key: &str) -> Result<Option<Vec<Project>>, PostHogError> {
    let base = host.trim_end_matches('/');
    let client = reqwest::Client::new();

    let orgs_resp = client
        .get(format!("{}/api/organizations/", base))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if !orgs_resp.status().is_success() {
        return Ok(None);
    }

    let orgs: ApiList<Organization> = orgs_resp.json().await?;

    let mut projects = Vec::new();
    for org in orgs.results {
        let proj_resp = client
            .get(format!("{}/api/organizations/{}/projects/", base, org.id))
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !proj_resp.status().is_success() {
            let status = proj_resp.status().as_u16();
            let text = proj_resp.text().await.unwrap_or_default();
            return Err(PostHogError {
                message: format!(
                    "Failed to fetch projects for org \"{}\" ({}): {}",
                    org.name, status, text
                ),
                code: ErrorCode::ApiError,
                hint: None,
                docs_url: None,
                status: Some(status),
            });
        }

        let list: ApiList<Project> = proj_resp.json().await?;
        projects.extend(list.results);
    }
    Ok(Some(projects))
}

/// Prompt on stderr, read a line from stdin, trim, and return.
fn prompt(question: &str) -> io::Result<String> {
    eprint!("{}", question);
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn prompt_secret(question: &str) -> io::Result<String> {
    if io::stdin().is_terminal() {
        eprint!("{}", question);
        io::stderr().flush()?;
        rpassword::read_password()
    } else {
        prompt(question)
    }
}

fn log(msg: &str) {
    eprintln!("{}", msg);
}

pub async fn run_login(opts: &OutputOptions) {
    match do_login(opts).await {
        Ok(()) => {}
        Err(e) => output_error(&e),
    }
}

async fn do_login(opts: &OutputOptions) -> Result<(), PostHogError> {
    // 1. Region
    log("");
    log("  Region:");
    log("    [1] US (us.posthog.com)");
    log("    [2] EU (eu.posthog.com)");
    let region = prompt("  Select (1/2): ")?;
    let host = host_for(&region).ok_or_else(|| {
        PostHogError::new("Invalid selection. Choose 1 or 2.", ErrorCode::Validation)
    })?;

    // 2. Open browser to API key page
    let key_url = format!("{}/settings/user-api-keys", host);
    log("");
    log("  Opening browser to create an API key...");
    log(&format!("  {}", key_url));
    let _ = open::that(&key_url);

    // 3. API key (masked)
    log("");
    let api_key = prompt_secret("  Paste your API key: ")?;
    if api_key.is_empty() || !api_key.starts_with("phx_") {
        return Err(
            PostHogError::new("Invalid API key. Must start with phx_", ErrorCode::Validation)
                .with_hint("Create a personal API key at the URL above and paste the full token."),
        );
    }

    // 4. Fetch and select project
    log("");
    log("  Fetching your projects...");
    let project_id = resolve_project_id(host, &api_key).await?;

    // 5. Save
    let saved = save_global_config(PartialConfig {
        api_key: Some(api_key),
        project_id: Some(project_id),
        host: Some(host.to_string()),
    })?;

    log("");
    log(&format!("  Config saved to {}", global_config_path().display()));
    log("");

    let display = json!({
        "host": saved.host,
        "projectId": saved.project_id,
        "apiKey": mask_api_key(&saved.api_key),
    });
    output_json(&display, opts);
    Ok(())
}

async fn resolve_project_id(host: &str, api_key: &str) -> Result<String, PostHogError> {
    match fetch_projects(host, api_key).await? {
        None => {
            log("  Project-scoped API key detected — cannot list projects.");
            log(&format!(
                "  Find your project ID at: {}/settings/project#variables",
                host
            ));
            let pid = prompt("  Enter your project ID: ")?;
            if pid.is_empty() || !pid.chars().all(|c| c.is_ascii_digit()) {
                return Err(PostHogError::new(
                    "Invalid project ID. Must be a number.",
                    ErrorCode::Validation,
                ));
            }
            Ok(pid)
        }
        Some(list) if list.is_empty() => Err(PostHogError::new(
            "No projects found for this API key.",
            ErrorCode::NotFound,
        )
        .with_hint(
            "Make sure your API key is associated with at least one organization/project.",
        )),
        Some(list) if list.len() == 1 => {
            log(&format!("  Using project: {} ({})", list[0].name, list[0].id));
            Ok(list[0].id.to_string())
        }
        Some(list) => {
            for (i, p) in list.iter().enumerate() {
                log(&format!("    [{}] {} (id: {})", i + 1, p.name, p.id));
            }
            let choice = prompt("  Select project: ")?;
            let idx = choice
                .parse::<usize>()
                .ok()
                .and_then(|n| n.checked_sub(1))
                .filter(|&i| i < list.len())
                .ok_or_else(|| {
                    PostHogError::new("Invalid selection.", ErrorCode::Validation)
                })?;
            Ok(list[idx].id.to_string())
        }
    }
}
