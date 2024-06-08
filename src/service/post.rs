use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

use chrono::{SecondsFormat, Utc};
use log::{error, info};
use reqwest::Client;
use rusqlite::{Connection, OptionalExtension};
use serde_json::json;
use ulid::Ulid;
use url::Url;

use super::httpsig::create_header;
use crate::structs::Method;

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

pub async fn create(content: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;
    let url = &config.server.url;

    let html = markdown::to_html(&content);
    let id = Ulid::new();
    let time = Utc::now();
    let time_str = time.to_rfc3339_opts(SecondsFormat::Secs, true);

    let conn = Connection::open(db).inspect_err(|e| error!("Failed to open database: {e}"))?;
    conn.execute(
        "INSERT INTO posts (id, content, date) VALUES (?1, ?2, ?3)",
        (id.to_string(), &html, &time_str),
    )
    .inspect_err(|e| error!("Failed to store post: {e}"))?;

    info!("Successfully stored note: {id}");

    let inboxes = get_inboxes().lock()?;

    let body = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": id.to_string(),
        "type": "Create",
        "actor": format!("{url}actor"),
        "published": time_str,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [format!("{url}followers")],
        "object": {
            "id": format!("{url}post/{id}"),
            "type": "Note",
            "attributedTo": format!("{url}profile"),
            "content": html,
            "url": format!("{url}post/{id}"),
            "published": time_str,
            "to": ["https://www.w3.org/ns/activitystreams#Public"],
            "cc": [format!("{url}followers")]
        }
    });

    let client = Client::new();
    for inbox in inboxes.iter() {
        let url: Url = inbox.parse()?;
        let header = create_header(Method::Post, &body, &url);
        client.post(url).headers(header).json(&body).send().await?;
    }

    Ok(())
}
