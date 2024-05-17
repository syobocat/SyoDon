use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Copy)]
pub enum Method {
    Get,
    Post,
}

#[derive(Serialize, Deserialize)]
pub enum ActivityType {
    Follow,
}

#[derive(Serialize, Deserialize)]
pub struct Activity {
    pub id: Url,
    pub r#type: ActivityType,
    pub actor: Url,
    pub object: serde_json::Value,
}

// We probably need more fields
#[derive(Serialize, Deserialize)]
pub struct Actor {
    pub followers: Url,
    pub following: Url,
    pub id: Url,
    pub inbox: Url,
    pub outbox: Url,
    pub name: String,
    pub summary: String,
    pub url: Url,
}
