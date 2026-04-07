//! Smoke-level integration tests for the v0.5.0 rewrite.
//!
//! These tests exercise the CLI end-to-end against the real binary and a
//! per-test SQLite database (via temp dirs). They are intentionally
//! lightweight: each `when_*` module covers one command family and asserts
//! only the behavior needed to catch regressions in plumbing (async dispatch,
//! storage wiring, output formatting).
//!
//! Unit-level coverage of logic lives next to the code in `src/`. These
//! integration tests are the source of truth for CLI behavior as a whole.

mod when_artifact;
mod when_config;
mod when_init;
mod when_iteration;
mod when_iteration_meta;
mod when_search;
mod when_tag;
mod when_task;
mod when_task_meta;
mod when_undo;
mod when_version;
