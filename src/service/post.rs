use chrono::{SecondsFormat, Utc};
use rusqlite::Connection;
use serde_json::json;
use ulid::Ulid;

pub async fn create(content: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;
    let host = &config.server.host;

    let html = markdown::to_html(&content);
    let id = Ulid::new();
    let time = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let conn = Connection::open(db)?;
    conn.execute(
        "INSERT INTO posts (id, content, date) VALUES (?1, ?2, ?3)",
        (id.to_string(), &html, &time),
    )?;

    let body = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": id.to_string(),
        "type": "Create",
        "actor": format!("https://{host}/actor"),
        "published": time,
        "to": ["https://www.w3.org/ns/activitystreams#Public"],
        "cc": [format!("https://{host}/followers")],
        "object": {
            "id": format!("https://{host}/post/{id}"),
            "type": "Note",
            "attributedTo": format!("https://{host}/profile"),
            "content": html,
            "url": format!("https://{host}/post/{id}"),
            "published": time,
            "to": ["https://www.w3.org/ns/activitystreams#Public"],
            "cc": [format!("https://{host}/followers")]
        }
    });

    // TODO: post this body

    Ok(())
}
