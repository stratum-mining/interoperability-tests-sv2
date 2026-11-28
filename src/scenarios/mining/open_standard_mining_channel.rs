//! `OpenStandardMiningChannel` and `.Success` (M-OSMC-*, M-OSMCS-*).

use stratum_apps::stratum_core::{
    mining_sv2::OpenStandardMiningChannelOwned,
    parsers_sv2::{AnyMessageOwned, CommonMessagesOwned, MiningOwned},
};

use super::super::ScenarioResult;
use crate::{
    client::TestClient,
    endpoint::Endpoint,
    scenarios::{ScenarioError, ScenarioFn},
};

const REQUEST_ID: u32 = 1;
/// Mining Protocol `SetupConnection.Success.flags` bit 1 (§5.3.1).
const REQUIRES_EXTENDED_CHANNELS: u32 = 1 << 1;

// ---------------------------------------------------------------------------
// M-OSMC-4 — standard channel accepted without `REQUIRES_EXTENDED_CHANNELS`
// ---------------------------------------------------------------------------

/// §5.2.2: servers MUST accept Standard Channels unless they set
/// `REQUIRES_EXTENDED_CHANNELS` in `SetupConnection.Success` (§5.3.1).
///
/// Covers: M-OSMC-4
pub async fn standard_channel_accepted(endpoint: Endpoint) -> ScenarioResult {
    let client = TestClient::connect(&endpoint).await?;

    match client.next_from_server().await? {
        (_, AnyMessageOwned::Common(CommonMessagesOwned::SetupConnectionSuccess(m))) => {
            if (m.flags & REQUIRES_EXTENDED_CHANNELS) != 0 {
                return Err(ScenarioError::not_exercised(
                    "server set REQUIRES_EXTENDED_CHANNELS in SetupConnection.Success",
                ));
            }
        }
        (_, AnyMessageOwned::Common(CommonMessagesOwned::SetupConnectionError(m))) => {
            return Err(ScenarioError::not_exercised(format!(
                "SetupConnection rejected (error_code=\"{}\")",
                m.error_code.as_utf8_or_hex()
            )));
        }
        (msg_type, other) => {
            return Err(format!(
                "expected SetupConnectionSuccess or SetupConnectionError, got type 0x{msg_type:02x}: {other}"
            )
            .into());
        }
    }

    client
        .send(AnyMessageOwned::Mining(
            MiningOwned::OpenStandardMiningChannel(open_standard_request(&endpoint)),
        ))
        .await?;

    match client.next_from_server().await? {
        (_, AnyMessageOwned::Mining(MiningOwned::OpenStandardMiningChannelSuccess(m))) => {
            Ok(Some(format!(
                "channel_id={}, group_channel_id={}",
                m.channel_id, m.group_channel_id
            )))
        }
        (_, AnyMessageOwned::Mining(MiningOwned::OpenMiningChannelError(e))) => Err(format!(
            "server did not set REQUIRES_EXTENDED_CHANNELS yet refused standard channel (error_code=\"{}\")",
            e.error_code.as_utf8_or_hex()
        )
        .into()),
        (msg_type, other) => Err(format!(
            "expected OpenStandardMiningChannelSuccess or OpenMiningChannelError, got type 0x{msg_type:02x}: {other}"
        )
        .into()),
    }
}

// ---------------------------------------------------------------------------
// M-OSMCS-1 — `request_id` echo on `OpenStandardMiningChannel.Success`
// ---------------------------------------------------------------------------

/// The endpoint MUST echo `request_id` in `OpenStandardMiningChannel.Success`.
///
/// Covers: M-OSMCS-1
pub async fn request_id_echo(endpoint: Endpoint) -> ScenarioResult {
    let client = TestClient::connect(&endpoint).await?;

    let m = client
        .open_standard_channel(open_standard_request(&endpoint))
        .await?;
    if m.request_id != REQUEST_ID {
        return Err(format!("request_id must echo {REQUEST_ID}, got {}", m.request_id).into());
    }
    Ok(Some(format!(
        "channel_id={}, group_channel_id={}",
        m.channel_id, m.group_channel_id
    )))
}

fn open_standard_request(endpoint: &Endpoint) -> OpenStandardMiningChannelOwned {
    OpenStandardMiningChannelOwned {
        request_id: REQUEST_ID,
        user_identity: endpoint.user_identity.clone().try_into().unwrap(),
        nominal_hash_rate: 1_000_000.0,
        max_target: [0xff_u8; 32].into(),
    }
}

// ---------------------------------------------------------------------------
// registry entries for this module
// ---------------------------------------------------------------------------

pub fn entries() -> Vec<(&'static str, ScenarioFn)> {
    vec![
        ("M-OSMC-4 standard-channel-accepted", |e| {
            Box::pin(standard_channel_accepted(e))
        }),
        ("M-OSMCS-1 request-id-echo", |e| {
            Box::pin(request_id_echo(e))
        }),
    ]
}
