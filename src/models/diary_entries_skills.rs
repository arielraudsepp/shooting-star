use crate::configuration::{AppData, Environment};
use crate::models::{Skill, DiaryEntry};
use crate::models::diary_entries;
use crate::models::Record;
use crate::controllers::DiaryForm;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};



#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct DiaryEntrySkills {
    pub diary_entry_id: i32,
    pub skills_id: i32,
    pub created_at: sqlx::types::chrono::DateTime<Utc>
}

//get transaction, query last record in Diary Entry for the
//entry id, iterate through vector of skills from Diary form, and add each into the table

// fn add_diary_entry_skills(form: DiaryForm, config: &AppData) {
//     let skill_list = form.skill_names;
//     let skill_ids = Skill::find_by_name(config, &skill_list).iter();
//     for skill_id in skill_ids {
//         save(skill_id, config);
//     }
// }

impl DiaryEntrySkills {
    pub async fn save_diary_entry_skill(config: &AppData, skill: &Skill, diary_entry: &DiaryEntry)
        -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("
    INSERT INTO diary_entries_skills (diary_entry_id, skills_id, created_at)
    VALUES ($1, $2) RETURNING diary_entry_id, skills_id, created_at
    ");
            let diary_entry_skill: DiaryEntrySkills  = sqlx::query_as(&query_statement)
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

#[async_trait]
impl Record for DiaryEntrySkills {
    async fn save(self, config: &AppData) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("
    INSERT INTO diary_entries_skills (diary_entry_id, skills_id, created_at)
    VALUES ($1, $2, $3) RETURNING diary_entry_id, skills_id, created_at
    ");
        let query: DiaryEntrySkills = sqlx::query_as(&query_statement)
            .bind(self.diary_entry_id)
            .bind(self.skills_id)
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
