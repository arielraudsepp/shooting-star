use crate::configuration::AppData;
use crate::models::{Record, Skill};

use actix_web::web;
use actix_web::HttpResponse;

pub async fn show(
    params: web::Path<(String,)>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let id = &params.0;
    let skill_id: i32 = id.parse().unwrap();

    match Skill::find_by_id(&config, skill_id).await {
        Ok(entry) => Ok(HttpResponse::Ok().json(entry)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn index(config: web::Data<AppData>) -> actix_web::Result<HttpResponse> {
    match Skill::find_all(&config).await {
        Ok(entries) => Ok(HttpResponse::Ok().json(entries)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
