pub mod configuration;
pub mod controllers;
pub mod models;

use crate::controllers::{create, health_check, show};

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};

use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .route("/health_check", web::get().to(health_check))
            .route("/create", web::post().to(create))
            .route("/show/{id}", web::get().to(show))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
