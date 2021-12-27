use shooting_star::configuration::{get_configuration, AppData};
use shooting_star::run;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

pub struct TestApp {
    pub address: String,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let app_data = AppData::init(&configuration).await;
    let server = run(listener, app_data).unwrap();
    let _ = tokio::spawn(server);
    TestApp {
        address,
    }
}
