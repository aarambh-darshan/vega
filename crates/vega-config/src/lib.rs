use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read Vega.toml from {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid Vega.toml: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Deserialize)]
pub struct VegaConfig {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub features: FeatureConfig,
    #[serde(default)]
    pub ssr: SsrConfig,
    #[serde(default)]
    pub ssg: SsgConfig,
    pub database: Option<DatabaseConfig>,
    pub auth: Option<AuthConfig>,
}

impl Default for VegaConfig {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            server: ServerConfig::default(),
            build: BuildConfig::default(),
            features: FeatureConfig::default(),
            ssr: SsrConfig::default(),
            ssg: SsgConfig::default(),
            database: None,
            auth: None,
        }
    }
}

impl VegaConfig {
    pub fn from_toml(raw: &str) -> Result<Self, ConfigError> {
        toml::from_str(raw).map_err(ConfigError::from)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_toml(&raw)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            base_url: default_base_url(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_out_dir")]
    pub out_dir: String,
    #[serde(default = "default_public_dir")]
    pub public_dir: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            out_dir: default_out_dir(),
            public_dir: default_public_dir(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureConfig {
    #[serde(default)]
    pub tailwind: bool,
    #[serde(default)]
    pub compress: bool,
    #[serde(default)]
    pub source_maps: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            tailwind: false,
            compress: false,
            source_maps: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SsrConfig {
    #[serde(default = "default_true")]
    pub streaming: bool,
}

impl Default for SsrConfig {
    fn default() -> Self {
        Self {
            streaming: default_true(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SsgConfig {
    #[serde(default = "default_concurrency")]
    pub concurrent: usize,
}

impl Default for SsgConfig {
    fn default() -> Self {
        Self {
            concurrent: default_concurrency(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub provider: String,
}

fn default_app_name() -> String {
    "vega-app".to_string()
}

fn default_base_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_out_dir() -> String {
    "dist".to_string()
}

fn default_public_dir() -> String {
    "public".to_string()
}

fn default_true() -> bool {
    true
}

fn default_concurrency() -> usize {
    4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults() {
        let config = VegaConfig::from_toml("").expect("must parse");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.ssg.concurrent, 4);
        assert_eq!(config.app.base_url, "http://localhost:3000");
    }

    #[test]
    fn parse_overrides() {
        let config = VegaConfig::from_toml(
            r#"
            [app]
            name = "demo"

            [server]
            port = 8080

            [features]
            compress = true
            "#,
        )
        .expect("must parse");

        assert_eq!(config.app.name, "demo");
        assert_eq!(config.server.port, 8080);
        assert!(config.features.compress);
    }

    #[test]
    fn parse_invalid() {
        let err =
            VegaConfig::from_toml("[server\nport = 3000").expect_err("invalid toml should fail");
        match err {
            ConfigError::Parse(_) => {}
            _ => panic!("unexpected error variant"),
        }
    }
}
