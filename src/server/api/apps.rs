use actix_multipart::form::{text::Text, MultipartForm};
use actix_web::{get, post, HttpResponse, Responder};
use serde_json::json;

#[derive(MultipartForm)]
struct Application {
    client_name: Text<String>,
    redirect_uris: Text<String>,
    scopes: Option<Text<String>>,
    website: Option<Text<String>>,
}

#[post("/api/v1/apps")]
async fn apps(MultipartForm(form): MultipartForm<Application>) -> impl Responder {
    let body = json!({
        "id": "000",
        "name": form.client_name.into_inner(),
        "website": form.website.map(|text| text.into_inner()),
        "redirect_uri": form.redirect_uris.into_inner(),
        "client_id": "000",
        "client_secret": "000"
    });

    HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .json(body)
}

#[get("/api/v1/apps")]
async fn get_apps() -> impl Responder {
    HttpResponse::MethodNotAllowed().finish()
}
