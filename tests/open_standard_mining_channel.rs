use std::time::Duration;

use integration_tests_sv2::{
    mocks::{MockServer, ServerSetupConnection},
    utils::get_available_address,
};
use interoperability_tests_sv2::{
    app_type::AppType,
    endpoint::Endpoint,
    scenarios::{mining::open_standard_mining_channel::standard_channel_accepted, ScenarioError},
};
use stratum_apps::stratum_core::{
    common_messages_sv2::{Protocol, MESSAGE_TYPE_SETUP_CONNECTION},
    mining_sv2::{
        OpenMiningChannelErrorOwned, OpenStandardMiningChannelSuccessOwned,
        MESSAGE_TYPE_OPEN_STANDARD_MINING_CHANNEL,
    },
    parsers_sv2::{AnyMessageOwned, MiningOwned},
};

const REQUIRES_FIXED_VERSION: u32 = 1 << 0;
const REQUIRES_EXTENDED_CHANNELS: u32 = 1 << 1;

async fn start_mining_mock(setup: ServerSetupConnection) -> (MockServer, Endpoint) {
    let addr = get_available_address();
    let server = MockServer::new(addr, Some(setup)).await;
    let endpoint = Endpoint {
        addr,
        app_type: AppType::MiningServer,
        user_identity: "interoperability_tests_sv2".to_string(),
    };
    (server, endpoint)
}

async fn next_open_standard_request_id(server: &MockServer) -> u32 {
    server
        .wait_for_message_type_and_clean_queue(0, MESSAGE_TYPE_SETUP_CONNECTION)
        .await;
    server
        .wait_for_message_type(0, MESSAGE_TYPE_OPEN_STANDARD_MINING_CHANNEL)
        .await;
    match server.next_message(0) {
        Some((_, AnyMessageOwned::Mining(MiningOwned::OpenStandardMiningChannel(m)))) => {
            m.request_id
        }
        other => panic!("expected OpenStandardMiningChannel, got {other:?}"),
    }
}

#[tokio::test]
async fn accepts_standard_channel_when_flag_is_absent() {
    let setup = ServerSetupConnection::new([(Protocol::MiningProtocol, 0)]);
    let (server, endpoint) = start_mining_mock(setup).await;

    let scenario = tokio::spawn(standard_channel_accepted(endpoint));
    let request_id = next_open_standard_request_id(&server).await;
    server
        .send(
            0,
            AnyMessageOwned::Mining(MiningOwned::OpenStandardMiningChannelSuccess(
                OpenStandardMiningChannelSuccessOwned {
                    request_id,
                    channel_id: 7,
                    target: [0xff_u8; 32].into(),
                    extranonce_prefix: vec![0x01, 0x02].try_into().unwrap(),
                    group_channel_id: 3,
                },
            )),
        )
        .await;

    let result = scenario.await.unwrap();
    let detail = result.expect("standard channel acceptance should pass");
    assert_eq!(detail.as_deref(), Some("channel_id=7, group_channel_id=3"));
}

#[tokio::test]
async fn fails_when_flag_is_absent_and_server_refuses() {
    let setup = ServerSetupConnection::new([(Protocol::MiningProtocol, 0)]);
    let (server, endpoint) = start_mining_mock(setup).await;

    let scenario = tokio::spawn(standard_channel_accepted(endpoint));
    let request_id = next_open_standard_request_id(&server).await;
    server
        .send(
            0,
            AnyMessageOwned::Mining(MiningOwned::OpenMiningChannelError(
                OpenMiningChannelErrorOwned {
                    request_id,
                    error_code: "standard-channels-not-supported"
                        .to_string()
                        .try_into()
                        .unwrap(),
                },
            )),
        )
        .await;

    let result = scenario.await.unwrap();
    match result {
        Err(ScenarioError::Failed(msg)) => {
            assert!(
                msg.contains("standard-channels-not-supported"),
                "msg: {msg}"
            );
        }
        other => panic!("expected ScenarioError::Failed, got {other:?}"),
    }
}

#[tokio::test]
async fn fails_when_only_unrelated_flag_is_set_and_server_refuses() {
    let setup = ServerSetupConnection::new([(Protocol::MiningProtocol, REQUIRES_FIXED_VERSION)]);
    let (server, endpoint) = start_mining_mock(setup).await;

    let scenario = tokio::spawn(standard_channel_accepted(endpoint));
    let request_id = next_open_standard_request_id(&server).await;
    server
        .send(
            0,
            AnyMessageOwned::Mining(MiningOwned::OpenMiningChannelError(
                OpenMiningChannelErrorOwned {
                    request_id,
                    error_code: "channel-capacity-exhausted".to_string().try_into().unwrap(),
                },
            )),
        )
        .await;

    let result = scenario.await.unwrap();
    assert!(
        matches!(result, Err(ScenarioError::Failed(_))),
        "expected ScenarioError::Failed, got {result:?}"
    );
}

#[tokio::test]
async fn not_exercised_when_requires_extended_channels_is_set() {
    let setup =
        ServerSetupConnection::new([(Protocol::MiningProtocol, REQUIRES_EXTENDED_CHANNELS)]);
    let (server, endpoint) = start_mining_mock(setup).await;

    let result = standard_channel_accepted(endpoint).await;
    assert!(
        matches!(result, Err(ScenarioError::NotExercised(_))),
        "expected ScenarioError::NotExercised, got {result:?}"
    );
    assert!(
        server
            .assert_message_not_present(
                0,
                MESSAGE_TYPE_OPEN_STANDARD_MINING_CHANNEL,
                Duration::from_millis(200),
            )
            .await,
        "client must not send OpenStandardMiningChannel after REQUIRES_EXTENDED_CHANNELS (§5.3.1)"
    );
}

#[tokio::test]
async fn not_exercised_when_setup_connection_is_rejected() {
    let setup = ServerSetupConnection::new([]);
    let (_server, endpoint) = start_mining_mock(setup).await;

    let result = standard_channel_accepted(endpoint).await;
    assert!(
        matches!(result, Err(ScenarioError::NotExercised(_))),
        "expected ScenarioError::NotExercised, got {result:?}"
    );
}
