pub mod configuration;
pub mod controllers;
pub mod models;

use controllers::{diary_entries_controller, health_check_controller, skills_controller, credentials_controller};

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::cookie::Key;
use actix_web::{web, App, HttpServer};
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use secrecy::{ExposeSecret, Secret};
use configuration::AppData;
use std::net::TcpListener;


pub async fn run(listener: TcpListener, app_config: AppData, hmac_secret: Secret<String>, redis_uri: Secret<String>) -> Result<Server, anyhow::Error> {
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let app_data: Data<AppData> = Data::new(app_config);
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .supports_credentials()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .route(
                "/health_check",
                web::get().to(health_check_controller::health_check),
            )
            .route(
                "/diary_entries",
                web::post().to(diary_entries_controller::create),
            )
            .route(
                "/diary_entries/{date}",
                web::get().to(diary_entries_controller::show),
            )
            .route(
                "/diary_entries",
                web::get().to(diary_entries_controller::index),
            )
            .route(
                "/diary_entries/{date}/skills",
                web::get().to(diary_entries_controller::show_skills),
            )
            .route(
                "/diary_entries/{id}",
                web::patch().to(diary_entries_controller::update),
            )
            .route("/skills", web::get().to(skills_controller::index))
            .route("/skills/{id}", web::get().to(skills_controller::show))
            .route("/login", web::post().to(credentials_controller::login))
            .route("/signup", web::post().to(credentials_controller::signup))
            .route("/session_username", web::get().to(credentials_controller::session_username))
            .route("/logout", web::get().to(credentials_controller::logout))
            .app_data(app_data.clone())
            .app_data(Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
