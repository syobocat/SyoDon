use actix_web::{get, web, HttpResponse, Responder};
use rusqlite::{Connection, OptionalExtension};
use serde_json::json;

#[get("/post/{id}")]
async fn post(path: web::Path<String>) -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let host = &config.server.host;
    let db = &config.server.db;

    let id = path.into_inner();

    let Ok(conn) = Connection::open(db) else {
        return HttpResponse::InternalServerError().finish();
    };
    let Ok(query): Result<(Option<String>, Option<String>), _> = conn.query_row(
        "SELECT content, date FROM posts WHERE id = ?1",
        [&id],
        |row| Ok((row.get(0).optional()?, row.get(1).optional()?)),
    ) else {
        return HttpResponse::InternalServerError().finish();
    };

    let (Some(content), Some(date)) = query else {
        return HttpResponse::NotFound().finish();
    };

    let body = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": format!("https://{host}/{id}"),
        "type": "Note",
        "published": date,
        "attributedTo": format!("https://{host}/actor"),
        "content": content,
        "to": "https://www.w3.org/ns/activitystreams#Public"
    });

    HttpResponse::Ok()
        .content_type("application/activity+json; charset=utf-8")
        .json(body)
}
