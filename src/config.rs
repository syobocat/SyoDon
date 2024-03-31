use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub user: UserConfig,
}

#[derive(Clone, Deserialize)]
pub struct ServerConfig {
    pub db: PathBuf,
    pub host: String,
    pub name: String,
    pub desc: String,
    pub bind: std::net::IpAddr,
    pub port: u16,
}

#[derive(Clone, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub privkey: PathBuf,
}

pub fn load(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    let toml = fs::read_to_string(path)
        .inspect_err(|e| eprintln!("Failed to load the config file: {e:?}"))?;
    let config: Config = toml::from_str(&toml)
        .inspect_err(|e| eprintln!("Failed to parse the config file: {e:?}"))?;

    Ok(config)
}
