mod cli;
mod config;
mod server;
mod setup;

use std::sync::OnceLock;

use clap::Parser;
use cli::SubCommand;
use rsa::{pkcs8::DecodePrivateKey, RsaPrivateKey};

pub static PRIVKEY: OnceLock<RsaPrivateKey> = OnceLock::new();

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Arg::parse();

    let Ok(config) = config::load(args.config) else {
        std::process::exit(1);
    };

    if !config.user.privkey.exists() {
        if cli::ask_confirmation("Privkey not found. Generate one now?") {
            setup::genkey(&config.user.privkey)?;
        } else {
            std::process::exit(0);
        }
    }
    let pem = std::fs::read_to_string(&config.user.privkey).unwrap();
    PRIVKEY.set(RsaPrivateKey::from_pkcs8_pem(&pem)?).unwrap();

    match args.subcommand {
        SubCommand::Run => server::serve(config).await?,
        _ => {}
    }

    Ok(())
}
