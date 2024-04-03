use std::path::PathBuf;

use rsa::{pkcs8::EncodePrivateKey, RsaPrivateKey};
use rusqlite::Connection;

pub fn generate_privkey() -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let privkey = &config.user.privkey;

    if privkey.exists() {
        eprintln!("Privkey file already exists!");
        std::process::exit(1);
    }

    println!("Generating private key...");
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, 2048)
        .inspect_err(|e| eprintln!("Failed to generate a key: {e}"))?;
    priv_key
        .write_pkcs8_pem_file(privkey, rsa::pkcs8::LineEnding::LF)
        .inspect_err(|e| eprintln!("Failed to write a key into a file: {e}"))?;

    println!("Private key generated.");

    Ok(())
}

pub fn prepare_database() -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;

    println!("Preparing database...");
    let conn = Connection::open(db)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            date TEXT NOT NULL
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS apps (
            client_id TEXT PRIMARY KEY,
            client_secret TEXT NOT NULL,
            name TEXT NOT NULL,
            redirect_uri TEXT NOT NULL,
            code TEXT
        )",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS token (
            token TEXT PRIMALY KEY,
            issuer TEXT NOT NULL
        )",
        (),
    )?;

    println!("Database is ready.");

    Ok(())
}
