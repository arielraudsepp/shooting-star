use crate::configuration::{AppData, Environment};
use crate::models::Record;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct DiaryEntry {
    pub id: i32,
    pub user_id: i32,
    pub entry_date: sqlx::types::chrono::NaiveDate,
    pub created_at: sqlx::types::chrono::DateTime<Utc>,
    pub updated_at: sqlx::types::chrono::DateTime<Utc>,
    pub notes: String,
}

#[derive(Deserialize, Debug)]
pub struct DateRangeRequest {
    pub start: Option<sqlx::types::chrono::NaiveDate>,
    pub end: Option<sqlx::types::chrono::NaiveDate>,
}

#[tracing::instrument(
    name = "Saving diary entry from form and user_id in the database",
    skip(config)
)]
pub async fn save_from_form(
    entry_date: &DateTime<Utc>,
    notes: &str,
    config: &AppData,
    user_id: &i32,
) -> Result<DiaryEntry, sqlx::Error> {
    let current_time = Utc::now();
    let mut transaction = config.pg_pool.begin().await?;
    let query_statement = r#"
    INSERT INTO diary_entries (user_id, entry_date, created_at, updated_at, notes)
    VALUES ($1, $2, $3, $4, $5) RETURNING id, user_id, entry_date, created_at, updated_at, notes
    "#;
    let query: DiaryEntry = sqlx::query_as(query_statement)
        .bind(user_id)
        .bind(entry_date)
        .bind(current_time)
        .bind(current_time)
        .bind(notes)
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

#[tracing::instrument(
    name = "Updating diary entry by id and user_id in the database",
    skip(config)
)]
pub async fn update_diary_entry(
    id: &i32,
    notes: &str,
    config: &AppData,
    user_id: &i32,
) -> Result<DiaryEntry, sqlx::Error> {
    let mut transaction = config.pg_pool.begin().await?;
    let query_statement = r#"
    UPDATE diary_entries
    SET updated_at = $1, notes = $2
    WHERE id = $3 AND user_id = $4
    RETURNING id, user_id, entry_date, created_at, updated_at, notes
    "#;
    let query: DiaryEntry = sqlx::query_as(query_statement)
        .bind(Utc::now())
        .bind(notes)
        .bind(id)
        .bind(user_id)
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

#[async_trait]
impl Record for DiaryEntry {
    #[tracing::instrument(name = "Saving diary entry in the database", skip(config))]
    async fn save(self, config: &AppData) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"
    INSERT INTO diary_entries (id, user_id, entry_date, created_at, updated_at, notes)
    VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, user_id, entry_date, created_at, updated_at, notes
    "#;
        let query: DiaryEntry = sqlx::query_as(query_statement)
            .bind(self.id)
            .bind(self.user_id)
            .bind(self.entry_date)
            .bind(self.created_at)
            .bind(self.updated_at)
            .bind(&self.notes)
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

    #[tracing::instrument(name = "Retrieving diary entry by id from the database", skip(config))]
    async fn find_by_id(config: &AppData, id: i32) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"SELECT id, user_id, entry_date, created_at, updated_at, notes FROM diary_entries WHERE id = $1"#;
        let diary_entry = sqlx::query_as(query_statement)
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

impl DiaryEntry {
    #[tracing::instrument(
        name = "Retrieving diary entry by date and user_id from the database",
        skip(config)
    )]
    pub async fn find_by_date(
        config: &AppData,
        date: sqlx::types::chrono::NaiveDate,
        user_id: &i32,
    ) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"SELECT id, user_id, entry_date, created_at, updated_at, notes FROM diary_entries WHERE entry_date = $1 AND user_id = $2"#;
        let diary_entry: DiaryEntry = sqlx::query_as(query_statement)
            .bind(date)
            .bind(user_id)
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

impl DiaryEntry {
    #[tracing::instrument(name = "Retrieving diary entries from database", skip(config))]
    pub async fn find_by_date_range(
        config: &AppData,
        date_range: DateRangeRequest,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;

        let query_statement: String;
        if date_range.start.is_none() || date_range.end.is_none() {
            query_statement =
                r#"SELECT id, user_id, entry_date, created_at, updated_at, notes FROM diary_entries"#
                    .to_string();
        } else {
            query_statement = format!(
                "SELECT id, user_id, entry_date, created_at, updated_at, notes FROM diary_entries WHERE entry_date BETWEEN '{}' AND '{}';",
                date_range.start.unwrap(),
                date_range.end.unwrap()
            );
        }
        let diary_entries: Vec<DiaryEntry> = sqlx::query_as(&query_statement)
            .fetch_all(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(diary_entries)
    }
}

impl DiaryEntry {
    #[tracing::instrument(
        name = "Retrieving diary entries by date range and user from database",
        skip(config)
    )]
    pub async fn find_by_date_range_user(
        config: &AppData,
        date_range: DateRangeRequest,
        user_id: &i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;

        let query_statement: String;
        if date_range.start.is_none() || date_range.end.is_none() {
            query_statement = format!(
                "SELECT id, user_id, entry_date, created_at, updated_at, notes FROM diary_entries WHERE user_id = {}",
                user_id
            );
        } else {
            query_statement = format!(
                "SELECT id, user_id, entry_date, created_at, updated_at, notes FROM diary_entries WHERE entry_date BETWEEN '{}' AND '{}' AND user_id = {};",
                date_range.start.unwrap(),
                date_range.end.unwrap(),
                user_id
            );
        }
        let diary_entries: Vec<DiaryEntry> = sqlx::query_as(&query_statement)
            .fetch_all(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(diary_entries)
    }
}
