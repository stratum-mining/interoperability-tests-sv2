use stratum_apps::stratum_core::common_messages_sv2::Protocol;

/// The type of Sv2 application under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppType {
    MiningServer,
    JobDeclarationServer,
    TemplateProvider,
}

impl AppType {
    /// The sub-protocol a well-behaved client speaks to this app type.
    pub fn protocol(&self) -> Protocol {
        match self {
            AppType::MiningServer => Protocol::MiningProtocol,
            AppType::JobDeclarationServer => Protocol::JobDeclarationProtocol,
            AppType::TemplateProvider => Protocol::TemplateDistributionProtocol,
        }
    }

    /// The flags a well-behaved client sets on `SetupConnection` for this app type.
    pub fn default_flags(&self) -> u32 {
        match self {
            AppType::MiningServer => 0,
            // DECLARE_TX_DATA is required by the JDS.
            AppType::JobDeclarationServer => 0b0001,
            AppType::TemplateProvider => 0,
        }
    }

    /// A sub-protocol this app type is expected to reject (C-SC-7).
    pub fn wrong_protocol(&self) -> Protocol {
        match self {
            AppType::MiningServer => Protocol::JobDeclarationProtocol,
            AppType::JobDeclarationServer => Protocol::TemplateDistributionProtocol,
            AppType::TemplateProvider => Protocol::MiningProtocol,
        }
    }
}
