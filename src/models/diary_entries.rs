use crate::configuration::{AppData, Environment};
use crate::models::{Form, Record};
use crate::controllers::DiaryForm;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct DiaryEntry {
    pub id: i32,
    pub entry_date: sqlx::types::chrono::NaiveDate,
    pub created_at: sqlx::types::chrono::DateTime<Utc>
}

#[async_trait]
impl Form<DiaryEntry> for DiaryForm {
    #[tracing::instrument(name = "Saving diary entry in the database", skip(self, config))]
    async fn save_from_form(&self, config: &AppData) -> Result<DiaryEntry, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("
    INSERT INTO diary_entries (entry_date, created_at)
    VALUES ($1, $2) RETURNING id, entry_date, created_at
    ");
        let query: DiaryEntry = sqlx::query_as(&query_statement)
            .bind(self.entry_date)
            .bind(Utc::now())
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
}

#[async_trait]
impl Record for DiaryEntry {
    #[tracing::instrument(name = "Saving diary entry in the database", skip(self, config))]
    async fn save(self, config: &AppData) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("
    INSERT INTO diary_entries (id, entry_date, created_at)
    VALUES ($1, $2, $3) RETURNING id, entry_date, created_at
    ");
        let query: DiaryEntry = sqlx::query_as(&query_statement)
            .bind(self.id)
            .bind(self.entry_date)
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

    #[tracing::instrument(name = "Retrieving diary entry from the database", skip(config, id))]
    async fn find_by_id(config: &AppData, id: i32) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("SELECT * from diary_entries WHERE id = $1");
            let diary_entry = sqlx::query_as(&query_statement)
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

        Ok(diary_entry)
    }
}