use crate::configuration::AppData;
use crate::models::{Record, Skill};

use actix_web::web;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct SkillForm {
    pub name: String,
    pub completed: bool,
}

#[allow(clippy::async_yields_async)]
/// Adds a new skill from an http form data
pub async fn create(
    form: web::Json<SkillForm>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let skill = Skill::init_from_form(form.into_inner());

    match skill.save(&config).await {
        Ok(record) => Ok(HttpResponse::Created().json(record)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn show(
    params: web::Path<(String,)>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let id = &params.0;
    let uuid_id = match Uuid::parse_str(id) {
        Ok(uuid_id) => uuid_id,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };
    match Skill::find_by_id(&config, uuid_id).await {
        Ok(entry) => Ok(HttpResponse::Ok().json(entry)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
