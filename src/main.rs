mod cli;
mod config;
mod server;
mod service;

use std::sync::OnceLock;

use clap::Parser;
use cli::{OauthCommand, SubCommand};
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey};

pub static CONFIG: OnceLock<config::Config> = OnceLock::new();
pub static PRIVKEY: OnceLock<RsaPrivateKey> = OnceLock::new();

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Arg::parse();

    let Ok(config) = config::load(args.config) else {
        std::process::exit(1);
    };
    let config = CONFIG.get_or_init(|| config);

    if !config.user.privkey.exists() {
        if cli::ask_confirmation("Privkey not found. Generate one now?") {
            cli::setup::generate_privkey()?;
        } else {
            std::process::exit(0);
        }
    }
    let pem = std::fs::read_to_string(&config.user.privkey).unwrap();
    PRIVKEY.set(RsaPrivateKey::from_pkcs8_pem(&pem)?).unwrap();

    if !config.server.db.exists() {
        if cli::ask_confirmation("Database not found. Create it now?") {
            cli::setup::prepare_database()?;
        } else {
            std::process::exit(0);
        }
    }

    //service::post::create("This is test :)".to_owned()).await?;

    match args.subcommand {
        SubCommand::Run => server::serve().await?,
        SubCommand::Setup => {
            if !config.user.privkey.exists() {
                cli::setup::generate_privkey()?;
            } else {
                println!("Privkey has already been generated. Skipping...");
            }
            cli::setup::prepare_database()?;
        }
        SubCommand::Oauth(oauthcommand) => match oauthcommand.command {
            OauthCommand::Accept { client_id } => cli::oauth::accept(client_id)?,
            OauthCommand::Revoke { client_id } => cli::oauth::revoke(client_id)?,
        },
    }

    Ok(())
}
