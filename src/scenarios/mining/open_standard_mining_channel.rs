//! `OpenStandardMiningChannel` and `.Success` (M-OSMC-*, M-OSMCS-*).

use stratum_apps::stratum_core::mining_sv2::OpenStandardMiningChannelOwned;

use super::super::ScenarioResult;
use crate::{client::TestClient, endpoint::Endpoint, scenarios::ScenarioFn};

const REQUEST_ID: u32 = 1;

// ---------------------------------------------------------------------------
// M-OSMCS-1 — `request_id` echo on `OpenStandardMiningChannel.Success`
// ---------------------------------------------------------------------------

/// The endpoint MUST echo `request_id` in `OpenStandardMiningChannel.Success`.
///
/// Covers: M-OSMCS-1
pub async fn request_id_echo(endpoint: Endpoint) -> ScenarioResult {
    let client = TestClient::connect(&endpoint).await?;

    let open = OpenStandardMiningChannelOwned {
        request_id: REQUEST_ID,
        user_identity: endpoint.user_identity.clone().try_into().unwrap(),
        nominal_hash_rate: 1_000_000.0,
        max_target: [0xff_u8; 32].into(),
    };
    let m = client.open_standard_channel(open).await?;
    if m.request_id != REQUEST_ID {
        return Err(format!("request_id must echo {REQUEST_ID}, got {}", m.request_id).into());
    }
    Ok(Some(format!(
        "channel_id={}, group_channel_id={}",
        m.channel_id, m.group_channel_id
    )))
}

// ---------------------------------------------------------------------------
// registry entries for this module
// ---------------------------------------------------------------------------

pub fn entries() -> Vec<(&'static str, ScenarioFn)> {
    vec![("M-OSMCS-1 request-id-echo", |e| {
        Box::pin(request_id_echo(e))
    })]
}
