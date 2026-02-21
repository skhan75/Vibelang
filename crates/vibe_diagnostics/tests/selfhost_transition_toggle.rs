use std::env;
use std::sync::{Mutex, OnceLock};

use vibe_diagnostics::{diagnostics_sort_mode_label, Diagnostic, Diagnostics, Severity, Span};

#[test]
fn promoted_diagnostics_mode_matches_host_ordering() {
    let _guard = env_lock().lock().expect("lock env mutex");
    clear_mode_env();

    let host = sorted_fixture();
    env::set_var("VIBE_DIAGNOSTICS_SORT_MODE", "selfhost-default");
    let promoted = sorted_fixture();

    assert_eq!(
        diagnostics_sort_mode_label(),
        "selfhost-default",
        "expected promoted diagnostics mode label"
    );
    assert_eq!(
        host, promoted,
        "promoted diagnostics ordering should remain parity-equivalent to host ordering"
    );
    clear_mode_env();
}

#[test]
fn fallback_toggle_forces_host_mode_immediately() {
    let _guard = env_lock().lock().expect("lock env mutex");
    clear_mode_env();

    env::set_var("VIBE_DIAGNOSTICS_SORT_MODE", "selfhost-default");
    assert_eq!(diagnostics_sort_mode_label(), "selfhost-default");

    env::set_var("VIBE_SELFHOST_FORCE_HOST_FALLBACK", "1");
    assert_eq!(
        diagnostics_sort_mode_label(),
        "host-fallback",
        "fallback toggle should override promoted mode immediately"
    );

    let fallback_sorted = sorted_fixture();
    env::remove_var("VIBE_SELFHOST_FORCE_HOST_FALLBACK");
    let promoted_sorted = sorted_fixture();
    assert_eq!(
        fallback_sorted, promoted_sorted,
        "fallback and promoted orderings should remain parity-equivalent for this candidate"
    );
    clear_mode_env();
}

fn sorted_fixture() -> Vec<(String, Severity, Span)> {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        "E3001",
        Severity::Info,
        "info",
        Span::new(5, 2, 5, 6),
    ));
    diagnostics.push(Diagnostic::new(
        "E1001",
        Severity::Error,
        "error",
        Span::new(1, 1, 1, 5),
    ));
    diagnostics.push(Diagnostic::new(
        "E3001",
        Severity::Warning,
        "warning",
        Span::new(5, 2, 5, 6),
    ));
    diagnostics
        .into_sorted()
        .into_iter()
        .map(|d| (d.code, d.severity, d.span))
        .collect()
}

fn clear_mode_env() {
    env::remove_var("VIBE_DIAGNOSTICS_SORT_MODE");
    env::remove_var("VIBE_SELFHOST_FORCE_HOST_FALLBACK");
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
