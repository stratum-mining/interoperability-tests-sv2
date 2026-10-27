use interoperability_tests_sv2::scenarios::ScenarioReport;

/// Holds the local SRI pool to a higher bar than the CLI: it opens both
/// channel kinds and sets `REQUIRES_EXTENDED_CHANNELS` only for a client that
/// asked for `REQUIRES_WORK_SELECTION`, which this suite never does (see
/// `AppType::default_flags`). A scenario that was not exercised therefore means
/// the harness lost a precondition, not that the pool is allowed to skip it.
/// Every report must be a pass.
pub fn assert_suite(reports: &[ScenarioReport]) {
    let failures: Vec<_> = reports
        .iter()
        .filter_map(|r| r.result.as_ref().err().map(|e| (r.id, e)))
        .collect();
    assert!(
        !reports.is_empty() && reports.iter().any(|r| r.result.is_ok()),
        "no scenarios ran"
    );
    assert!(failures.is_empty(), "failed scenarios: {failures:?}");
}
