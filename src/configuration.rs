use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use secrecy::Secret;
use sqlx::{postgres, ConnectOptions, PgPool};


#[derive(serde::Deserialize)]
pub struct EnvSettings {
    pub dev: Settings,
    pub test: Settings,
}

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
    pub redis_uri: Secret<String>,
    pub hmac_secret: Secret<String>,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

// Possible environments for the app
#[derive(Debug, Clone)]
pub enum Environment {
    Dev,
    Test,
}

#[derive(Clone)]
pub struct AppData {
    pub env: Environment,
    pub db_name: String,
    pub pg_pool: sqlx::PgPool,
}

impl AppData {
    pub async fn init(setting: &Settings) -> Self {
        let mut options =
            postgres::PgConnectOptions::from_str(&setting.database.connection_string()).unwrap();
        options.log_statements(tracing::log::LevelFilter::Warn);

        let pg_pool = PgPool::connect_with(options)
            .await
            .expect("Failed to connect to Postgres");

        let env: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "dev".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");
        AppData {
            db_name: setting.database.database_name.clone(),
            env,
            pg_pool,
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "dev" => Ok(Self::Dev),
            "test" => Ok(Self::Test),
            other => Err(format!(
                "{} is not a supported environment. Use either 'dev' or 'test'.",
                other
            )),
        }
    }
}

/// get_configuration returns the apps settings based on the app environment:
/// loads in the configuration file, detects the app environment and
/// returns the correct app settings
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut config = config::Config::default();
    config.merge(config::File::with_name("configuration"))?;
    let settings: EnvSettings = config.try_into()?;

    // Takes in an environment variable (Result type), unwraps it (defaulting to "dev" if unwrap fails),
    // and calls try_into (as we defined try_from on Environment type) on the str to convert
    // it to Environment type.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "dev".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    // matches app environment to correct app settings
    match environment {
        Environment::Dev => Ok(settings.dev),
        Environment::Test => Ok(settings.test),
    }
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}
