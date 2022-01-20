pub mod configuration;
pub mod controllers;
pub mod models;

use controllers::{diary_entries_controller, health_check_controller, skills_controller};

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use configuration::AppData;

use std::net::TcpListener;

pub fn run(listener: TcpListener, app_config: AppData) -> Result<Server, std::io::Error> {
    let app_data: Data<AppData> = Data::new(app_config);
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .route(
                "/health_check",
                web::get().to(health_check_controller::health_check),
            )
            .route(
                "/diary_entries",
                web::post().to(diary_entries_controller::create),
            )
            .route(
                "/diary_entries/{id}",
                web::get().to(diary_entries_controller::show),
            )
            .route(
                "/diary_entries",
                web::get().to(diary_entries_controller::index),
            )
            .route(
                "/diary_entries/{id}/skills",
                web::get().to(diary_entries_controller::index_skills),
            )
            .route("/skills", web::get().to(skills_controller::index))
            .route("/skills/{id}", web::get().to(skills_controller::show))
            .app_data(app_data.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
