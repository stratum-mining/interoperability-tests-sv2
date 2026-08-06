//! TOML batch configuration for the `sv2-compliance` CLI.

use std::{fs, net::SocketAddr, path::Path};

use serde::Deserialize;

use crate::{app_type::AppType, endpoint::Endpoint};

/// A set of [`Site`]s to test, loaded from a TOML file on disk.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub site: Vec<Site>,
}

impl Config {
    /// Loads a [`Config`] from the TOML file at `path`.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        toml::from_str(&raw).map_err(|e| format!("failed to parse {}: {e}", path.display()))
    }

    /// One [`Target`] per configured address: `mining_server_address` is tested as a
    /// Mining Protocol Server, `job_declaration_server_address` as a Job Declarator Server and
    /// `template_provider_address` as a Template Provider.
    pub fn targets(&self) -> Vec<Target> {
        let mut out = Vec::new();
        for site in &self.site {
            let user_identity = site
                .user_identity
                .clone()
                .unwrap_or_else(|| "interoperability_tests_sv2".to_string());
            for (addr, app_type) in [
                (site.mining_server_address, AppType::MiningServer),
                (
                    site.job_declaration_server_address,
                    AppType::JobDeclarationServer,
                ),
                (site.template_provider_address, AppType::TemplateProvider),
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
#[serde(deny_unknown_fields)]
pub struct Site {
    pub name: String,
    pub mining_server_address: Option<SocketAddr>,
    pub job_declaration_server_address: Option<SocketAddr>,
    pub template_provider_address: Option<SocketAddr>,
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
mining_server_address = "51.161.49.55:34255"
job_declaration_server_address = "51.161.49.55:34256"
template_provider_address = "51.161.49.55:8442"

[[site]]
name = "Empty Site"
"#;
        let config: Config = toml::from_str(raw).expect("valid toml");
        let targets = config.targets();
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].endpoint.app_type, AppType::MiningServer);
        assert_eq!(targets[1].endpoint.app_type, AppType::JobDeclarationServer);
        assert_eq!(targets[2].endpoint.app_type, AppType::TemplateProvider);
    }

    /// A misspelled field must fail to parse rather than leave a site with
    /// nothing to test.
    #[test]
    fn rejects_unknown_fields() {
        let raw = r#"
[[site]]
name = "Typo"
mining_server_adress = "51.161.49.55:34255"
"#;
        assert!(toml::from_str::<Config>(raw).is_err());
    }
}
