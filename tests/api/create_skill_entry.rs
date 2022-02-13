use crate::helpers::spawn_app;
use shooting_star::models::{DiaryEntry};
use shooting_star::{configuration::get_configuration, run};
use sqlx::{postgres::PgConnection, Connection};

#[actix_rt::test]
async fn create_diary_entry_returns_a_201() {
    // Calling spawn up first because we need our test database to exist, and spawn app creates it.
    let app = spawn_app().await;

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/diary_entries", &app.address))
        .json(&serde_json::json!({
            "entry_date": "2022-02-07T00:00:00Z",
            "skill_ids": [1, 3, 5]
        }))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, response.status().as_u16());

    let body: DiaryEntry = response.json().await.unwrap();
    let entry_date = body.entry_date.format("%Y-%m-%d").to_string();
    assert_eq!(&entry_date, "2022-02-07");
}
