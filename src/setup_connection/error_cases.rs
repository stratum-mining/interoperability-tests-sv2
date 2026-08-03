use std::net::SocketAddr;

use integration_tests_sv2::{
    interceptor::MessageDirection,
    mock_roles::{MockDownstream, WithSetup},
    sniffer::Sniffer,
    start_sniffer,
};
use stratum_apps::stratum_core::{
    common_messages_sv2::{
        Protocol, SetupConnectionError, ERROR_CODE_SETUP_CONNECTION_MISSING_DECLARE_TX_DATA_FLAG,
        ERROR_CODE_SETUP_CONNECTION_UNSUPPORTED_PROTOCOL, MESSAGE_TYPE_SETUP_CONNECTION,
        MESSAGE_TYPE_SETUP_CONNECTION_ERROR,
    },
    parsers_sv2::{AnyMessage, CommonMessages},
};

/// Sends a `SetupConnection` with a protocol the target does not support and
/// asserts that it is rejected with `"unsupported-protocol"`.
pub async fn test_rejected_unsupported_protocol(
    target: SocketAddr,
    wrong_protocol: Protocol,
) -> (SetupConnectionError<'static>, Sniffer<'static>) {
    test_rejected_setup(target, wrong_protocol, 0, ERROR_CODE_SETUP_CONNECTION_UNSUPPORTED_PROTOCOL)
        .await
}

/// Sends a `SetupConnection` to a JDS with `JobDeclarationProtocol` but without
/// the `DECLARE_TX_DATA` flag, and asserts that it is rejected with
/// `"missing-declare-tx-data-flag"`.
pub async fn test_rejected_missing_declare_tx_data_flag(
    target: SocketAddr,
) -> (SetupConnectionError<'static>, Sniffer<'static>) {
    test_rejected_setup(
        target,
        Protocol::JobDeclarationProtocol,
        0,
        ERROR_CODE_SETUP_CONNECTION_MISSING_DECLARE_TX_DATA_FLAG,
    )
    .await
}

/// Sends a `SetupConnection` with arbitrary parameters against the given
/// target and asserts that it is rejected with the expected error code.
///
/// Returns the `SetupConnectionError` for further inspection.
pub async fn test_rejected_setup(
    target: SocketAddr,
    protocol: Protocol,
    flags: u32,
    expected_error_code: &str,
) -> (SetupConnectionError<'static>, Sniffer<'static>) {
    let (sniffer, sniffer_addr) =
        start_sniffer("setup_error", target, false, vec![], None);

    let _downstream = MockDownstream::new(
        sniffer_addr,
        WithSetup::yes_with_defaults(protocol, flags),
    )
    .start()
    .await;

    sniffer
        .wait_for_message_type(MessageDirection::ToUpstream, MESSAGE_TYPE_SETUP_CONNECTION)
        .await;

    sniffer
        .wait_for_message_type(
            MessageDirection::ToDownstream,
            MESSAGE_TYPE_SETUP_CONNECTION_ERROR,
        )
        .await;

    let response = sniffer.next_message_from_upstream();
    let error = match response {
        Some((_, AnyMessage::Common(CommonMessages::SetupConnectionError(msg)))) => msg,
        other => panic!(
            "Expected SetupConnectionError, got: {:?}",
            other.map(|(_, m)| m)
        ),
    };

    assert_error_code(&error, expected_error_code);

    (error, sniffer)
}

/// Asserts that the error code in a `SetupConnectionError` matches the
/// expected value.
pub fn assert_error_code(error: &SetupConnectionError<'_>, expected: &str) {
    let actual = error.error_code.as_utf8_or_hex();
    assert_eq!(
        actual, expected,
        "SetupConnectionError.error_code mismatch: got \"{actual}\", expected \"{expected}\""
    );
}

/// Asserts that `flags` in a `SetupConnectionError` equals the expected
/// value.
pub fn assert_error_flags(error: &SetupConnectionError<'_>, expected: u32) {
    assert_eq!(
        error.flags, expected,
        "SetupConnectionError.flags mismatch: got {:#010x}, expected {:#010x}",
        error.flags, expected
    );
}
