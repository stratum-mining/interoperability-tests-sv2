use std::time::Duration;

use async_channel::Sender;
use integration_tests_sv2::{
    interceptor::MessageDirection,
    mock_roles::{MockDownstream, WithSetup},
    sniffer::Sniffer,
    start_sniffer,
};
use stratum_apps::stratum_core::{
    common_messages_sv2::SetupConnectionOwned,
    mining_sv2::{OpenStandardMiningChannelOwned, OpenStandardMiningChannelSuccessOwned},
    parsers_sv2::{AnyMessageOwned, MiningOwned},
};

use crate::{endpoint::Endpoint, scenarios::ScenarioError};

/// How long a scenario waits for a message from the endpoint before failing.
pub const DEFAULT_MSG_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// A test client connected to an endpoint under test.
///
/// Wires `MockDownstream -> Sniffer -> endpoint`: messages are driven through
/// [`TestClient::send`] and everything the endpoint sends back is observed via
/// the sniffer (a `MockDownstream` only logs responses, it cannot read them).
///
/// All waits are bounded by our own timeout-bounded polling: the panicking
/// `Sniffer::wait_for_message_type` primitive is deliberately never used.
pub struct TestClient {
    sender: Sender<AnyMessageOwned>,
    sniffer: Sniffer<'static>,
}

impl TestClient {
    /// Connects and performs the `SetupConnection` handshake with the
    /// protocol and flags appropriate for `endpoint.app_type`.
    pub async fn connect(endpoint: &Endpoint) -> Result<Self, ScenarioError> {
        let setup = WithSetup::yes_with_defaults(
            endpoint.app_type.protocol(),
            endpoint.app_type.default_flags(),
        );
        Self::connect_with_setup(endpoint, setup).await
    }

    /// Connects and sends a custom `SetupConnection` (for negotiation edge cases).
    pub async fn connect_with_setup(
        endpoint: &Endpoint,
        setup: WithSetup,
    ) -> Result<Self, ScenarioError> {
        let (sniffer, sniffer_addr) =
            start_sniffer("test-client", endpoint.addr, false, vec![], None);
        let sender = MockDownstream::new(sniffer_addr, setup).start().await;
        Ok(Self { sender, sniffer })
    }

    /// Sends a message to the endpoint.
    pub async fn send(&self, msg: AnyMessageOwned) -> Result<(), ScenarioError> {
        self.sender
            .send(msg)
            .await
            .map_err(|e| format!("failed to send message: {e}").into())
    }

    /// Waits until the endpoint sends a message of `msg_type`, then returns it.
    ///
    /// Messages received before it (if any) are consumed and discarded.
    pub async fn expect_from_server(&self, msg_type: u8) -> Result<AnyMessageOwned, ScenarioError> {
        tokio::time::timeout(DEFAULT_MSG_TIMEOUT, async {
            loop {
                if self
                    .sniffer
                    .has_message_type(MessageDirection::ToDownstream, msg_type)
                {
                    while let Some((t, msg)) = self.sniffer.next_message_from_upstream() {
                        if t == msg_type {
                            return msg;
                        }
                    }
                }
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        })
        .await
        .map_err(|_| {
            ScenarioError::Failed(format!(
                "timeout waiting for message type 0x{msg_type:02x} from endpoint"
            ))
        })
    }

    /// Waits for the next message of any type from the endpoint.
    /// Opens a standard channel. A server that refuses it with
    /// `OpenMiningChannel.Error` (allowed when it set
    /// `REQUIRES_EXTENDED_CHANNELS`, C-SC-5) leaves standard-channel scenarios
    /// *not exercised* rather than failed.
    pub async fn open_standard_channel(
        &self,
        open: OpenStandardMiningChannelOwned,
    ) -> Result<OpenStandardMiningChannelSuccessOwned, ScenarioError> {
        self.send(AnyMessageOwned::Mining(
            MiningOwned::OpenStandardMiningChannel(open),
        ))
        .await?;
        // Skip whatever is still queued from before the open (the
        // `SetupConnection.Success` frame); stop at the open's response.
        loop {
            match self.next_from_server().await? {
                (_, AnyMessageOwned::Mining(MiningOwned::OpenStandardMiningChannelSuccess(m))) => {
                    return Ok(m)
                }
                (_, AnyMessageOwned::Mining(MiningOwned::OpenMiningChannelError(e))) => {
                    return Err(ScenarioError::not_exercised(format!(
                        "server refused a standard channel (error_code=\"{}\")",
                        e.error_code.as_utf8_or_hex()
                    )))
                }
                _ => continue,
            }
        }
    }

    pub async fn next_from_server(&self) -> Result<(u8, AnyMessageOwned), ScenarioError> {
        tokio::time::timeout(DEFAULT_MSG_TIMEOUT, async {
            loop {
                if let Some((t, msg)) = self.sniffer.next_message_from_upstream() {
                    return (t, msg);
                }
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        })
        .await
        .map_err(|_| ScenarioError::Failed("timeout waiting for any message from endpoint".into()))
    }

    /// Asserts the endpoint sends no message of `msg_type` within `duration`.
    pub async fn expect_silence(
        &self,
        msg_type: u8,
        duration: Duration,
    ) -> Result<(), ScenarioError> {
        if self
            .sniffer
            .assert_message_not_present(MessageDirection::ToDownstream, msg_type, duration)
            .await
        {
            Ok(())
        } else {
            Err(format!("expected no message of type 0x{msg_type:02x}, but one arrived").into())
        }
    }
}

/// Builds a `SetupConnection` with custom version range and flags, for
/// negotiation edge cases.
pub fn setup_connection(
    protocol: stratum_apps::stratum_core::common_messages_sv2::Protocol,
    min_version: u16,
    max_version: u16,
    flags: u32,
) -> SetupConnectionOwned {
    SetupConnectionOwned {
        protocol,
        min_version,
        max_version,
        flags,
        endpoint_host: "0.0.0.0".try_into().unwrap(),
        endpoint_port: 0,
        vendor: "interoperability-tests-sv2".try_into().unwrap(),
        hardware_version: "".try_into().unwrap(),
        firmware: "".try_into().unwrap(),
        device_id: "".try_into().unwrap(),
    }
}
