pub mod error_cases;
pub mod happy_path;

pub use error_cases::*;
pub use happy_path::*;

use std::net::SocketAddr;

use stratum_apps::stratum_core::common_messages_sv2::Protocol;

use crate::scenario::{AppType, TestResult};

/// Runs all SetupConnection tests against the given endpoint.
pub async fn run(endpoint: SocketAddr, app_type: AppType) -> Vec<TestResult> {
    let mut results = Vec::new();

    let (happy, wrong1, wrong2) = app_type.use_protocols();
    let default_flags = app_type.default_flags();

    // Happy path
    let result = test_successful_setup_with_result(endpoint, happy, default_flags).await;
    results.push(result);

    // Error case 1: wrong protocol
    let result = test_rejected_unsupported_protocol_with_result(endpoint, wrong1).await;
    results.push(result);

    // Error case 2: another wrong protocol
    let result = test_rejected_unsupported_protocol_with_result(endpoint, wrong2).await;
    results.push(result);

    // App-type-specific error cases
    if let AppType::JobDeclaratorServer = app_type {
        let result = test_rejected_missing_declare_tx_data_flag_with_result(endpoint).await;
        results.push(result);
    }

    results
}

async fn test_successful_setup_with_result(
    endpoint: SocketAddr,
    protocol: Protocol,
    flags: u32,
) -> TestResult {
    let name = format!("SetupConnection ({protocol:?})");
    let (success, _) = happy_path::test_successful_setup(endpoint, protocol, flags).await;
    assert_version_used(&success, 2);
    TestResult::pass_with(&name, format!("version={}", success.used_version))
}

async fn test_rejected_unsupported_protocol_with_result(
    endpoint: SocketAddr,
    wrong_protocol: Protocol,
) -> TestResult {
    let name = format!("SetupConnection ({wrong_protocol:?}) expects rejection");
    let (error, _) =
        error_cases::test_rejected_unsupported_protocol(endpoint, wrong_protocol).await;
    assert_error_code(&error, "unsupported-protocol");
    TestResult::pass(&name)
}

async fn test_rejected_missing_declare_tx_data_flag_with_result(
    endpoint: SocketAddr,
) -> TestResult {
    let name = "SetupConnection (JobDeclarationProtocol, no DECLARE_TX_DATA) expects rejection";
    let (error, _) = error_cases::test_rejected_missing_declare_tx_data_flag(endpoint).await;
    assert_error_code(&error, "missing-declare-tx-data-flag");
    TestResult::pass(name)
}

#[cfg(test)]
mod tests {
    use integration_tests_sv2::{
        start_pool, start_template_provider, start_tracing, sv2_tp_config,
        template_provider::DifficultyLevel,
    };

    use super::*;

    #[tokio::test]
    async fn run_setup_connection_scenario_against_pool() {
        start_tracing();
        let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
        let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

        let results = run(pool_addr, AppType::Pool).await;
        pool.shutdown().await;

        for r in &results {
            assert!(r.passed, "{} failed: {}", r.name, r.detail);
        }
    }
}
