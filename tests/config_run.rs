use std::fs;

use integration_tests_sv2::{
    start_pool, start_pool_with_jds, start_template_provider, start_tracing, sv2_tp_config,
    template_provider::DifficultyLevel,
};

use interoperability_tests_sv2::config::Config;
use interoperability_tests_sv2::runner::{build_targets, run_targets};
use interoperability_tests_sv2::scenario::{AppType, Scenario};
use interoperability_tests_sv2::setup_connection;

fn scenarios() -> Vec<Scenario> {
    vec![Scenario::new(
        "SetupConnection",
        "Happy-path and error-case SetupConnection handshakes",
        |endpoint, app_type| Box::pin(setup_connection::run(endpoint, app_type)),
    )]
}

fn write_temp_config(path: &str, contents: &str) {
    fs::write(path, contents).expect("write temp config");
}

#[tokio::test]
async fn config_driven_run_pool_and_jds_all_pass() {
    start_tracing();
    let (tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;
    let (pool_with_jds, _pool_addr, jds_addr, _) =
        start_pool_with_jds(tp.bitcoin_core(), vec![], vec![], false).await;

    let config_path = std::env::temp_dir().join("sv2_compliance_test_config.toml");
    let config_path = config_path.to_str().expect("utf8 path");
    write_temp_config(
        config_path,
        &format!(
            r#"
[[site]]
name = "Local Pool"
pool_address = "{pool_addr}"

[[site]]
name = "Local JDS"
jds_address = "{jds_addr}"
"#
        ),
    );

    let config = Config::from_path(config_path).expect("parse config");
    let targets = build_targets(&config);
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].app_type, AppType::Pool);
    assert_eq!(targets[1].app_type, AppType::JobDeclaratorServer);

    let all_scenarios = scenarios();
    let summary = run_targets(&targets, None, &all_scenarios).await;

    assert_eq!(summary.failed, 0, "all tests should pass");
    assert!(summary.passed > 0);

    pool.shutdown().await;
    pool_with_jds.shutdown().await;
}

#[tokio::test]
async fn config_site_scenario_filter_skips_mismatched_scenario() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

    // The only registered scenario is "SetupConnection"; a filter that does
    // not match it should skip everything for this site.
    let config_path = std::env::temp_dir().join("sv2_compliance_test_filter.toml");
    let config_path = config_path.to_str().expect("utf8 path");
    write_temp_config(
        config_path,
        &format!(
            r#"
[[site]]
name = "Local Pool"
pool_address = "{pool_addr}"
scenarios = ["channel-open"]
"#
        ),
    );

    let config = Config::from_path(config_path).expect("parse config");
    let targets = build_targets(&config);
    let all_scenarios = scenarios();
    let summary = run_targets(&targets, None, &all_scenarios).await;

    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);

    pool.shutdown().await;
}

#[tokio::test]
async fn config_site_ignores_missing_addresses() {
    start_tracing();
    let (_tp, tp_addr) = start_template_provider(None, DifficultyLevel::Low);
    let (pool, _pool_addr, _) = start_pool(sv2_tp_config(tp_addr), vec![], vec![], false).await;

    // No addresses configured -> build_targets produces no targets.
    let config_path = std::env::temp_dir().join("sv2_compliance_test_empty.toml");
    let config_path = config_path.to_str().expect("utf8 path");
    write_temp_config(
        config_path,
        r#"
[[site]]
name = "Empty Site"
"#,
    );

    let config = Config::from_path(config_path).expect("parse config");
    let targets = build_targets(&config);
    assert!(targets.is_empty());

    pool.shutdown().await;
}
