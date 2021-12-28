use shooting_star::{configuration::{get_configuration, AppData}, run};
use std::net::TcpListener;
use tracing_subscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let app_data = AppData::init(&configuration).await;

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, app_data)?.await?;
    Ok(())
}
