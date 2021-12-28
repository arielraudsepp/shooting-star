use crate::{configuration::{AppData, Environment}, controllers::SkillForm};
use crate::models::Record;
use sqlx::{FromRow, ConnectOptions};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct Skill {
    pub id: sqlx::types::Uuid,
    pub name: String,
    pub completed: bool,
    pub created_at: sqlx::types::chrono::DateTime<Utc>,
}

impl Skill {
    pub fn init_from_form(form: SkillForm) -> Skill {
        Skill {
            id: Uuid::new_v4(),
            name: form.name,
            completed: form.completed,
            created_at: Utc::now(),
        }
    }
}

#[async_trait]
impl Record for Skill {
    #[tracing::instrument(name = "Saving data in the database", skip(self, config))]
    async fn save(self, config: &AppData) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("
    INSERT INTO skills (id, name, completed, created_at)
    VALUES ($1, $2, $3, $4) RETURNING id, name, completed, created_at
    ");
        let query: Skill = sqlx::query_as(&query_statement)
            .bind(self.id)
            .bind(self.name)
            .bind(self.completed)
            .bind(self.created_at)
            .fetch_one(&mut transaction)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}", e);
            e
        })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(query)
    }

    #[tracing::instrument(name = "Retrieving data from the database", skip(config, id))]
    async fn find_by_id(config: &AppData, id: Uuid) -> Result<Self, sqlx::Error> {
        // Transaction might not be needed here
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("SELECT * from skills WHERE id = $1");
            let skill = sqlx::query_as(&query_statement)
            .bind(id)
            .fetch_one(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(skill)
    }
}
