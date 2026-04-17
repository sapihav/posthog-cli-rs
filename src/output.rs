//! Output helpers — stdout is always JSON, stderr is structured JSON errors.

use serde::Serialize;
use serde_json::{Map, Value};

use crate::errors::PostHogError;

#[derive(Debug, Default, Clone)]
pub struct OutputOptions {
    pub pretty: bool,
    /// Comma-separated list of fields to project from object/array-of-object outputs.
    pub fields: Option<String>,
}

/// Project an object (or array of objects) down to only the listed fields.
/// Non-object values pass through unchanged. Missing fields are silently omitted.
pub fn project_fields(data: Value, fields: Option<&str>) -> Value {
    let Some(raw) = fields else { return data };
    let keys: Vec<&str> = raw.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if keys.is_empty() {
        return data;
    }

    fn project_one(v: Value, keys: &[&str]) -> Value {
        match v {
            Value::Object(map) => {
                let mut out = Map::new();
                for k in keys {
                    if let Some(val) = map.get(*k) {
                        out.insert((*k).to_string(), val.clone());
                    }
                }
                Value::Object(out)
            }
            other => other,
        }
    }

    match data {
        Value::Array(arr) => Value::Array(arr.into_iter().map(|v| project_one(v, &keys)).collect()),
        other => project_one(other, &keys),
    }
}

/// Pure-JSON renderer. Exposed so tests don't need to capture stdout.
pub fn render_json<T: Serialize>(data: &T, opts: &OutputOptions) -> String {
    let value = serde_json::to_value(data).expect("value is serializable");
    let projected = project_fields(value, opts.fields.as_deref());
    if opts.pretty {
        serde_json::to_string_pretty(&projected).expect("serializable")
    } else {
        serde_json::to_string(&projected).expect("serializable")
    }
}

/// Write JSON to stdout followed by a newline. Always valid JSON.
pub fn output_json<T: Serialize>(data: &T, opts: &OutputOptions) {
    println!("{}", render_json(data, opts));
}

/// Render the stderr error envelope. Public for tests.
pub fn render_error(err: &PostHogError) -> String {
    let mut inner = Map::new();
    inner.insert("message".into(), Value::String(err.message.clone()));
    inner.insert("code".into(), serde_json::to_value(err.code).expect("code"));
    if let Some(h) = &err.hint {
        inner.insert("hint".into(), Value::String(h.clone()));
    }
    if let Some(d) = &err.docs_url {
        inner.insert("docs_url".into(), Value::String(d.clone()));
    }
    let mut outer = Map::new();
    outer.insert("error".into(), Value::Object(inner));
    serde_json::to_string(&Value::Object(outer)).expect("serializable")
}

/// Emit a structured JSON error to stderr and exit with code 1.
pub fn output_error(err: &PostHogError) -> ! {
    eprintln!("{}", render_error(err));
    std::process::exit(1);
}
