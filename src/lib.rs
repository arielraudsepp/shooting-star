pub mod configuration;
pub mod controllers;
pub mod models;

use crate::controllers::{health_check, enter_data, get_data};

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};

use sqlx::PgPool;
use std::net::TcpListener;
use chrono::Utc;
use serde::{Deserialize, Serialize};


#[derive(Serialize,Deserialize,Debug,PartialEq)]
pub struct Skill {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub completed: bool,
    pub created_at: sqlx::types::chrono::DateTime<Utc>,
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        let cors = Cors::default().allow_any_header().allow_any_method().allow_any_origin().max_age(3600);
        App::new()
            .wrap(cors)
            .route("/health_check", web::get().to(health_check))
            .route("/enter_data", web::post().to(enter_data))
            .route("/get_data/{id}", web::get().to(get_data))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
