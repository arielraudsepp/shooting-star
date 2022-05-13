use shooting_star::configuration::{get_configuration, AppData};
use shooting_star::controllers::LoginForm;
use shooting_star::models::Credentials;
use shooting_star::run;
use std::net::TcpListener;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use uuid::Uuid;
use sqlx::Connection;
use sqlx::postgres::PgConnection;
use serde::{Deserialize, Serialize};

pub struct TestApp {
    pub address: String,
    pub db_url: String,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let app_data = AppData::init(&configuration).await;
    let hmac_secret = configuration.hmac_secret;
    let redis_uri = configuration.redis_uri;
    let server = run(listener, app_data, hmac_secret, redis_uri).await.unwrap();
    let _ = tokio::spawn(server);

    let test_app = TestApp {
        address: address,
        db_url: configuration.database.connection_string(),
    };

    test_app
}

#[derive(Deserialize, Serialize)]
pub struct TestUser {
    pub username: String,
    pub password: String,
}


impl TestUser {
    pub fn generate() -> Self {
        Self {
            username: Uuid::new_v4().to_string(),
            password: "password".to_string(),
        }
    }
}

pub async fn create_test_user(mut connection: PgConnection) -> TestUser {
    let user = TestUser::generate();
    let salt = SaltString::generate(&mut rand::thread_rng());
    // Match production parameters
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
        .hash_password(user.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let query = "INSERT INTO users (username, password_hash)
            VALUES ($1, $2)";
    sqlx::query(query)
        .bind(&user.username)
        .bind(password_hash)
        .execute(&mut connection)
        .await
        .expect("Failed to store test user.");
    user
}


#[actix_rt::test]
async fn login_user() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
        let configuration = get_configuration().expect("Failed to read configuration.");
    let user_connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    let test_user: TestUser = create_test_user(user_connection).await;

    let create_response = client
        .post(&format!("{}/login", &app.address))
        .json(&test_user)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, create_response.status().as_u16());
}
