mod common;

use integration_tests_sv2::{
    start_pool, start_template_provider, start_tracing, sv2_tp_config,
    template_provider::DifficultyLevel,
};
use interoperability_tests_sv2::{app_type::AppType, endpoint::Endpoint, runner::run_suite};

#[tokio::test]
async fn suite_passes_against_sri_pool() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

    let endpoint = Endpoint {
        addr: pool_addr,
        app_type: AppType::MiningProtocolServer,
        user_identity: "interoperability_tests_sv2".to_string(),
    };
    let reports = run_suite(&endpoint).await;
    pool.shutdown().await;

    common::assert_suite(&reports);
}
