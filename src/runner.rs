use crate::{
    app_type::AppType,
    endpoint::Endpoint,
    scenarios::{self, ScenarioReport},
};

/// Runs the suite applicable to the endpoint's app type, collecting all
/// failures.
///
/// Each scenario runs in its own task on a fresh connection: a panic (ours or
/// `integration_tests_sv2`'s) becomes a failed report instead of killing the
/// run.
pub async fn run_suite(endpoint: &Endpoint) -> Vec<ScenarioReport> {
    let suite = match endpoint.app_type {
        AppType::MiningServer => scenarios::mining_server(),
        AppType::JobDeclarationServer => vec![], // no JDS suite until JDP scenarios land
        AppType::TemplateProvider => vec![],     // no TP suite until a TP test exists
    };

    if let Err(e) = endpoint.preflight().await {
        return suite
            .into_iter()
            .map(|(id, _)| ScenarioReport {
                id,
                result: Err(e.to_string().into()),
            })
            .collect();
    }

    let mut reports = Vec::new();
    for (id, scenario) in suite {
        let result = match tokio::spawn(scenario(endpoint.clone())).await {
            Ok(r) => r,
            Err(e) => Err(e.to_string().into()),
        };
        reports.push(ScenarioReport { id, result });
    }
    reports
}
