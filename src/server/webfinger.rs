use std::collections::HashMap;

use actix_web::{get, web, HttpResponse, Responder};
use log::debug;
use serde_json::json;

#[get("/.well-known/webfinger")]
async fn webfinger(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let url = &config.server.url;
    let host = url.host_str().unwrap();
    let name = &config.user.name;

    let Some(resource) = query.get("resource") else {
        return HttpResponse::BadRequest().finish();
    };

    let acceptable = [
        format!("acct:{name}@{host}"),
        format!("{url}{name}"),
        format!("{url}profile"),
        format!("{url}actor"),
        format!("{url}"),
    ];

    debug!("webfinger requested for {resource}");

    if !acceptable.contains(resource) {
        return HttpResponse::NotFound().finish();
    }

    let body = json!({
        "subject": format!("acct:{name}@{host}"),
        "links": [
            {
                "rel": "self",
                "type": "application/activity+json",
                "href": format!("{url}actor")
            },
            {
                "rel": "http://webfinger.net/rel/profile-page",
                "type": "text/plain",
                "href": format!("{url}profile")
            }
        ]
    });
    HttpResponse::Ok()
        .content_type("application/jrd+json; charset=utf-8")
        .json(body)
}
