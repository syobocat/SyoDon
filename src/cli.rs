pub mod oauth;
pub mod setup;

use std::io::{stdin, stdout, Write};

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Arg {
    #[command(subcommand)]
    pub subcommand: SubCommand,
    #[arg(short, long, value_name = "CONFIG_FILE", default_value = "config.toml")]
    pub config: std::path::PathBuf,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Run,
    Setup,
    Oauth(OauthArgs),
    Post(PostArgs),
    Follow(FollowArgs),
}

#[derive(Args)]
pub struct OauthArgs {
    #[command(subcommand)]
    pub command: OauthCommand,
}

#[derive(Subcommand)]
pub enum OauthCommand {
    Accept { client_id: String },
    Revoke { client_id: String },
}

#[derive(Args)]
pub struct PostArgs {
    #[command(subcommand)]
    pub command: PostCommand,
}

#[derive(Subcommand)]
pub enum PostCommand {
    Create { content: String },
    Delete { id: String },
}

#[derive(Args)]
pub struct FollowArgs {
    #[command(subcommand)]
    pub command: FollowCommand,
}

#[derive(Subcommand)]
pub enum FollowCommand {
    Add { acct: String },
    Delete { acct: String },
}

fn readline() -> String {
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("IO Error.");
    buf.trim().to_owned()
}

pub fn ask_confirmation(message: impl std::fmt::Display) -> bool {
    loop {
        print!("{message} [Y/n]: ");
        stdout().flush().expect("IO Error.");
        match readline().as_str() {
            "Y" | "y" => return true,
            "N" | "n" => return false,
            _ => continue,
        }
    }
}
