use crate::models::{DiaryEntry, Credentials};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct DiaryEntryUser {
    pub diary_entry_id: i32,
    pub user_id: i32,
    pub created_at: sqlx::types::chrono::DateTime<Utc>,
}


impl DiaryEntryUser {
    #[tracing::instrument(name = "Saving diary_entry_user in the database", skip(config))]
    pub async fn save_diary_entry_user(
        config: &AppData,
        diary_entry: &DiaryEntry,
        user: &Credentials,
    ) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"
    INSERT INTO diary_entries_users (diary_entry_id, user_id, created_at)
    VALUES ($1, $2, $3) RETURNING diary_entry_id, user_if, created_at
    "#;
        let diary_entry_user: DiaryEntryUser = sqlx::query_as(query_statement)
            .bind(diary_entry.id)
            .bind(user.user_id)
            .bind(diary_entry.created_at)
            .fetch_one(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(diary_entry_user)
    }
}


impl DiaryEntryUser {
    #[tracing::instrument(name = "Deleting diary_entry_user by diary entry id in the database", skip(config))]
    pub async fn delete(
        config: &AppData,
        diary_entry: &DiaryEntry,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"
    DELETE FROM diary_entries_users
    WHERE diary_entry_id = $1
    RETURNING *;
    "#;
        let diary_entry_users: Vec<DiaryEntryUser> = sqlx::query_as(query_statement)
            .bind(diary_entry.id)
            .fetch_all(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(diary_entry_users)
    }
}
