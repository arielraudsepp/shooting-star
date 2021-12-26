use crate::controllers::SkillForm;
use crate::models::Record;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
    #[tracing::instrument(name = "Saving data in the database", skip(self, pool))]
    async fn save(self, pool: &PgPool) -> Result<Self, sqlx::Error> {
        let query = sqlx::query_as!(
            Skill,
            r#"
    INSERT INTO skills (id, name, completed, created_at)
    VALUES ($1, $2, $3, $4) RETURNING id, name, completed, created_at
    "#,
            self.id,
            self.name,
            self.completed,
            self.created_at,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}", e);
            e
        })?;
        Ok(query)
    }
    #[tracing::instrument(name = "Retrieving data from the database", skip(pool, id))]
    async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Self, sqlx::Error> {
        let skill = sqlx::query_as!(Skill, "SELECT * from skills WHERE id = $1", id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;
        Ok(skill)
    }
}
