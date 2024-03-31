use actix_web::{web::Data, App, HttpServer};
use log::info;

mod actor;
mod nodeinfo;
mod webfinger;
use crate::config;

#[actix_web::get("/hello/{name}")]
async fn greet(name: actix_web::web::Path<String>) -> impl actix_web::Responder {
    format!("Hello {name}!")
}

pub async fn serve(config: config::Config) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("syodon=info"));
    info!("This is {}!", config.server.host);

    let bind = config.server.bind;
    let port = config.server.port;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(config.clone()))
            .service(greet)
            .service(webfinger::webfinger)
            .service(actor::actor)
            .service(nodeinfo::nodeinfo)
            .service(nodeinfo::nodeinfo_20)
            .service(nodeinfo::nodeinfo_21)
    })
    .bind((bind, port))?
    .run()
    .await
}
