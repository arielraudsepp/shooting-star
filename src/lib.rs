pub mod configuration;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::Utc;
use sqlx::PgPool;
use std::net::TcpListener;
use uuid::Uuid;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct SkillForm {
    name: String,
    completed: bool
}

#[derive(Debug)]
struct Skill {
    id: sqlx::types::Uuid,
    name: String,
    completed: bool,
    created_at: sqlx::types::chrono::DateTime<Utc>,
}
#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new entry",
    skip(skill, pool),
    fields(
        data_name = %skill.name,
        data_completed = %skill.completed
    )
)]
async fn enter_data(skill: web::Form<SkillForm>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_entry(&pool, &skill).await {
        Ok(record) => HttpResponse::Ok().body(record),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving data in the database", skip(skill, pool))]
async fn insert_entry(pool: &PgPool, skill: &SkillForm) -> Result<String, sqlx::Error> {
    let query = sqlx::query!(
        r#"
    INSERT INTO skills (id, name, completed, created_at)
    VALUES ($1, $2, $3, $4) RETURNING id, name, completed, created_at
"#,
        Uuid::new_v4(),
        skill.name,
        skill.completed,
        Utc::now()
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        e
    })?;
    Ok(format!("{:?}", query))
}

async fn get_data(params: web::Path<(String,)>, pool: web::Data<PgPool>) -> HttpResponse {
    let id = &params.0;
    let rid = match Uuid::parse_str(id) {
        Ok(rid) => rid,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match get_entry(&pool, rid).await {
        Ok(entry) => HttpResponse::Ok().body(format!("{:?}", entry)),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(name = "Retrieving data from the database", skip(pool))]
async fn get_entry(pool: &PgPool, id: Uuid) -> Result<Skill, sqlx::Error> {
    let skill = sqlx::query_as!(Skill, "SELECT * from skills WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(skill)
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/enter_data", web::post().to(enter_data))
            .route("/get_data/{id}", web::get().to(get_data))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
