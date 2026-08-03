use std::net::SocketAddr;

use integration_tests_sv2::{
    interceptor::MessageDirection,
    mock_roles::{MockDownstream, WithSetup},
    sniffer::Sniffer,
    start_sniffer,
};
use stratum_apps::stratum_core::{
    common_messages_sv2::{
        Protocol, SetupConnectionSuccess, MESSAGE_TYPE_SETUP_CONNECTION,
        MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS,
    },
    parsers_sv2::{AnyMessage, CommonMessages},
};

/// Sets up a Sniffer + MockDownstream against the given target, sends a valid
/// `SetupConnection`, waits for the response, and returns the
/// `SetupConnectionSuccess`.
///
/// The caller must eventually drop the returned `Sniffer` with
/// `check_on_drop: false`, or consume all messages from its queues.
pub async fn test_successful_setup(
    target: SocketAddr,
    protocol: Protocol,
    flags: u32,
) -> (SetupConnectionSuccess, Sniffer<'static>) {
    let (sniffer, sniffer_addr) = start_sniffer("setup_happy", target, false, vec![], None);

    let _downstream =
        MockDownstream::new(sniffer_addr, WithSetup::yes_with_defaults(protocol, flags))
            .start()
            .await;

    sniffer
        .wait_for_message_type(MessageDirection::ToUpstream, MESSAGE_TYPE_SETUP_CONNECTION)
        .await;

    sniffer
        .wait_for_message_type(
            MessageDirection::ToDownstream,
            MESSAGE_TYPE_SETUP_CONNECTION_SUCCESS,
        )
        .await;

    let response = sniffer.next_message_from_upstream();
    let success = match response {
        Some((_, AnyMessage::Common(CommonMessages::SetupConnectionSuccess(msg)))) => msg,
        other => panic!(
            "Expected SetupConnectionSuccess, got: {:?}",
            other.map(|(_, m)| m)
        ),
    };

    (success, sniffer)
}

/// Asserts that `used_version` in a `SetupConnectionSuccess` equals the
/// expected value.
pub fn assert_version_used(success: &SetupConnectionSuccess, expected: u16) {
    assert_eq!(
        success.used_version, expected,
        "SetupConnectionSuccess.used_version mismatch: got {}, expected {}",
        success.used_version, expected
    );
}

/// Asserts that `flags` in a `SetupConnectionSuccess` equals the expected
/// value.
pub fn assert_flags_eq(success: &SetupConnectionSuccess, expected: u32) {
    assert_eq!(
        success.flags, expected,
        "SetupConnectionSuccess.flags mismatch: got {:#010x}, expected {:#010x}",
        success.flags, expected
    );
}

/// Asserts that a specific flag bit is set in `SetupConnectionSuccess.flags`.
pub fn assert_flags_has(success: &SetupConnectionSuccess, bit: u32) {
    assert!(
        success.flags & bit != 0,
        "SetupConnectionSuccess.flags expected bit {:#010x} to be set, but flags are {:#010x}",
        bit,
        success.flags
    );
}
