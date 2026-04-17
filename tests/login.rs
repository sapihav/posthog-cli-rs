use posthog_cli_rs::commands::login::{fetch_projects, host_for, HOST_EU, HOST_US};
use wiremock::matchers::{bearer_token, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// --- host_for ---

#[test]
fn host_for_maps_1_to_us() {
    assert_eq!(host_for("1"), Some(HOST_US));
}

#[test]
fn host_for_maps_2_to_eu() {
    assert_eq!(host_for("2"), Some(HOST_EU));
}

#[test]
fn host_for_returns_none_for_invalid_selection() {
    assert_eq!(host_for("3"), None);
    assert_eq!(host_for(""), None);
    assert_eq!(host_for("foo"), None);
}

// --- fetch_projects ---

#[tokio::test]
async fn fetches_orgs_then_projects() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{ "id": "org-1", "name": "My Org" }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/org-1/projects/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                { "id": 100, "name": "Prod" },
                { "id": 200, "name": "Staging" },
            ]
        })))
        .mount(&server)
        .await;

    let projects = fetch_projects(&server.uri(), "phx_test")
        .await
        .expect("ok")
        .expect("projects present");

    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "Prod");
    assert_eq!(projects[1].name, "Staging");
}

#[tokio::test]
async fn aggregates_projects_across_multiple_orgs() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [
                { "id": "org-1", "name": "Org A" },
                { "id": "org-2", "name": "Org B" },
            ]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/org-1/projects/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{ "id": 1, "name": "P1" }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/org-2/projects/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{ "id": 2, "name": "P2" }]
        })))
        .mount(&server)
        .await;

    let projects = fetch_projects(&server.uri(), "phx_test")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].id, 1);
    assert_eq!(projects[1].id, 2);
}

#[tokio::test]
async fn sends_bearer_auth_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/"))
        .and(bearer_token("phx_my_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    fetch_projects(&server.uri(), "phx_my_key").await.unwrap();
    // MockServer asserts `expect(1)` on drop.
}

#[tokio::test]
async fn returns_none_when_orgs_endpoint_is_forbidden() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "detail": "permission_denied"
        })))
        .mount(&server)
        .await;

    let result = fetch_projects(&server.uri(), "phx_scoped").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn errors_when_project_fetch_fails() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{ "id": "org-1", "name": "My Org" }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/org-1/projects/"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "detail": "Server error"
        })))
        .mount(&server)
        .await;

    let err = fetch_projects(&server.uri(), "phx_test")
        .await
        .expect_err("should error");
    assert!(
        err.message.contains("Failed to fetch projects for org \"My Org\" (500)"),
        "message was: {}",
        err.message
    );
}

#[tokio::test]
async fn returns_empty_array_when_org_has_no_projects() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{ "id": "org-1", "name": "Empty Org" }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/organizations/org-1/projects/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": []
        })))
        .mount(&server)
        .await;

    let projects = fetch_projects(&server.uri(), "phx_test")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(projects.len(), 0);
}
