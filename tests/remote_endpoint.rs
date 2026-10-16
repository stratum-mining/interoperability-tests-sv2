use std::net::SocketAddr;

use interoperability_tests_sv2::{app_type::AppType, endpoint::Endpoint, runner::run_suite};

/// Runs the suite against a remote endpoint. Off by default: set
/// `SV2_TEST_ENDPOINT=host:port` (and optionally `SV2_TEST_USER_IDENTITY`,
/// `SV2_TEST_APP_TYPE=pool|jds|tp`) to enable.
#[tokio::test]
async fn suite_against_remote_endpoint() {
    let addr: SocketAddr = match std::env::var("SV2_TEST_ENDPOINT") {
        Ok(addr) => addr.parse().expect("invalid SV2_TEST_ENDPOINT"),
        Err(_) => return,
    };
    let app_type = match std::env::var("SV2_TEST_APP_TYPE").as_deref() {
        Ok("jds") => AppType::JobDeclaratorServer,
        Ok("tp") => AppType::TemplateProvider,
        _ => AppType::MiningProtocolServer,
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
            Err(e) => println!("FAIL {}: {e}", report.id),
        }
    }
    let failures: Vec<_> = reports
        .iter()
        .filter_map(|r| r.result.as_ref().err().map(|e| (r.id, e)))
        .collect();
    assert!(failures.is_empty(), "failed scenarios: {failures:?}");
}
