//! `NewMiningJob` (M-NMJ-*).

use stratum_apps::stratum_core::{
    mining_sv2::{NewMiningJobOwned, OpenStandardMiningChannelOwned, MESSAGE_TYPE_NEW_MINING_JOB},
    parsers_sv2::{AnyMessageOwned, MiningOwned},
};

use super::super::ScenarioResult;
use crate::{
    client::TestClient,
    endpoint::Endpoint,
    scenarios::{ScenarioError, ScenarioFn},
};

// ---------------------------------------------------------------------------
// M-NMJ-1 — first post-open message is `NewMiningJob`
// ---------------------------------------------------------------------------

/// §5.3.15: the first message after a standard channel opens MUST be a
/// `NewMiningJob`, whether or not `REQUIRES_STANDARD_JOBS` is set.
///
/// Covers: M-NMJ-1
pub async fn first_message_after_open_is_job(endpoint: Endpoint) -> ScenarioResult {
    first_job(&endpoint).await?;
    Ok(None)
}

// ---------------------------------------------------------------------------
// M-NMJ-2 — first job is a future job
// ---------------------------------------------------------------------------

/// §5.3.15: the first `NewMiningJob` after a standard channel opens MUST have
/// `ntime_start` unset, since no `SetNewPrevHash` exists yet to activate it.
///
/// Covers: M-NMJ-2
pub async fn first_job_is_future_job(endpoint: Endpoint) -> ScenarioResult {
    let job = first_job(&endpoint).await?;
    match job.ntime_start.into_inner() {
        None => Ok(None),
        Some(ntime_start) => Err(format!(
            "first NewMiningJob must be a future job (ntime_start unset), got ntime_start={ntime_start}"
        )
        .into()),
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Opens a standard channel and returns the first message the endpoint sends
/// afterwards, failing unless it is a `NewMiningJob`.
async fn first_job(endpoint: &Endpoint) -> Result<NewMiningJobOwned, ScenarioError> {
    let client = TestClient::connect(endpoint).await?;

    let open = OpenStandardMiningChannelOwned {
        request_id: 1,
        user_identity: endpoint.user_identity.clone().try_into().unwrap(),
        nominal_hash_rate: 1_000_000.0,
        max_target: [0xff_u8; 32].into(),
    };
    client.open_standard_channel(open).await?;

    match client.next_from_server().await? {
        (_, AnyMessageOwned::Mining(MiningOwned::NewMiningJob(job))) => Ok(job),
        (msg_type, _) => Err(format!(
            "expected NewMiningJob (0x{MESSAGE_TYPE_NEW_MINING_JOB:02x}) after standard channel open, got 0x{msg_type:02x}"
        )
        .into()),
    }
}

// ---------------------------------------------------------------------------
// registry entries for this module
// ---------------------------------------------------------------------------

pub fn entries() -> Vec<(&'static str, ScenarioFn)> {
    vec![
        ("M-NMJ-1 first-message-after-open-is-job", |e| {
            Box::pin(first_message_after_open_is_job(e))
        }),
        ("M-NMJ-2 first-job-is-future-job", |e| {
            Box::pin(first_job_is_future_job(e))
        }),
    ]
}
