use actix_web::{App, HttpServer};
use log::info;

mod actor;
mod api;
mod inbox;
mod nodeinfo;
mod post;
mod webfinger;

#[actix_web::get("/hello/{name}")]
async fn greet(name: actix_web::web::Path<String>) -> impl actix_web::Responder {
    format!("Hello {name}!")
}

pub async fn serve() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("syodon=info"));
    let config = crate::CONFIG.get().unwrap();
    info!("This is {}!", config.server.url.host_str().unwrap());

    HttpServer::new(move || {
        App::new()
            .service(greet)
            .service(webfinger::webfinger)
            .service(actor::actor)
            .service(nodeinfo::nodeinfo)
            .service(nodeinfo::nodeinfo_20)
            .service(nodeinfo::nodeinfo_21)
            .service(post::post)
            .service(inbox::inbox)
            .service(api::apps::apps)
            .service(api::apps::get_apps)
            .service(api::oauth::authorize)
            .service(api::oauth::token)
    })
    .bind((config.server.bind, config.server.port))?
    .run()
    .await
}
