use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use clap::ValueEnum;
use stratum_apps::stratum_core::common_messages_sv2::Protocol;

/// The type of SV2 application under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AppType {
    Pool,
    JobDeclaratorServer,
    TemplateProvider,
}

impl AppType {
    /// Returns `(happy_protocol, wrong_protocol_1, wrong_protocol_2)` for this app type.
    pub fn use_protocols(&self) -> (Protocol, Protocol, Protocol) {
        match self {
            AppType::Pool => (
                Protocol::MiningProtocol,
                Protocol::TemplateDistributionProtocol,
                Protocol::JobDeclarationProtocol,
            ),
            AppType::JobDeclaratorServer => (
                Protocol::JobDeclarationProtocol,
                Protocol::MiningProtocol,
                Protocol::TemplateDistributionProtocol,
            ),
            AppType::TemplateProvider => (
                Protocol::TemplateDistributionProtocol,
                Protocol::MiningProtocol,
                Protocol::JobDeclarationProtocol,
            ),
        }
    }

    /// Returns the flags that a well-behaved client uses when setting up a
    /// connection to this app type.
    pub fn default_flags(&self) -> u32 {
        match self {
            // REQUIRES_STANDARD_JOBS etc. are optional; 0 is accepted.
            AppType::Pool => 0,
            // DECLARE_TX_DATA is required by the JDS.
            AppType::JobDeclaratorServer => 0b0001,
            // No flags defined for TemplateDistributionProtocol yet.
            AppType::TemplateProvider => 0,
        }
    }
}

type ScenarioFn = Box<
    dyn Fn(SocketAddr, AppType) -> Pin<Box<dyn Future<Output = Vec<TestResult>> + Send>>
        + Send
        + Sync,
>;

/// The outcome of a single test within a scenario.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

impl TestResult {
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            detail: String::new(),
        }
    }

    pub fn pass_with(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            detail: detail.into(),
        }
    }

    pub fn fail(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            detail: detail.into(),
        }
    }
}

/// A named collection of related tests against a single SV2 endpoint.
pub struct Scenario {
    pub name: &'static str,
    pub description: &'static str,
    pub run: ScenarioFn,
}

impl Scenario {
    pub fn new(
        name: &'static str,
        description: &'static str,
        run: impl (Fn(SocketAddr, AppType) -> Pin<Box<dyn Future<Output = Vec<TestResult>> + Send>>)
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            name,
            description,
            run: Box::new(run),
        }
    }

    pub async fn execute(&self, endpoint: SocketAddr, app_type: AppType) -> Vec<TestResult> {
        (self.run)(endpoint, app_type).await
    }
}
