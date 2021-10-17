//! tests/health_check.rs


use std::net::TcpListener;
use shooting_star::run;
use shooting_star::configuration::{get_configuration, DatabaseSettings};
use sqlx::PgPool;
use uuid::Uuid;

#[actix_rt::test]
async fn health_check_works() {
    spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap.port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database)
        .await
        .expect("Failed to connect to Postgres");
    let server = run(listener, connection_pool)
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool
    }
}
