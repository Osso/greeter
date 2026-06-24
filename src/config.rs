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
        Self::load_from_path(Path::new(CONFIG_PATH), CONFIG_PATH)
    }

    fn load_from_path(path: &Path, label: &str) -> Self {
        if !path.exists() {
            info!("No config file at {label}, using defaults");
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    info!("Loaded config from {label}");
                    config
                }
                Err(e) => {
                    warn!("Failed to parse {label}: {e}");
                    Self::default()
                }
            },
            Err(e) => {
                warn!("Failed to read {label}: {e}");
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

    #[test]
    fn load_from_path_reads_valid_config() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            file.path(),
            "default_session = \"niri\"\ndefault_user = \"osso\"\n",
        )
        .unwrap();

        let config = Config::load_from_path(file.path(), "test config");

        assert_eq!(config.default_session, Some("niri".to_string()));
        assert_eq!(config.default_user, Some("osso".to_string()));
    }

    #[test]
    fn load_from_path_uses_defaults_for_missing_invalid_or_unreadable_config() {
        let missing = std::env::temp_dir().join("greeter-missing-config.toml");
        let missing_config = Config::load_from_path(&missing, "missing");
        assert!(missing_config.default_session.is_none());
        assert!(missing_config.default_user.is_none());

        let invalid = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(invalid.path(), "default_session = ").unwrap();
        let invalid_config = Config::load_from_path(invalid.path(), "invalid");
        assert!(invalid_config.default_session.is_none());

        let unreadable = tempfile::tempdir().unwrap();
        let unreadable_config = Config::load_from_path(unreadable.path(), "unreadable");
        assert!(unreadable_config.default_user.is_none());
    }
}
