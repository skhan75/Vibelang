use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};

#[test]
fn host_diagnostics_sort_matches_shadow_contract_cases() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        "E1002",
        Severity::Warning,
        "shadow case warning",
        Span::new(3, 1, 3, 5),
    ));
    diagnostics.push(Diagnostic::new(
        "E1001",
        Severity::Error,
        "shadow case later span",
        Span::new(2, 1, 2, 5),
    ));
    diagnostics.push(Diagnostic::new(
        "E1002",
        Severity::Info,
        "shadow case info",
        Span::new(3, 1, 3, 5),
    ));
    diagnostics.push(Diagnostic::new(
        "E1002",
        Severity::Error,
        "shadow case code rank",
        Span::new(3, 1, 3, 5),
    ));
    diagnostics.push(Diagnostic::new(
        "E1001",
        Severity::Error,
        "shadow case earliest span",
        Span::new(1, 1, 1, 5),
    ));
    diagnostics.push(Diagnostic::new(
        "E1001",
        Severity::Error,
        "shadow case same span lower code",
        Span::new(3, 1, 3, 5),
    ));

    let sorted = diagnostics.into_sorted();
    let observed = sorted
        .iter()
        .map(|d| (d.code.clone(), d.severity, d.span))
        .collect::<Vec<_>>();

    let expected = vec![
        ("E1001".to_string(), Severity::Error, Span::new(1, 1, 1, 5)),
        ("E1001".to_string(), Severity::Error, Span::new(2, 1, 2, 5)),
        ("E1001".to_string(), Severity::Error, Span::new(3, 1, 3, 5)),
        ("E1002".to_string(), Severity::Error, Span::new(3, 1, 3, 5)),
        ("E1002".to_string(), Severity::Warning, Span::new(3, 1, 3, 5)),
        ("E1002".to_string(), Severity::Info, Span::new(3, 1, 3, 5)),
    ];

    assert_eq!(
        observed, expected,
        "host diagnostics ordering should match selfhost shadow ordering contract"
    );
}

#[test]
fn host_diagnostics_sort_is_repeat_run_deterministic() {
    let mut first_input = Diagnostics::default();
    first_input.push(Diagnostic::new(
        "E2002",
        Severity::Warning,
        "warning",
        Span::new(8, 1, 8, 5),
    ));
    first_input.push(Diagnostic::new(
        "E1001",
        Severity::Error,
        "error",
        Span::new(3, 2, 3, 7),
    ));

    let mut second_input = Diagnostics::default();
    second_input.push(Diagnostic::new(
        "E2002",
        Severity::Warning,
        "warning",
        Span::new(8, 1, 8, 5),
    ));
    second_input.push(Diagnostic::new(
        "E1001",
        Severity::Error,
        "error",
        Span::new(3, 2, 3, 7),
    ));

    assert_eq!(
        first_input.to_golden(),
        second_input.to_golden(),
        "diagnostic golden output should be deterministic across repeated runs"
    );
}
