use crate::configuration::{AppData, Environment};
use secrecy::{ExposeSecret, Secret};


#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}


#[tracing::instrument(name = "Retrieving stored credentials", skip(username, config))]
async fn get_stored_credentials(config: &AppData, username: &str) -> Result<Option<(i32, Secret<String>)>, anyhow::Error> {
    let mut transaction = config.pg_pool.begin().await?;
    let query_statement = r#"SELECT id, password_hash from users WHERE username = $1"#;
    let result = sqlx::query_as(query_statement)
        .bind(username)
        .fetch_optional(&mut transaction)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?
        .map(|result| (result.id, Secret::new(result.password_hash)));

    if let Environment::Dev = config.env {
        transaction.commit().await?;
    }

    Ok(result)
}


#[tracing::instrument(name = "Validate credentials", skip(credentials, config))]
async fn validate_credentials(config: &AppData, credentials: Crdentials) -> Result<i32, AuthError> {
    let mut transaction = config.pg_pool.begin().await?;
    let mut user_id = None;
    // fallback expected password to protect against timing attacks
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
         gZiV/M1gPc22ElAH/Jh1Hw$\
         CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_password_hash)) =
        get_stored_crdentials(&credentials.username, pool).await?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
        .await
        .context("Failed to spawn blocking task.")??;

    user_id
        .ok_or_else(|| anyhow::anyhow!("Unknown username."))
        .map_err(AuthError::InvalidCredentials)
}


#[tracing::instrument(name = "Validate credentials", skip(expected_password_hash, password_candidate))]
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
        .map_err(AuthError::InvalidCredentials
}



pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    actix_web::rt::task::spawn_blocking(move || current_span.in_scope(f))
}
