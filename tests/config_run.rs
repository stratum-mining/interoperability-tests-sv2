mod common;

use integration_tests_sv2::{
    start_pool, start_template_provider, start_tracing, sv2_tp_config,
    template_provider::DifficultyLevel,
};
use interoperability_tests_sv2::{config::Config, runner::run_suite};

/// The TOML batch path used by the CLI.
#[tokio::test]
async fn config_driven_run_against_pool() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

    let raw = format!(
        r#"
[[site]]
name = "Local Pool"
pool_address = "{pool_addr}"
"#
    );
    let config: Config = toml::from_str(&raw).expect("valid toml");
    let targets = config.targets();
    assert_eq!(targets.len(), 1);

    for target in &targets {
        let reports = run_suite(&target.endpoint).await;
        common::assert_suite(&reports);
    }

    pool.shutdown().await;
}
