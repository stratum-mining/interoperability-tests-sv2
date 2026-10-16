//! TOML batch configuration for the `sv2-compliance` CLI.

use std::{fs, net::SocketAddr};

use serde::Deserialize;

use crate::{app_type::AppType, endpoint::Endpoint};

/// A set of [`Site`]s to test, loaded from a TOML file on disk.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub site: Vec<Site>,
}

impl Config {
    pub fn from_path(path: &str) -> Result<Self, String> {
        let raw = fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
        toml::from_str(&raw).map_err(|e| format!("failed to parse {path}: {e}"))
    }

    /// One [`Target`] per configured address: `pool_address` is tested as a
    /// pool, `jds_address` as a Job Declarator Server.
    pub fn targets(&self) -> Vec<Target> {
        let mut out = Vec::new();
        for site in &self.site {
            let user_identity = site
                .user_identity
                .clone()
                .unwrap_or_else(|| "interoperability_tests_sv2".to_string());
            for (addr, app_type) in [
                (site.pool_address, AppType::MiningProtocolServer),
                (site.jds_address, AppType::JobDeclaratorServer),
            ]
            .into_iter()
            .filter_map(|(addr, app_type)| addr.map(|a| (a, app_type)))
            {
                out.push(Target {
                    site_name: site.name.clone(),
                    endpoint: Endpoint {
                        addr,
                        app_type,
                        user_identity: user_identity.clone(),
                    },
                });
            }
        }
        out
    }
}

/// A single endpoint under test, derived from a [`Site`].
#[derive(Debug, Clone)]
pub struct Target {
    pub site_name: String,
    pub endpoint: Endpoint,
}

/// A named deployment of one or more Sv2 endpoints.
#[derive(Debug, Deserialize)]
pub struct Site {
    pub name: String,
    pub pool_address: Option<SocketAddr>,
    pub jds_address: Option<SocketAddr>,
    pub user_identity: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sites_into_endpoints() {
        let raw = r#"
[[site]]
name = "SRI Pool"
pool_address = "51.161.49.55:34255"
jds_address = "51.161.49.55:34256"

[[site]]
name = "Empty Site"
"#;
        let config: Config = toml::from_str(raw).expect("valid toml");
        let targets = config.targets();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].endpoint.app_type, AppType::MiningProtocolServer);
        assert_eq!(targets[1].endpoint.app_type, AppType::JobDeclaratorServer);
    }
}
