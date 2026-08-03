use integration_tests_sv2::{
    start_pool, start_pool_with_jds, start_template_provider, start_tracing, sv2_tp_config,
    template_provider::DifficultyLevel,
};
use stratum_apps::stratum_core::common_messages_sv2::Protocol;

use interoperability_tests_sv2::setup_connection::{
    assert_error_code, assert_flags_eq, assert_version_used, test_rejected_missing_declare_tx_data_flag,
    test_rejected_unsupported_protocol, test_successful_setup,
};

#[tokio::test]
async fn pool_accepts_mining_protocol_default_flags() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

    let (success, _sniffer) =
        test_successful_setup(pool_addr, Protocol::MiningProtocol, 0).await;

    assert_version_used(&success, 2);
    assert_flags_eq(&success, 0);

    pool.shutdown().await;
}

#[tokio::test]
async fn pool_accepts_mining_protocol_work_selection() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

    // WORK_SELECTION flag = 0b0010
    let (success, _sniffer) =
        test_successful_setup(pool_addr, Protocol::MiningProtocol, 0b0010).await;

    assert_version_used(&success, 2);
    // Pool responds with REQUIRES_EXTENDED_CHANNELS when work selection is requested
    assert_flags_eq(&success, 0b0010);

    pool.shutdown().await;
}

#[tokio::test]
async fn pool_rejects_template_distribution_protocol() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

    let (error, _sniffer) = test_rejected_unsupported_protocol(
        pool_addr,
        Protocol::TemplateDistributionProtocol,
    )
    .await;

    assert_error_code(&error, "unsupported-protocol");

    pool.shutdown().await;
}

#[tokio::test]
async fn jds_accepts_job_declaration_protocol_with_declare_tx_data_flag() {
    start_tracing();
    let (tp, _tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, _pool_addr, jds_addr, _) =
        start_pool_with_jds(tp.bitcoin_core(), vec![], vec![], false).await;

    // DECLARE_TX_DATA flag = 0b0001 is required by the JDS.
    let (success, _sniffer) =
        test_successful_setup(jds_addr, Protocol::JobDeclarationProtocol, 0b0001).await;

    assert_version_used(&success, 2);

    pool.shutdown().await;
}

#[tokio::test]
async fn jds_rejects_job_declaration_protocol_without_declare_tx_data_flag() {
    start_tracing();
    let (tp, _tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, _pool_addr, jds_addr, _) =
        start_pool_with_jds(tp.bitcoin_core(), vec![], vec![], false).await;

    let (error, _sniffer) = test_rejected_missing_declare_tx_data_flag(jds_addr).await;

    assert_error_code(&error, "missing-declare-tx-data-flag");

    pool.shutdown().await;
}
