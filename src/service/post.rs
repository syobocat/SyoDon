use chrono::{SecondsFormat, Utc};
use log::{error, info};
use rusqlite::Connection;
use serde_json::json;
use ulid::Ulid;

pub async fn create(content: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;
    let url = &config.server.url;

    let html = markdown::to_html(&content);
    let id = Ulid::new();
    let time = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let conn = Connection::open(db).inspect_err(|e| error!("Failed to open database: {e}"))?;
    conn.execute(
        "INSERT INTO posts (id, content, date) VALUES (?1, ?2, ?3)",
        (id.to_string(), &html, &time),
    )
    .inspect_err(|e| error!("Failed to store post: {e}"))?;

    info!("Successfully stored note: {id}");

    let body = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": id.to_string(),
        "type": "Create",
        "actor": format!("{url}actor"),
        "published": time,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [format!("{url}followers")],
        "object": {
            "id": format!("{url}post/{id}"),
            "type": "Note",
            "attributedTo": format!("{url}profile"),
            "content": html,
            "url": format!("{url}post/{id}"),
            "published": time,
            "to": ["https://www.w3.org/ns/activitystreams#Public"],
            "cc": [format!("{url}followers")]
        }
    });

    // TODO: post this body

    Ok(())
}
