use ai_flow_synth::utils::{LogConfig, MongoConfig};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub frontend_config: FrontendConfig,
    pub backend_config: BackendConfig,
    pub log_config: LogConfig,
    pub mongo_config: MongoConfig,
}

impl Config {
    pub fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct FrontendConfig {
    pub cors: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub address: String,
    pub jwt: Jwt,
}

#[derive(Debug, Deserialize)]
pub struct Jwt {
    pub access_secret: String,
    pub refresh_secret: String,
    // pub access_expiration: Option<i64>, // in seconds
    // pub refresh_expiration: Option<i64>, // in seconds
}
