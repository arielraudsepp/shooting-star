use shooting_star::{
    configuration::{get_configuration, AppData},
    run,
};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let app_data = AppData::init(&configuration).await;
    let hmac_secret = configuration.hmac_secret;
    let redis_uri = configuration.redis_uri;
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, app_data, hmac_secret, redis_uri)
        .await?
        .await?;
    Ok(())
}
