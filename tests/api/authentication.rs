use crate::helpers::spawn_app;
use shooting_star::configuration::get_configuration;
use uuid::Uuid;

#[actix_rt::test]
async fn non_exisiting_user_is_rejected() {
    let app = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration.");
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/diary_entries", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(401, response.status().as_u16());
}
