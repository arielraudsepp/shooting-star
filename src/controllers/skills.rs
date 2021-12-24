

use crate::models::{insert_entry, get_entry};

use actix_web::{HttpResponse};
use uuid::Uuid;
use actix_web::web;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize)]
pub struct SkillForm {
    pub name: String,
    pub completed: bool
}

#[allow(clippy::async_yields_async)]
/// Adds a new skill from an http form data
///
pub async fn enter_data(skill: web::Json<SkillForm>, pool: web::Data<PgPool>) -> actix_web::Result<HttpResponse> {
    match insert_entry(&pool, &skill).await {
        Ok(record) => Ok(HttpResponse::Created().json(record)),
         Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub async fn get_data(params: web::Path<(String,)>, pool: web::Data<PgPool>) -> actix_web::Result<HttpResponse> {
    let id = &params.0;
    let uuid_id = match Uuid::parse_str(id) {
        Ok(uuid_id) => uuid_id,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };
    match get_entry(&pool, uuid_id).await {
        Ok(entry) => Ok(HttpResponse::Ok().json(entry)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish())
    }
}
