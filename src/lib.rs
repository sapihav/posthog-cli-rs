//! Library surface for `posthog-cli-rs`. The binary at `src/main.rs` is a
//! thin wiring layer on top of these modules. Exposing them as a library
//! lets integration tests under `tests/` exercise the same code paths.

pub mod commands;
pub mod config;
pub mod errors;
pub mod output;
