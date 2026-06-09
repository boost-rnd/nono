//! abs-prototype patch: library target for nono-cli.
//!
//! Upstream `nono-cli` ships only as a binary; this file is added on
//! the local `abs-lib-export` branch so the `a-better-sandbox`
//! prototype can call into nono-cli's high-level APIs (profile
//! loading, `run_sandbox`, session management, audit, rollback) as a
//! library, without reimplementing them on top of the pure `nono`
//! library primitive.
//!
//! The module tree mirrors `src/main.rs`'s. Each module compiles
//! twice (once into this lib crate, once into the bin crate) so the
//! bin's `crate::*` references continue to resolve and we avoid
//! touching ~70 source files.
//!
//! **Do not push this file upstream.** It is part of the abs-prototype
//! local fork; nono itself is responsible for deciding whether it
//! wants a stable library API.

// We intentionally mirror the bin crate's full module tree even
// though only a subset is re-exported here. Each unreferenced
// `pub(crate)` item triggers `dead_code` from this lib's vantage
// point — silenced because it'll be reached from `main.rs`'s tree.
#![allow(clippy::pedantic, dead_code, private_interfaces)]

pub mod app_runtime;
pub mod audit_attestation;
pub mod audit_commands;
pub mod audit_integrity;
pub mod audit_ledger;
pub mod audit_session;
pub mod capability_ext;
pub mod cli;
pub mod cli_bootstrap;
pub mod command_blocking_deprecation;
pub mod command_display;
pub mod command_runtime;
pub mod completions;
pub mod config;
pub mod credential_runtime;
pub mod deprecated_policy;
pub mod deprecated_schema;
pub mod deprecation_warnings;
pub mod exec_strategy;
pub mod execution_runtime;
#[cfg(unix)]
pub mod hook_runtime;
pub mod instruction_deny;
pub mod jsonc;
pub mod launch_runtime;
pub mod learn;
pub mod learn_runtime;
pub mod legacy_cleanup;
#[cfg(target_os = "macos")]
pub mod macos_trust;
pub mod migration;
pub mod network_policy;
pub mod open_url_runtime;
pub mod output;
pub mod pack_update_hint;
pub mod package;
pub mod package_cmd;
pub mod package_status;
pub mod platform;
pub mod policy;
pub mod profile;
pub mod profile_cmd;
pub mod profile_runtime;
pub mod profile_save_runtime;
pub mod protected_paths;
pub mod proxy_runtime;
pub mod pty_proxy;
pub mod pull_ui;
pub mod query_ext;
pub mod registry_client;
pub mod rollback_commands;
pub mod rollback_preflight;
pub mod rollback_runtime;
pub mod rollback_session;
pub mod rollback_ui;
pub mod sandbox_log;
pub mod sandbox_prepare;
pub mod sandbox_state;
pub mod session;
pub mod session_commands;
pub mod setup;
pub mod startup_prompt;
pub mod startup_runtime;
pub mod supervised_runtime;
pub mod terminal_approval;
pub mod theme;
pub mod timeouts;
pub mod trust_cmd;
pub mod trust_intercept;
pub mod trust_keystore;
pub mod trust_scan;
pub mod update_check;
pub mod why_runtime;
pub mod wiring;

#[cfg(test)]
mod test_env;

// Mirror the crate-root items that main.rs declares, so modules that
// reference `crate::Result`, `crate::DETACHED_*`, `crate::rollback_base_exclusions`,
// and `crate::merge_dedup_ports` resolve identically when compiled
// into this lib crate root.
pub(crate) use nono::Result;

pub(crate) const DETACHED_LAUNCH_ENV: &str = "NONO_DETACHED_LAUNCH";
pub(crate) const DETACHED_CWD_PROMPT_RESPONSE_ENV: &str = "NONO_DETACHED_CWD_PROMPT_RESPONSE";
pub(crate) const DETACHED_SESSION_ID_ENV: &str = "NONO_DETACHED_SESSION_ID";

pub(crate) use launch_runtime::rollback_base_exclusions;
pub(crate) use proxy_runtime::merge_dedup_ports;

// -----------------------------------------------------------------
// Public entry points for downstream crates (abs-nono).
//
// `command_runtime::run_sandbox` etc. are `pub(crate)` upstream; we
// provide thin `pub` wrappers here so abs-nono can call them without
// our patch having to touch the original module sources.
// -----------------------------------------------------------------

/// Path to a session's persisted JSON record. Widening of
/// `session::session_file_path` (`pub(crate)` upstream) so
/// `abs-nono::launch_detached` can poll for the file to appear.
///
/// # Errors
///
/// Returns the underlying `NonoError` if the sessions directory
/// can't be resolved.
pub fn session_file_path(session_id: &str) -> Result<std::path::PathBuf> {
    session::session_file_path(session_id)
}

/// Path to a session's Unix-socket file. Companion to
/// [`session_file_path`].
///
/// # Errors
///
/// See [`session_file_path`].
pub fn session_socket_path(session_id: &str) -> Result<std::path::PathBuf> {
    session::session_socket_path(session_id)
}

/// Synchronous supervised launch — same code path as `nono run …`
/// when `--detached` is unset. Detached launches happen in
/// `abs-nono::launch_detached`, which reproduces the fork-and-fly
/// pattern with an `abs __supervise` re-exec; see
/// `docs/nono-integration.md` for why nono-cli's own
/// `run_detached_launch` can't be reused.
///
/// # Errors
///
/// Returns whatever the underlying runtime returns; `nono::Result`
/// already encodes the full error surface.
pub fn run_sandbox(args: cli::RunArgs, silent: bool) -> Result<()> {
    command_runtime::run_sandbox(args, silent)
}

/// Mirror of `nono shell …`. See [`run_sandbox`].
///
/// # Errors
///
/// See [`run_sandbox`].
pub fn run_shell(args: cli::ShellArgs, silent: bool) -> Result<()> {
    command_runtime::run_shell(args, silent)
}

/// Mirror of `nono wrap …`. See [`run_sandbox`].
///
/// # Errors
///
/// See [`run_sandbox`].
pub fn run_wrap(args: cli::WrapArgs, silent: bool) -> Result<()> {
    command_runtime::run_wrap(args, silent)
}

/// Directory the user's editable profile JSONs live in
/// (`$XDG_CONFIG_HOME/nono/profiles` or `$HOME/.config/nono/profiles`).
/// Widening of `profile::user_profile_dir` (`pub(crate)` upstream) so
/// abs's profile-save / install paths can write through the same
/// resolution rules nono itself uses.
///
/// # Errors
///
/// Returns the underlying `NonoError` if HOME can't be resolved.
pub fn user_profile_dir() -> Result<std::path::PathBuf> {
    profile::user_profile_dir()
}

/// Absolute path of `<name>.json` inside the user's profile dir.
/// Companion to [`user_profile_dir`].
///
/// # Errors
///
/// See [`user_profile_dir`].
pub fn get_user_profile_path(name: &str) -> Result<std::path::PathBuf> {
    profile::get_user_profile_path(name)
}

/// `true` iff `name` is acceptable as a profile registry key
/// (alphanumeric + hyphen, no leading/trailing hyphen). Widening of
/// `profile::is_valid_profile_name`.
#[must_use]
pub fn is_valid_profile_name(name: &str) -> bool {
    profile::is_valid_profile_name(name)
}
