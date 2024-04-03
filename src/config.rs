use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub user: UserConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub db: PathBuf,
    pub url: Url,
    pub name: String,
    pub desc: String,
    pub bind: std::net::IpAddr,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub privkey: PathBuf,
}

pub fn load(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    let toml = fs::read_to_string(path)
        .inspect_err(|e| eprintln!("Failed to load the config file: {e:?}"))?;
    let config: Config = toml::from_str(&toml)
        .inspect_err(|e| eprintln!("Failed to parse the config file: {e:?}"))?;

    if !["http", "https"].contains(&config.server.url.scheme()) {
        return Err("Scheme must be either http or https".into());
    }
    if config.server.url.cannot_be_a_base() {
        return Err("URL is invalid".into());
    }
    if config.server.url.path() != "/" {
        return Err("URL must be the root".into());
    }

    Ok(config)
}
