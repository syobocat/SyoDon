use actix_web::{get, post, web, HttpResponse, Responder};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use log::{error, info};
use rand::{thread_rng, RngCore};
use rusqlite::Connection;
use serde::Deserialize;

#[derive(Deserialize)]
struct OAuthRequest {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    _scope: Option<String>,
    _force_login: Option<String>,
    _lang: Option<String>,
}

#[derive(Deserialize)]
struct TokenRequest {
    grant_type: String,
    code: String, // Currently we don't support app-level token
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    _scope: Option<String>,
}

#[get("/oauth/authorize")]
async fn authorize(query: web::Query<OAuthRequest>) -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;

    if query.response_type != "code" {
        return HttpResponse::BadRequest().body("Field code should be \"code\"");
    }

    let client_id = &query.client_id;
    let redirect_uri = &query.redirect_uri;

    let Ok(conn) = Connection::open(db).inspect_err(|e| error!("Failed to open database: {e}"))
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok((redirect_uris, code)): Result<(String, Option<String>), _> = conn.query_row(
        "SELECT redirect_uris, code FROM apps WHERE client_id = ?1",
        [&client_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ) else {
        return HttpResponse::BadRequest().body("Application you requested does not exist");
    };

    if !redirect_uris.contains(redirect_uri) {
        return HttpResponse::BadRequest().body("redirect_uri is not in redirect_uris");
    }

    let Some(code) = code else {
        let body =
            format!("Authentication required. To accept, issue `syodon oauth accept {client_id}`");
        return HttpResponse::Accepted().body(body);
    };

    HttpResponse::TemporaryRedirect()
        .append_header(("LOCATION", format!("{redirect_uri}?code={code}")))
        .finish()
}

#[post("/oauth/token")]
async fn token(query: web::Query<TokenRequest>) -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;

    if query.grant_type != "authorization_code" {
        return HttpResponse::BadRequest()
            .body("Field grant_type should be \"authorization_code\"");
    }

    let Ok(conn) = Connection::open(db).inspect_err(|e| error!("Failed to open database: {e}"))
    else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok((client_secret, redirect_uris, code)): Result<(String, String, Option<String>), _> =
        conn.query_row(
            "SELECT client_secret, redirect_uris, code FROM apps WHERE client_id = ?1",
            [&query.client_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
    else {
        return HttpResponse::BadRequest().body("Application you requested does not exist");
    };

    if query.client_secret != client_secret {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    if query.code != code.unwrap_or_default() {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    if !redirect_uris.contains(&query.redirect_uri) {
        return HttpResponse::BadRequest().body("redirect_uri is not in redirect_uris");
    }

    let mut token = [0u8; 32];
    thread_rng().fill_bytes(&mut token);
    let token_base64 = BASE64_URL_SAFE_NO_PAD.encode(token);

    if conn
        .execute(
            "INSERT INTO token (issuer, token) VALUES (?1, ?2)",
            (&query.client_id, token_base64),
        )
        .inspect_err(|e| error!("Failed to register token: {e}"))
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    info!("New token has been issued by {}", query.client_id);

    HttpResponse::Ok().finish()
}
