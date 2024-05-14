use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use log::{error, info};
use rusqlite::Connection;
use serde::Deserialize;

#[derive(Deserialize)]
struct Status {
    status: String,
    _media_ids: Option<String>,
    _poll: Option<String>,
    in_reply_to_id: Option<String>,
    sensitive: Option<bool>,
    spoiler_text: Option<String>,
    visibility: Option<String>,
}

#[post("/api/v1/statuses")]
async fn create_status(
    authorization: web::Header<Authorization<Bearer>>,
    web::Form(form): web::Form<Status>,
) -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;

    let token = authorization.0.as_ref().token();

    let Ok(conn) = Connection::open(db).inspect_err(|e| error!("Failed to open database: {e}"))
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(mut stmt) = conn.prepare("SELECT * FROM token WHERE token = ?1") else {
        return HttpResponse::InternalServerError().finish();
    };
    let Ok(is_valid_token) = stmt.exists([token]) else {
        return HttpResponse::InternalServerError().finish();
    };
    if !is_valid_token {
        return HttpResponse::Unauthorized().finish();
    }

    HttpResponse::Ok().finish()
}
