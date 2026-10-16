use interoperability_tests_sv2::scenarios::ScenarioReport;

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
