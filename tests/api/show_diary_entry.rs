use crate::helpers::spawn_app;
use shooting_star::models::DiaryEntry;
use shooting_star::configuration::get_configuration;
use sqlx::Executor;
use sqlx::{postgres::PgConnection, Connection};
use shooting_star::controllers::DiaryForm;
use chrono::{DateTime, Utc, NaiveDate};

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
async fn show_diary_entry_by_date() {
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

    let create_response = client
        .post(&format!("{}/diary_entries", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(201, create_response.status().as_u16());

    let response_body: DiaryEntry = create_response.json().await.unwrap();
    let entry_date = response_body.entry_date.format("%Y-%m-%d").to_string();
    assert_eq!(&entry_date, "2022-02-07");

    let show_response = client
        .get(&format!("{}/diary_entries/{}", &app.address, "2022-02-07"))
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, show_response.status().as_u16());
    }
