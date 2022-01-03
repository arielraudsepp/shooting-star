use shooting_star::configuration::get_configuration;
use sqlx::postgres::PgPoolOptions;
use std::fs;

#[tokio::main]
async fn main() {
    let file_name = "seeds/seed_skills.sql";
    let seed = fs::read_to_string(&file_name).expect("Unable to read file");

    let config = get_configuration().expect("Unable to read settings file");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&config.database.connection_string())
        .await
        .expect("Unable to connect to postgres");

    sqlx::query(&seed)
        .fetch_all(&pool)
        .await
        .expect("Unable to generate seed data");
}
