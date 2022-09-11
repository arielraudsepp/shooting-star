use crate::configuration::{AppData, Environment};
use crate::controllers::{LoginForm, SignupForm};
use actix_web::rt::task::JoinHandle;
use anyhow::Context;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use secrecy::{ExposeSecret, Secret};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub user_id: i32,
    pub email: String,
    pub name: String,
    pub password: Secret<String>,
}

#[tracing::instrument(name = "Create new user", skip(user, config))]
pub async fn create_user(user: SignupForm, config: &AppData) -> Result<(), anyhow::Error> {
    let mut transaction = config.pg_pool.begin().await?;
    let email = user.email;
    let name = user.name;
    let password = user.password;
    let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await?
        .context("Failed to hash password")?;
    let query_statement = r#"
        INSERT INTO users (email, name, password_hash)
        VALUES ($1, $2, $3)"#;
    sqlx::query(query_statement)
        .bind(email)
        .bind(name)
        .bind(password_hash.expose_secret())
        .execute(&mut transaction)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}", e);
            e
        })?;

    if let Environment::Dev = config.env {
        transaction.commit().await?;
    }

    Ok(())
}

#[tracing::instrument(name = "Get stored credentials", skip(email, config))]
async fn get_stored_credentials(
    email: &str,
    config: &AppData,
) -> Result<Option<(i32, Secret<String>)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE email = $1
        "#,
        email,
    )
    .fetch_optional(&config.pg_pool)
    .await
    .context("Failed to performed a query to retrieve stored credentials.")?
    .map(|row| (row.id, Secret::new(row.password_hash)));
    Ok(row)
}

#[tracing::instrument(name = "Get username", skip(config))]
pub async fn get_name(user_id: i32, config: &AppData) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT name
        FROM users
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_one(&config.pg_pool)
    .await
    .context("Failed to performed a query to retrieve username")?;
    Ok(row.name)
}

#[tracing::instrument(name = "Validate credentials", skip(config, login_data))]
pub async fn validate_credentials(
    config: &AppData,
    login_data: LoginForm,
) -> Result<i32, AuthError> {
    let mut user_id = None;
    // fallback expected password to protect against timing attacks
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
         gZiV/M1gPc22ElAH/Jh1Hw$\
         CWOrkoo7oJBQ/"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_credentials(&login_data.email, config).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, login_data.password)
    })
    .await
    .context("Failed to spawn blocking task.")??;

    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username."))
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(name = "Change password", skip(password, config))]
pub async fn change_password(
    user_id: uuid::Uuid,
    password: Secret<String>,
    config: &AppData,
) -> Result<(), anyhow::Error> {
    let mut transaction = config.pg_pool.begin().await?;
    let password_hash = spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await?
        .context("Failed to hash password")?;
    let query_statement = r#"UPDATE users SET password_hash = $1 WHERE user_id = $2"#;
    sqlx::query(query_statement)
        .bind(password_hash.expose_secret())
        .bind(user_id)
        .execute(&mut transaction)
        .await
        .context("Failed to change user's password in the database.")?;
    Ok(())
}

fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(Secret::new(password_hash))
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    actix_web::rt::task::spawn_blocking(move || current_span.in_scope(f))
}
