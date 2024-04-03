use std::path::PathBuf;

use rsa::{pkcs8::EncodePrivateKey, RsaPrivateKey};
use rusqlite::Connection;

pub fn generate_privkey(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        eprintln!("Privkey file already exists!");
        std::process::exit(1);
    }

    println!("Generating private key...");
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048)
        .inspect_err(|e| eprintln!("Failed to generate a key: {e}"))?;
    priv_key
        .write_pkcs8_pem_file(path, rsa::pkcs8::LineEnding::LF)
        .inspect_err(|e| eprintln!("Failed to write a key into a file: {e}"))?;

    println!("Private key generated.");

    Ok(())
}

pub fn prepare_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Preparing database...");
    let conn = Connection::open(path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id TEXT PRIMARY KEY,
            content TEXT,
            date TEXT
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS apps (
            client_id TEXT PRIMARY KEY,
            client_secret TEXT,
            name TEXT,
            redirect_uri TEXT
        )",
        (),
    )?;

    println!("Database is ready.");

    Ok(())
}
