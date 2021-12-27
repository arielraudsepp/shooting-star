use crate::helpers::spawn_app;
use shooting_star::models::Skill;
use shooting_star::{configuration::get_configuration, run};
use sqlx::{postgres::PgConnection, Connection};

#[actix_rt::test]
async fn enter_skill_returns_a_201() {
    // Calling spawn up first because we need our test database to exist, and spawn app creates it.
    let app = spawn_app().await;

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // Manually renaming here until we implement a test configuration. This
    // ensure that we can grab data from the same database as our servers is
    // inserting into it. In the old way, the zero2prod way it was creating a
    // database with a random name, one we would never know, so we would not b
    // able to connect to it in our tests, to grab the newly inserted data.
    configuration.database.database_name = "test_skills".to_string();
    let mut connection_pool = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/enter_data", &app.address))
        .json(&serde_json::json!({
            "name": "mindfulness",
            "completed": true,
        }))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, response.status().as_u16());
    let body: Skill = response.json().await.unwrap();
    let skill = sqlx::query_as!(
        Skill,
        "SELECT * FROM skills ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_one(&mut connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })
    .unwrap();

    assert_eq!(body, skill);
}
