//!tests/helpers.rs

use shooting_star::configuration::{get_configuration, DatabaseSettings};
use shooting_star::run;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");

    // This is not done yet, we need to change the type of server to accept a transaction or a PgPool
    let connection_pool = configure_database(&configuration.database)
        .await
        .begin()
        .await
        .expect("Unable to start transaction");
    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn delete_database(connection: &mut PgConnection, name: &str) -> Result<(), sqlx::Error> {
    connection
        .execute(&*format!(r#"DROP DATABASE IF EXISTS "{}";"#, name))
        .await?;
    Ok(())
}

pub async fn create_database(connection: &mut PgConnection, name: &str) -> Result<(), sqlx::Error> {
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, name))
        .await?;

    Ok(())
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    delete_database(&mut connection, &config.database_name)
        .await
        .unwrap();
    create_database(&mut connection, &config.database_name)
        .await
        .unwrap();

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
