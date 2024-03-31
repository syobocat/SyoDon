use crate::config;

use actix_web::{get, web, HttpResponse, Responder};
use rsa::pkcs8::EncodePublicKey;
use serde_json::json;

#[get("/actor")]
pub async fn actor(config: web::Data<config::Config>) -> impl Responder {
    let host = &config.server.host;
    let name = &config.user.name;
    let pubkey = crate::PRIVKEY
        .get()
        .unwrap()
        .to_public_key()
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .unwrap();

    let body = json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],
        "id": format!("https://{host}/actor"),
        "type": "Person",
        "preferredUsername": name,
        "inbox": format!("https://{host}/inbox"),
        "publicKey": {
            "id": format!("https://{host}/actor#main-key"),
            "owner": format!("https://{host}/actor"),
            "publicKeyPem": pubkey
        }
    });
    HttpResponse::Ok()
        .content_type("application/activity+json; charset=utf-8")
        .json(body)
}
