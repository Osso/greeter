use serde::Deserialize;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

const CONFIG_PATH: &str = "/etc/greeter.toml";

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_session: Option<String>,
    #[serde(default)]
    pub default_user: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let path = Path::new(CONFIG_PATH);
        if !path.exists() {
            info!("No config file at {CONFIG_PATH}, using defaults");
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    info!("Loaded config from {CONFIG_PATH}");
                    config
                }
                Err(e) => {
                    warn!("Failed to parse {CONFIG_PATH}: {e}");
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {CONFIG_PATH}: {e}");
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.default_session.is_none());
    }

    #[test]
    fn parse_default_session() {
        let config: Config = toml::from_str(r#"default_session = "niri""#).unwrap();
        assert_eq!(config.default_session, Some("niri".to_string()));
    }

    #[test]
    fn parse_default_user() {
        let config: Config = toml::from_str(r#"default_user = "osso""#).unwrap();
        assert_eq!(config.default_user, Some("osso".to_string()));
    }

    #[test]
    fn parse_full_config() {
        let content = r#"
default_session = "niri"
default_user = "osso"
"#;
        let config: Config = toml::from_str(content).unwrap();
        assert_eq!(config.default_session, Some("niri".to_string()));
        assert_eq!(config.default_user, Some("osso".to_string()));
    }
}
