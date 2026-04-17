use posthog_cli_rs::commands::config::mask_api_key;
use posthog_cli_rs::errors::{ErrorCode, PostHogError};
use posthog_cli_rs::output::{project_fields, render_error, render_json, OutputOptions};
use serde_json::{json, Value};

// --- render_json ---

#[test]
fn renders_compact_json_by_default() {
    let out = render_json(&json!({ "key": "value", "num": 42 }), &OutputOptions::default());
    assert_eq!(out, r#"{"key":"value","num":42}"#);
}

#[test]
fn renders_pretty_json_when_requested() {
    let opts = OutputOptions { pretty: true, fields: None };
    let out = render_json(&json!({ "key": "value" }), &opts);
    assert_eq!(out, "{\n  \"key\": \"value\"\n}");
}

#[test]
fn renders_arrays() {
    let out = render_json(&json!([1, 2, 3]), &OutputOptions::default());
    assert_eq!(out, "[1,2,3]");
}

#[test]
fn renders_null() {
    let out = render_json(&Value::Null, &OutputOptions::default());
    assert_eq!(out, "null");
}

#[test]
fn projects_fields_on_single_object() {
    let opts = OutputOptions {
        pretty: false,
        fields: Some("key,active".into()),
    };
    let out = render_json(
        &json!({ "id": 1, "key": "my-flag", "name": "My Flag", "active": true }),
        &opts,
    );
    assert_eq!(out, r#"{"key":"my-flag","active":true}"#);
}

#[test]
fn projects_fields_on_array_of_objects() {
    let opts = OutputOptions {
        pretty: false,
        fields: Some("key,active".into()),
    };
    let out = render_json(
        &json!([
            { "id": 1, "key": "a", "active": true, "extra": "x" },
            { "id": 2, "key": "b", "active": false, "extra": "y" },
        ]),
        &opts,
    );
    assert_eq!(
        out,
        r#"[{"key":"a","active":true},{"key":"b","active":false}]"#
    );
}

#[test]
fn silently_omits_missing_fields() {
    let opts = OutputOptions {
        pretty: false,
        fields: Some("key,nope".into()),
    };
    let out = render_json(&json!({ "id": 1, "key": "x" }), &opts);
    assert_eq!(out, r#"{"key":"x"}"#);
}

#[test]
fn trims_whitespace_in_fields_list() {
    let opts = OutputOptions {
        pretty: false,
        fields: Some(" a , c ".into()),
    };
    let out = render_json(&json!({ "a": 1, "b": 2, "c": 3 }), &opts);
    assert_eq!(out, r#"{"a":1,"c":3}"#);
}

#[test]
fn passes_scalar_values_through_unchanged_when_fields_set() {
    let opts = OutputOptions {
        pretty: false,
        fields: Some("key".into()),
    };
    let out = render_json(&json!(42), &opts);
    assert_eq!(out, "42");
}

#[test]
fn combines_pretty_and_fields() {
    let opts = OutputOptions {
        pretty: true,
        fields: Some("key".into()),
    };
    let out = render_json(&json!({ "id": 1, "key": "x", "name": "X" }), &opts);
    assert_eq!(out, "{\n  \"key\": \"x\"\n}");
}

// --- project_fields ---

#[test]
fn project_fields_passes_through_when_none() {
    let data = json!({ "a": 1, "b": 2 });
    assert_eq!(project_fields(data.clone(), None), data);
}

#[test]
fn project_fields_passes_through_on_empty_string() {
    let data = json!({ "a": 1, "b": 2 });
    assert_eq!(project_fields(data.clone(), Some("")), data);
}

#[test]
fn project_fields_preserves_null_values() {
    let data = json!({ "a": null, "b": 1 });
    assert_eq!(project_fields(data, Some("a,b")), json!({ "a": null, "b": 1 }));
}

// --- render_error ---

#[test]
fn error_emits_structured_json() {
    let err = PostHogError::new("Something broke", ErrorCode::ApiError);
    let s = render_error(&err);
    let parsed: Value = serde_json::from_str(&s).unwrap();
    assert_eq!(
        parsed,
        json!({ "error": { "message": "Something broke", "code": "API_ERROR" } })
    );
}

#[test]
fn error_preserves_code_hint_and_docs_url() {
    let err = PostHogError {
        message: "Nope".into(),
        code: ErrorCode::AuthInvalid,
        hint: Some("Run posthog login".into()),
        docs_url: Some("https://example.com/docs".into()),
        status: None,
    };
    let parsed: Value = serde_json::from_str(&render_error(&err)).unwrap();
    assert_eq!(
        parsed,
        json!({
            "error": {
                "message": "Nope",
                "code": "AUTH_INVALID",
                "hint": "Run posthog login",
                "docs_url": "https://example.com/docs",
            }
        })
    );
}

#[test]
fn error_omits_hint_and_docs_url_when_absent() {
    let err = PostHogError::new("x", ErrorCode::ApiError);
    let parsed: Value = serde_json::from_str(&render_error(&err)).unwrap();
    assert_eq!(parsed["error"].get("hint"), None);
    assert_eq!(parsed["error"].get("docs_url"), None);
}

// --- mask_api_key ---

#[test]
fn mask_empty_key_returns_not_set() {
    assert_eq!(mask_api_key(""), "(not set)");
}

#[test]
fn mask_typical_phx_key() {
    // head=7 chars, "..." separator, tail=4 chars
    assert_eq!(mask_api_key("phx_abcdefghijklmnop1234"), "phx_abc...1234");
}

#[test]
fn mask_short_key_degrades_gracefully() {
    // 3-char input: head=entire string, tail=entire string → "aaa...aaa"
    assert_eq!(mask_api_key("aaa"), "aaa...aaa");
}

#[test]
fn mask_non_ascii_key_does_not_panic() {
    // Defensive: even though real API keys are ASCII, masking must be char-safe.
    let result = mask_api_key("phx_😀😀😀😀😀");
    // Should not panic and should produce a string (exact formatting is not load-bearing).
    assert!(result.contains("..."));
}

#[test]
fn error_code_serializes_all_variants_to_screaming_snake() {
    assert_eq!(
        serde_json::to_string(&ErrorCode::AuthMissing).unwrap(),
        "\"AUTH_MISSING\""
    );
    assert_eq!(
        serde_json::to_string(&ErrorCode::RateLimited).unwrap(),
        "\"RATE_LIMITED\""
    );
    assert_eq!(
        serde_json::to_string(&ErrorCode::ApiError).unwrap(),
        "\"API_ERROR\""
    );
}
