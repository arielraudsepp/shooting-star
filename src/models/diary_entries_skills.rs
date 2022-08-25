use crate::configuration::{AppData, Environment};
use crate::models::{DiaryEntry, Skill};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct DiaryEntrySkills {
    pub diary_entry_id: i32,
    pub skills_id: i32,
    pub created_at: sqlx::types::chrono::DateTime<Utc>,
}

impl DiaryEntrySkills {
    #[tracing::instrument(name = "Saving diary_entry_skill in the database", skip(config))]
    pub async fn save_diary_entry_skill(
        config: &AppData,
        skill: &Skill,
        diary_entry: &DiaryEntry,
    ) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"
    INSERT INTO diary_entries_skills (diary_entry_id, skills_id, created_at)
    VALUES ($1, $2, $3) RETURNING diary_entry_id, skills_id, created_at
    "#;
        let diary_entry_skill: DiaryEntrySkills = sqlx::query_as(query_statement)
            .bind(diary_entry.id)
            .bind(skill.id)
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

        Ok(diary_entry_skill)
    }
}

impl DiaryEntrySkills {
    #[tracing::instrument(
        name = "Retrieving diary_entry_skills by diary entry id from the database",
        skip(config)
    )]
    pub async fn find_diary_entry_skills_by_diary_id(
        config: &AppData,
        diary_entry_id: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"SELECT * from diary_entries_skills WHERE diary_entry_id = $1"#;
        let diary_entry_skills: Vec<DiaryEntrySkills> = sqlx::query_as(query_statement)
            .bind(diary_entry_id)
            .fetch_all(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(diary_entry_skills)
    }
}

impl DiaryEntrySkills {
    #[tracing::instrument(
        name = "Retrieving diary_entry_skills by diary entry date from the database",
        skip(config)
    )]
    pub async fn find_diary_entry_skills_by_date(
        config: &AppData,
        diary_entry_date: sqlx::types::chrono::NaiveDate,
        user_id: &i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement =
            r#"SELECT diary_entries_skills.diary_entry_id,
            diary_entries_skills.skills_id,
            diary_entries_skills.created_at FROM diary_entries_skills
            JOIN diary_entries
            ON diary_entries_skills.diary_entry_id = diary_entries.id
            WHERE diary_entries.entry_date = $1 AND diary_entries.user_id = $2"#;
        let diary_entry_skills: Vec<DiaryEntrySkills> = sqlx::query_as(&query_statement)
            .bind(diary_entry_date)
            .bind(user_id)
            .fetch_all(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(diary_entry_skills)
    }
}

impl DiaryEntrySkills {
    #[tracing::instrument(name = "Deleting diary_entry_skills by diary entry id in the database", skip(config))]
    pub async fn delete(
        config: &AppData,
        diary_entry: &DiaryEntry,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"
    DELETE FROM diary_entries_skills
    WHERE diary_entry_id = $1
    RETURNING *;
    "#;
        let diary_entry_skills: Vec<DiaryEntrySkills> = sqlx::query_as(query_statement)
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

        Ok(diary_entry_skills)
    }
}
