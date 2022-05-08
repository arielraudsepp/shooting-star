use crate::helpers::{spawn_app, create_test_user};
use shooting_star::models::DiaryEntry;
use shooting_star::controllers::DiaryForm;
use shooting_star::configuration::get_configuration;
use chrono::{DateTime, Utc, NaiveDate};
use sqlx::Executor;
use sqlx::{postgres::PgConnection, Connection};

async fn create_test_data(connection: PgConnection) {

    let query = r#"INSERT INTO skills (name, category)
       VALUES
       ('observe', 'mindfulness'),
       ('describe', 'mindfulness'),
       ('activities', 'distress_tolerance'),
       ('contributing', 'distress_tolerance'),
       ('sleep', 'emotion_regulation'),
       ('eating', 'emotion_regulation')"#;
    let mut pg_connection = connection;
    pg_connection
        .execute(query)
        .await
        .expect("Unable to add skills to database");

}


#[actix_rt::test]
async fn create_diary_entry_returns_a_201_for_valid_form_data() {
    let app = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration.");
    let data_connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    create_test_data(data_connection).await;
    let user_connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    create_test_user(user_connection).await;

    let client = reqwest::Client::new();

    let naive_date = NaiveDate::parse_from_str("2022-02-07", "%Y-%m-%d").unwrap();
    let datetime_utc = DateTime::<Utc>::from_utc(naive_date.and_hms(0,0,0), Utc);
    let ids: Vec<i32> = vec![1, 3, 5];
    let body = DiaryForm
    {
    entry_date: datetime_utc.into(),
    skill_ids: ids.into(),
    };


    let response = client
        .post(&format!("{}/diary_entries", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, response.status().as_u16());

    let body: DiaryEntry = response.json().await.unwrap();
    let entry_date = body.entry_date.format("%Y-%m-%d").to_string();
    assert_eq!(&entry_date, "2022-02-07");
}

#[actix_rt::test]
async fn create_diary_entry_adds_diary_entry_skills() {
    let app = spawn_app().await;

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    create_test_data(connection).await;
    let client = reqwest::Client::new();
    let naive_date = NaiveDate::parse_from_str("2022-02-07", "%Y-%m-%d").unwrap();
    let datetime_utc = DateTime::<Utc>::from_utc(naive_date.and_hms(0,0,0), Utc);
    let ids: Vec<i32> = vec![1, 3, 5];
    let body = DiaryForm
    {
    entry_date: datetime_utc.into(),
    skill_ids: ids.into(),
    };


    let response = client
        .post(&format!("{}/diary_entries", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, response.status().as_u16());

    let body: DiaryEntry = response.json().await.unwrap();
    let entry_date = body.entry_date.format("%Y-%m-%d").to_string();
    assert_eq!(&entry_date, "2022-02-07");

    let diary_entries_skills = client
        .get(&format!("{}/diary_entries/{}/skills", &app.address, &entry_date))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, diary_entries_skills.status().as_u16());
}
