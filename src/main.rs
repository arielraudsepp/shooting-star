use config::Environment;
use shooting_star::{configuration::{get_configuration, AppData}, run};
use sqlx::postgres::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let app_data = AppData::init(&configuration).await;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, app_data)?.await?;
    Ok(())
}
