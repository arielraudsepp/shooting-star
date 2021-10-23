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
struct FormData {
    skill_name: String,
    completed: bool,
}
#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        data_skill_name = %form.skill_name,
        data_completed = %form.completed
    )
)]
async fn enter_data(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    match insert_entry(&pool, &form).await {
        Ok(record) => HttpResponse::Ok().body(record),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving data in the database", skip(form, pool))]
async fn insert_entry(pool: &PgPool, form: &FormData) -> Result<String, sqlx::Error> {
    let query = sqlx::query!(
        r#"
    INSERT INTO skills_tracker (id, skill_name, completed, entered_at)
    VALUES ($1, $2, $3, $4) RETURNING id, skill_name, completed, entered_at
"#,
        Uuid::new_v4(),
        form.skill_name,
        form.completed,
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

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/enter_data", web::post().to(enter_data))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
