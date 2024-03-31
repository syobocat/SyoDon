use crate::config;
use std::collections::HashMap;

use actix_web::{get, web, HttpResponse, Responder};
use serde_json::json;

#[get("/.well-known/webfinger")]
pub async fn webfinger(
    config: web::Data<config::Config>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let host = &config.server.host;
    let name = &config.user.name;

    let Some(resource) = query.get("resource") else {
        return HttpResponse::BadRequest().finish();
    };

    let acceptable = [
        format!("acct:{name}@{host}"),
        format!("https://{host}/{name}"),
        format!("https://{host}/profile"),
        format!("https://{host}/actor"),
        format!("https://{host}"),
    ];

    if !acceptable.contains(resource) {
        return HttpResponse::NotFound().finish();
    }

    let body = json!({
        "subject": format!("acct:{name}@{host}"),
        "links": [
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": format!("https://{host}/actor")
            },
            {
                "rel": "http://webfinger.net/rel/profile-page",
                "type": "text/plain",
                "href": format!("https://{host}/profile")
            }
        ]
    });
    HttpResponse::Ok()
        .content_type("application/jrd+json")
        .json(body)
}
