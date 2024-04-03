use actix_web::{get, HttpResponse, Responder};
use rsa::pkcs8::EncodePublicKey;
use serde_json::json;

#[get("/actor")]
async fn actor() -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let url = &config.server.url;
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
        "id": format!("{url}actor"),
        "type": "Person",
        "preferredUsername": name,
        "inbox": format!("{url}inbox"),
        "publicKey": {
            "id": format!("{url}actor#main-key"),
            "owner": format!("{url}actor"),
            "publicKeyPem": pubkey
        }
    });
    HttpResponse::Ok()
        .content_type("application/activity+json; charset=utf-8")
        .json(body)
}
