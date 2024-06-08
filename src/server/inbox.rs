use actix_web::{http::header, post, web, HttpRequest, HttpResponse, Responder};

use crate::{
    service::httpsig::verify_header,
    structs::{Activity, ActivityType},
};

#[post("/inbox")]
async fn inbox(
    web::Header(content_type): web::Header<header::ContentType>,
    web::Json(activity): web::Json<Activity>,
    request: HttpRequest,
) -> impl Responder {
    let Some(mime) = content_type.suffix() else {
        return HttpResponse::BadRequest().finish();
    };
    if mime != "json" {
        return HttpResponse::BadRequest().finish();
    }
    if verify_header(request).await.is_err() {
        return HttpResponse::BadRequest().finish();
    }

    match activity.r#type {
        ActivityType::Follow => crate::service::user::accept_follow(activity).await.unwrap(), // TODO: Denyable request / remove unwrap
    }

    HttpResponse::Ok().finish()
}
