use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

use reqwest::Client;
use rusqlite::{Connection, OptionalExtension};
use url::Url;

use crate::structs::Method;

use super::httpsig::create_header;

pub static INBOXES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub fn get_inboxes() -> &'static Mutex<HashSet<String>> {
    INBOXES.get_or_init(|| {
        let config = crate::CONFIG.get().unwrap();
        let db = &config.server.db;

        let conn = Connection::open(db).expect("failed to connect to the database");
        let mut stmt = conn
            .prepare("SELECT shared_inbox, inbox FROM followers")
            .expect("failed to communicate with the database");
        let query = stmt
            .query_map([], |row| row.get(0).or(row.get(1)))
            .optional()
            .expect("failed to load followers");

        let hashset = if let Some(query) = query {
            query.filter_map(|res| res.ok()).collect()
        } else {
            HashSet::new()
        };

        Mutex::new(hashset)
    })
}

pub async fn deliver_to_followers(
    json: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let inboxes = get_inboxes().lock()?;
    for inbox in inboxes.iter() {
        let url: Url = inbox.parse()?;
        let header = create_header(Method::Post, &json, &url);
        client.post(url).headers(header).json(&json).send().await?;
    }

    Ok(())
}
