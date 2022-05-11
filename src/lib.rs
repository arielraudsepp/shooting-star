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
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::{ExposeSecret, Secret};
use configuration::AppData;
use std::net::TcpListener;


pub async fn run(listener: TcpListener, app_config: AppData, hmac_secret: Secret<String>, redis_uri: Secret<String>) -> Result<Server, anyhow::Error> {
    let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let app_data: Data<AppData> = Data::new(app_config);
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(message_framework.clone())
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
            .route("/login", web::post().to(credentials_controller::post))
            .app_data(app_data.clone())
            .app_data(Data::new(HmacSecret(hmac_secret.clone())))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
