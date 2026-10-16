//! `OpenExtendedMiningChannel` and `.Success` (M-OEMC-*, M-OEMCS-*).

use stratum_apps::stratum_core::{
    mining_sv2::{
        OpenExtendedMiningChannelOwned, MESSAGE_TYPE_OPEN_EXTENDED_MINING_CHANNEL_SUCCESS,
    },
    parsers_sv2::{AnyMessageOwned, MiningOwned},
};

use super::super::ScenarioResult;
use crate::{client::TestClient, endpoint::Endpoint, scenarios::ScenarioFn};

const REQUEST_ID: u32 = 1;
const MIN_EXTRANONCE_SIZE: u16 = 4;

// ---------------------------------------------------------------------------
// M-OEMCS-1 — `request_id` echo on `OpenExtendedMiningChannel.Success`
// ---------------------------------------------------------------------------

/// The endpoint MUST echo `request_id` in `OpenExtendedMiningChannel.Success`.
///
/// Covers: M-OEMCS-1
pub async fn request_id_echo(endpoint: Endpoint) -> ScenarioResult {
    let client = TestClient::connect(&endpoint).await?;

    let open = OpenExtendedMiningChannelOwned {
        request_id: REQUEST_ID,
        user_identity: endpoint.user_identity.clone().try_into().unwrap(),
        nominal_hash_rate: 1_000_000.0,
        max_target: [0xff_u8; 32].into(),
        min_extranonce_size: MIN_EXTRANONCE_SIZE,
    };
    client
        .send(AnyMessageOwned::Mining(
            MiningOwned::OpenExtendedMiningChannel(open),
        ))
        .await?;

    let msg = client
        .expect_from_server(MESSAGE_TYPE_OPEN_EXTENDED_MINING_CHANNEL_SUCCESS)
        .await?;
    match msg {
        AnyMessageOwned::Mining(MiningOwned::OpenExtendedMiningChannelSuccess(m)) => {
            if m.request_id != REQUEST_ID {
                return Err(
                    format!("request_id must echo {REQUEST_ID}, got {}", m.request_id).into(),
                );
            }
            Ok(Some(format!(
                "channel_id={}, group_channel_id={}, extranonce_size={}",
                m.channel_id, m.group_channel_id, m.extranonce_size
            )))
        }
        other => Err(format!("expected OpenExtendedMiningChannelSuccess, got {other}").into()),
    }
}

// ---------------------------------------------------------------------------
// M-OEMCS-3 — `extranonce_size >= min_extranonce_size`
// ---------------------------------------------------------------------------

/// The endpoint MUST honour `min_extranonce_size` (§5.3.4, §5.3.5).
///
/// Covers: M-OEMCS-3
pub async fn extranonce_size_at_least_min(endpoint: Endpoint) -> ScenarioResult {
    let client = TestClient::connect(&endpoint).await?;

    let open = OpenExtendedMiningChannelOwned {
        request_id: REQUEST_ID,
        user_identity: endpoint.user_identity.clone().try_into().unwrap(),
        nominal_hash_rate: 1_000_000.0,
        max_target: [0xff_u8; 32].into(),
        min_extranonce_size: MIN_EXTRANONCE_SIZE,
    };
    client
        .send(AnyMessageOwned::Mining(
            MiningOwned::OpenExtendedMiningChannel(open),
        ))
        .await?;

    let msg = client
        .expect_from_server(MESSAGE_TYPE_OPEN_EXTENDED_MINING_CHANNEL_SUCCESS)
        .await?;
    match msg {
        AnyMessageOwned::Mining(MiningOwned::OpenExtendedMiningChannelSuccess(m)) => {
            if m.extranonce_size < MIN_EXTRANONCE_SIZE {
                return Err(format!(
                    "extranonce_size {} smaller than requested min_extranonce_size {MIN_EXTRANONCE_SIZE}",
                    m.extranonce_size
                )
                .into());
            }
            Ok(Some(format!(
                "channel_id={}, group_channel_id={}, extranonce_size={}",
                m.channel_id, m.group_channel_id, m.extranonce_size
            )))
        }
        other => Err(format!("expected OpenExtendedMiningChannelSuccess, got {other}").into()),
    }
}

// ---------------------------------------------------------------------------
// registry entries for this module
// ---------------------------------------------------------------------------

pub fn entries() -> Vec<(&'static str, ScenarioFn)> {
    vec![
        ("M-OEMCS-1 request-id-echo", |e| {
            Box::pin(request_id_echo(e))
        }),
        ("M-OEMCS-3 extranonce-size-at-least-min", |e| {
            Box::pin(extranonce_size_at_least_min(e))
        }),
    ]
}
