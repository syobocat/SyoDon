use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use rand::{thread_rng, RngCore};
use rusqlite::Connection;

pub fn accept(client_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;

    let mut code = [0u8; 32];
    thread_rng().fill_bytes(&mut code);
    let code_base64 = BASE64_URL_SAFE_NO_PAD.encode(code);

    let conn = Connection::open(db)?;
    conn.execute(
        "UPDATE apps SET code = ?2 WHERE client_id = ?1",
        (client_id, code_base64),
    )?;

    Ok(())
}

pub fn revoke(client_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;

    let conn = Connection::open(db)?;
    conn.execute(
        "UPDATE apps SET code = NULL WHERE client_id = ?1",
        [&client_id],
    )?;
    conn.execute("DELETE FROM token WHERE issuer = ?1", [client_id])?;

    Ok(())
}
