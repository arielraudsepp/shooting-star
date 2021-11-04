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
struct Skill {
    #[serde(skip_deserializing)]
    id: Option<Uuid>,
    name: String,
    completed: bool,
    #[serde(skip_deserializing)]
    created_at: Option<Utc>,
}
#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(skill, pool),
    fields(
        data_name = %skill.name,
        data_completed = %skill.completed
    )
)]
async fn enter_data(skill: web::Form<Skill>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_entry(&pool, &skill).await {
        Ok(record) => HttpResponse::Ok().body(record),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving data in the database", skip(skill, pool))]
async fn insert_entry(pool: &PgPool, skill: &Skill) -> Result<String, sqlx::Error> {
    let query = sqlx::query!(
        r#"
    INSERT INTO skills_tracker (id, skill_name, completed, created_at)
    VALUES ($1, $2, $3, $4) RETURNING id, skill_name, completed, created_at
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

#[tracing::instrument(name = "Retrieving data from the database", skip(pool))]
async fn get_data(pool: &PgPool) -> Result<HttpResponse, sqlx::Error> {
    sqlx::query!(
        r#"
    SELECT * FROM skills_tracker
"#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(HttpResponse::Ok().finish())
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/enter_data", web::post().to(enter_data))
            //.route("/get_data", web::get().to(get_data))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
