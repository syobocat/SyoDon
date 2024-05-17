use reqwest::{header::ACCEPT, Client};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::structs::{Activity, Actor, Method};

use super::sign_header::create_header;

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
        .post(actor.inbox)
        .headers(header)
        .json(&json)
        .send()
        .await?;

    Ok(())
}