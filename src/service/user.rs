use log::{error, info};
use reqwest::{header::ACCEPT, Client};
use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::structs::{Activity, Actor, Method};

use super::httpsig::create_header;

#[derive(Deserialize)]
struct UserJson {
    links: Vec<ListJson>,
}

#[derive(Deserialize)]
struct ListJson {
    rel: String,
    //r#type: String,
    href: String,
}

pub async fn get_actor_url(acct: String) -> Result<Url, Box<dyn std::error::Error>> {
    let server = acct.split('@').last().ok_or("invalid acct")?;
    let webfinger: Url = format!("{server}/.well-known/webfinger").parse()?;
    let client = Client::new();
    let user_json: UserJson = client
        .get(webfinger)
        .query(&[("resource", format!("acct:{acct}"))])
        .header(ACCEPT, "application/jrd+json")
        .send()
        .await?
        .json()
        .await?;

    let url = &user_json
        .links
        .iter()
        .find(|link| link.rel == "self".to_owned())
        .ok_or("invalid user")?
        .href;

    Url::parse(url).map_err(|e| e.into())
}

pub async fn get_actor(url: Url) -> Result<Actor, Box<dyn std::error::Error>> {
    let client = Client::new();
    let actor = client
        .get(url)
        .header(ACCEPT, "application/activity+json")
        .send()
        .await?
        .json()
        .await?;

    Ok(actor)
}

pub async fn get_pubkey(url: Url) -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    let client = Client::new();
    let json: serde_json::Value = client
        .get(url)
        .header(ACCEPT, "application/activity+json")
        .send()
        .await?
        .json()
        .await?;
    let pem = json
        .get("publicKey")
        .ok_or("pubkey not found")?
        .get("publicKeyPem")
        .ok_or("pubkey not found")?
        .as_str()
        .ok_or("pubkey should be in pem format")?;

    RsaPublicKey::from_public_key_pem(pem).map_err(|e| e.into())
}

pub async fn follow(actor: Actor) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let url = &config.server.url;

    let json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        //"id": todo!(),
        "type": "Follow",
        "actor": format!("{url}actor"),
        "object": url
    });

    let client = Client::new();
    let header = create_header(Method::Post, &json, &actor.inbox);
    client
        .post(actor.inbox)
        .headers(header)
        .json(&json)
        .send()
        .await?;

    Ok(())
}

pub async fn follow_by_acct(acct: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = get_actor_url(acct).await?;
    let actor = get_actor(url).await?;
    follow(actor).await
}

pub async fn accept_follow(activity: Activity) -> Result<(), Box<dyn std::error::Error>> {
    let config = crate::CONFIG.get().unwrap();
    let db = &config.server.db;
    let url = &config.server.url;

    let actor = get_actor(activity.actor.clone()).await?;

    let json = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        //"id": todo!(),
        "type": "Accept",
        "actor": format!("{url}actor"),
        "object": activity,
    });

    let client = Client::new();
    let header = create_header(Method::Post, &json, &actor.inbox);
    client
        .post(actor.inbox.clone())
        .headers(header)
        .json(&json)
        .send()
        .await?;

    let conn = Connection::open(db).inspect_err(|e| error!("Failed to open database: {e}"))?;
    conn.execute(
        "INSERT INTO followers (id inbox shared_inbox) VALUES (?1, ?2, ?3)",
        (
            actor.id.to_string(),
            actor.inbox.to_string(),
            actor.shared_inbox.clone().map(|url| url.to_string()),
        ),
    )
    .inspect_err(|e| error!("Failed to add a follower: {e}"))?;

    let mut inboxes = super::deliver::get_inboxes().lock()?;
    inboxes.insert(actor.shared_inbox.unwrap_or(actor.inbox).to_string());

    Ok(())
}
