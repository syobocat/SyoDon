use actix_multipart::form::{text::Text, MultipartForm};
use actix_web::{get, post, HttpResponse, Responder};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use log::{error, info};
use rand::{thread_rng, Rng, RngCore};
use rusqlite::Connection;
use serde_json::json;

#[derive(MultipartForm)]
struct Application {
    client_name: Text<String>,
    redirect_uris: Text<String>,
    _scopes: Option<Text<String>>, // Currently we don't use this
    website: Option<Text<String>>,
}

#[post("/api/v1/apps")]
async fn apps(MultipartForm(form): MultipartForm<Application>) -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;

    let client_name = form.client_name.into_inner();
    let redirect_uris = form.redirect_uris.into_inner();

    let mut rng = thread_rng();
    let mut client_id = [0u8; 32];
    rng.fill_bytes(&mut client_id);
    let client_id_base64 = BASE64_URL_SAFE_NO_PAD.encode(&client_id);

    let mut client_secret = [0u8; 32];
    rng.fill_bytes(&mut client_secret);
    let client_secret_base64 = BASE64_URL_SAFE_NO_PAD.encode(client_secret);

    let Ok(conn) = Connection::open(db).inspect_err(|e| error!("Failed to open database: {e}"))
    else {
        return HttpResponse::InternalServerError().finish();
    };
    if conn.execute(
        "INSERT INTO apps (client_id, client_secret, name, redirect_uri) VALUES (?1, ?2, ?3, ?4)",
        (
            &client_id_base64,
            &client_secret_base64,
            &client_name,
            &redirect_uris,
        ),
    ).inspect_err(|e| error!("Failed to register app: {e}")).is_err() {
        return HttpResponse::InternalServerError().finish();
    };

    let body = json!({
        "id": rng.gen::<u16>().to_string(),
        "name": client_name,
        "website": form.website.map(|text| text.into_inner()),
        "redirect_uri": redirect_uris,
        "client_id": client_id_base64,
        "client_secret": client_secret_base64
    });

    info!("New application has been registered: {client_name}");
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .json(body)
}

#[get("/api/v1/apps")]
async fn get_apps() -> impl Responder {
    HttpResponse::MethodNotAllowed().finish()
}
