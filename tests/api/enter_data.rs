use crate::helpers::spawn_app;
use shooting_star::models::Skill;
use shooting_star::{configuration::get_configuration, run};
use sqlx::{postgres::PgConnection, Connection};

#[actix_rt::test]
async fn enter_skill_returns_a_201() {
    // Calling spawn up first because we need our test database to exist, and spawn app creates it.
    let app = spawn_app().await;

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    let mut connection_pool = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/create", &app.address))
        .json(&serde_json::json!({
            "name": "mindfulness",
            "completed": true,
        }))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, response.status().as_u16());

    let body: Skill = response.json().await.unwrap();
    assert_eq!(body.name, "mindfulness");
    assert_eq!(body.completed, true);
}
