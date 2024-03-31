use std::path::PathBuf;

use rsa::{pkcs8::EncodePrivateKey, RsaPrivateKey};

pub fn genkey(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
