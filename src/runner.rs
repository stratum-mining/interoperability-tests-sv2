use crate::{
    app_type::AppType,
    endpoint::Endpoint,
    scenarios::{ScenarioFn, ScenarioReport},
};

/// Runs the suite applicable to the endpoint's app type, collecting all
/// failures.
///
/// Each scenario runs in its own task on a fresh connection: a panic (ours or
/// `integration_tests_sv2`'s) becomes a failed report instead of killing the
/// run.
pub async fn run_suite(endpoint: &Endpoint) -> Vec<ScenarioReport> {
    run(endpoint, suite_for(endpoint.app_type)).await
}

// ---------------------------------------------------------------------------
// internals
// ---------------------------------------------------------------------------

fn suite_for(app_type: AppType) -> Vec<(&'static str, ScenarioFn)> {
    match app_type {
        AppType::MiningProtocolServer => crate::scenarios::mining_protocol_server(),
        AppType::JobDeclaratorServer => vec![], // no JDS suite until JDP scenarios land
        AppType::TemplateProvider => vec![],    // no TP suite until a TP test exists
    }
}

async fn run(
    endpoint: &Endpoint,
    selected: Vec<(&'static str, ScenarioFn)>,
) -> Vec<ScenarioReport> {
    if let Err(e) = endpoint.preflight().await {
        return selected
            .into_iter()
            .map(|(id, _)| ScenarioReport {
                id,
                result: Err(e.to_string().into()),
            })
            .collect();
    }

    let mut reports = Vec::new();
    for (id, scenario) in selected {
        let result = match tokio::spawn(scenario(endpoint.clone())).await {
            Ok(r) => r,
            Err(e) => Err(e.to_string().into()),
        };
        reports.push(ScenarioReport { id, result });
    }
    reports
}
