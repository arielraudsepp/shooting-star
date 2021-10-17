pub mod configuration;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;
use sqlx::PgPool;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
