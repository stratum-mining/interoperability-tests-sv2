use std::net::SocketAddr;

use interoperability_tests_sv2::{
    app_type::AppType, endpoint::Endpoint, runner::run_suite, scenarios::ScenarioError,
};

/// Runs the suite against a remote endpoint. Off by default: set
/// `SV2_TEST_ENDPOINT=host:port` (and optionally `SV2_TEST_USER_IDENTITY`,
/// `SV2_TEST_APP_TYPE=mining-server|job-declaration-server|template-provider`,
/// defaulting to `mining-server`) to enable.
#[tokio::test]
async fn suite_against_remote_endpoint() {
    let addr: SocketAddr = match std::env::var("SV2_TEST_ENDPOINT") {
        Ok(addr) => addr.parse().expect("invalid SV2_TEST_ENDPOINT"),
        Err(_) => return,
    };
    let app_type = match std::env::var("SV2_TEST_APP_TYPE").as_deref() {
        Ok("mining-server") | Err(_) => AppType::MiningServer,
        Ok("job-declaration-server") => AppType::JobDeclarationServer,
        Ok("template-provider") => AppType::TemplateProvider,
        Ok(other) => panic!("invalid SV2_TEST_APP_TYPE: {other}"),
    };
    let user_identity = std::env::var("SV2_TEST_USER_IDENTITY")
        .unwrap_or_else(|_| "interoperability_tests_sv2".into());

    let endpoint = Endpoint {
        addr,
        app_type,
        user_identity,
    };
    let reports = run_suite(&endpoint).await;

    for report in &reports {
        match &report.result {
            Ok(detail) => match detail {
                Some(d) => println!("PASS {} ({d})", report.id),
                None => println!("PASS {}", report.id),
            },
            Err(ScenarioError::NotExercised(reason)) => {
                println!("NOT EXERCISED {}: {reason}", report.id)
            }
            Err(e) => println!("FAIL {}: {e}", report.id),
        }
    }
    let failures: Vec<_> = reports
        .iter()
        .filter(|r| r.failed())
        .map(|r| (r.id, r.result.as_ref().unwrap_err()))
        .collect();
    assert!(failures.is_empty(), "failed scenarios: {failures:?}");
}
