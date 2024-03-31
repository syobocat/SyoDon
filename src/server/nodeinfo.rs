use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/.well-known/nodeinfo")]
pub async fn nodeinfo() -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let host = &config.server.host;

    let body = json!({
        "links": [
            {
                "href": format!("https://{host}/nodeinfo/2.0.json"),
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0"
            },
            {
                "href": format!("https://{host}/nodeinfo/2.1.json"),
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1"
            }
        ]
    });
    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .json(body)
}

#[get("/nodeinfo/2.0.json")]
pub async fn nodeinfo_20() -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let name = &config.server.name;
    let desc = &config.server.desc;

    let body = json!({
        "version": "2.0",
        "software": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
        },
        "protocols": ["activitypub"],
        "services": {
            "inbound": [],
            "outbound": []
        },
        "openRegistrations": false,
        "usage": {
            "users": {}
        },
        "metadata": {
            "nodeName": name,
            "nodeDescription": desc
        }
    });
    HttpResponse::Ok()
        .content_type("application/json; profile=http://nodeinfo.diaspora.software/ns/schema/2.0#; charset=utf-8")
        .json(body)
}

#[get("/nodeinfo/2.1.json")]
pub async fn nodeinfo_21() -> impl Responder {
    let config = crate::CONFIG.get().unwrap();
    let name = &config.server.name;
    let desc = &config.server.desc;

    let body = json!({
        "version": "2.1",
        "software": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
            "repository": env!("CARGO_PKG_REPOSITORY")
        },
        "protocols": ["activitypub"],
        "services": {
            "inbound": [],
            "outbound": []
        },
        "openRegistrations": false,
        "usage": {
            "users": {}
        },
        "metadata": {
            "nodeName": name,
            "nodeDescription": desc
        }
    });
    HttpResponse::Ok()
        .content_type("application/json; profile=http://nodeinfo.diaspora.software/ns/schema/2.1#; charset=utf-8")
        .json(body)
}
