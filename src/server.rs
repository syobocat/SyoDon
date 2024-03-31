use actix_web::{web::Data, App, HttpServer};
use log::info;

mod actor;
mod webfinger;
use crate::config;

#[actix_web::get("/hello/{name}")]
async fn greet(name: actix_web::web::Path<String>) -> impl actix_web::Responder {
    format!("Hello {name}!")
}

pub async fn serve(config: config::Config) -> std::io::Result<()> {
    info!("This is {}!", config.server.host);

    let bind = config.server.bind;
    let port = config.server.port;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(config.clone()))
            .service(greet)
            .service(webfinger::webfinger)
            .service(actor::actor)
    })
    .bind((bind, port))?
    .run()
    .await
}
