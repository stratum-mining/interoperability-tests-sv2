use std::{fs, net::SocketAddr};

use anyhow::Context;
use serde::Deserialize;

/// A set of [`Site`]s to test, loaded from a TOML config file.
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub site: Vec<Site>,
}

impl Config {
    /// Loads a config from a TOML file on disk.
    pub fn from_path(path: &str) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {path}"))?;
        toml::from_str(&raw).with_context(|| format!("failed to parse config file: {path}"))
    }
}

/// A named deployment of one or more SV2 endpoints.
#[derive(Debug, Deserialize)]
pub struct Site {
    pub name: String,
    pub pool_address: Option<SocketAddr>,
    pub jds_address: Option<SocketAddr>,
    /// Per-site scenario allowlist. When present, only scenarios whose names
    /// partially match an entry run against this site.
    pub scenarios: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let raw = r#"
[[site]]
name = "SRI Pool"
pool_address = "51.161.49.55:34255"
jds_address = "51.161.49.55:34256"
scenarios = ["setup-connection"]

[[site]]
name = "JDS Only"
jds_address = "10.0.0.5:34256"
"#;

        let config: Config = toml::from_str(raw).expect("valid toml");

        assert_eq!(config.site.len(), 2);

        let sri = &config.site[0];
        assert_eq!(sri.name, "SRI Pool");
        assert_eq!(
            sri.pool_address,
            Some("51.161.49.55:34255".parse().expect("valid addr"))
        );
        assert_eq!(
            sri.jds_address,
            Some("51.161.49.55:34256".parse().expect("valid addr"))
        );
        assert_eq!(sri.scenarios, Some(vec!["setup-connection".to_string()]));

        let jds_only = &config.site[1];
        assert_eq!(jds_only.name, "JDS Only");
        assert_eq!(jds_only.pool_address, None);
        assert_eq!(
            jds_only.jds_address,
            Some("10.0.0.5:34256".parse().expect("valid addr"))
        );
        assert_eq!(jds_only.scenarios, None);
    }

    #[test]
    fn site_requires_address() {
        let raw = r#"
[[site]]
name = "Empty Site"
"#;

        let config: Result<Config, _> = toml::from_str(raw);
        assert!(config.is_ok());
        let config = config.expect("valid toml");
        assert_eq!(config.site.len(), 1);
        assert_eq!(config.site[0].pool_address, None);
        assert_eq!(config.site[0].jds_address, None);
    }
}
