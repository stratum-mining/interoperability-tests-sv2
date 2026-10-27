use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use interoperability_tests_sv2::{app_type::AppType, endpoint::Endpoint, runner::run_suite};

/// Regression test for https://github.com/stratum-mining/interoperability-tests-sv2/pull/2#issuecomment-5185463970
/// A dead endpoint must produce fast all-failed reports — no panic, no 60s stall.
#[tokio::test]
async fn dead_endpoint_fails_fast_without_panicking() {
    // a port nothing listens on
    let addr = SocketAddr::from(([127, 0, 0, 1], 9));
    let endpoint = Endpoint {
        addr,
        app_type: AppType::MiningServer,
        user_identity: "interoperability_tests_sv2".to_string(),
    };

    let start = Instant::now();
    let reports = tokio::time::timeout(Duration::from_secs(15), run_suite(&endpoint))
        .await
        .expect("run_suite stalled on a dead endpoint");
    assert!(
        start.elapsed() < Duration::from_secs(15),
        "dead endpoint took too long to report"
    );

    assert!(!reports.is_empty(), "no scenarios ran");
    for report in &reports {
        assert!(
            report.result.is_err(),
            "scenario {} must fail against a dead endpoint",
            report.id
        );
    }
}

/// An endpoint that accepts TCP but never completes the Noise handshake must
/// be reported as a handshake failure, not as a per-scenario message timeout.
#[tokio::test]
async fn handshake_failure_is_reported_by_name() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        // accept and drop: the initiator's first handshake message hits a closed socket
        while let Ok((socket, _)) = listener.accept().await {
            drop(socket);
        }
    });
    let endpoint = Endpoint {
        addr,
        app_type: AppType::MiningServer,
        user_identity: "interoperability_tests_sv2".to_string(),
    };

    let reports = tokio::time::timeout(Duration::from_secs(15), run_suite(&endpoint))
        .await
        .expect("run_suite stalled on a handshake-refusing endpoint");

    assert!(!reports.is_empty(), "no scenarios ran");
    for report in &reports {
        let err = report.result.as_ref().expect_err("must fail").to_string();
        assert!(
            err.contains("Noise handshake"),
            "scenario {} should report the handshake failure, got: {err}",
            report.id
        );
    }
}

/// An endpoint that accepts TCP and then stays silent must be reported as a
/// handshake failure within the preflight budget, not hang the run.
#[tokio::test]
async fn silent_handshake_is_reported_by_name() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        // accept and hold: the socket stays open, the responder never writes
        let mut held = Vec::new();
        while let Ok((socket, _)) = listener.accept().await {
            held.push(socket);
        }
    });
    let endpoint = Endpoint {
        addr,
        app_type: AppType::MiningServer,
        user_identity: "interoperability_tests_sv2".to_string(),
    };

    let reports = tokio::time::timeout(Duration::from_secs(15), run_suite(&endpoint))
        .await
        .expect("run_suite stalled on a silent endpoint");

    assert!(!reports.is_empty(), "no scenarios ran");
    for report in &reports {
        let err = report.result.as_ref().expect_err("must fail").to_string();
        assert!(
            err.contains("Noise handshake with endpoint timed out"),
            "scenario {} should report the handshake timeout, got: {err}",
            report.id
        );
    }
}
