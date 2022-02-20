use crate::configuration::{AppData, Environment};
use crate::models::Record;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub category: String,
}

#[async_trait]
impl Record for Skill {
    #[tracing::instrument(name = "Saving skill in the database", skip(config))]
    async fn save(self, config: &AppData) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"
    INSERT INTO skills (id, name, category)
    VALUES ($1, $2) RETURNING id, name, category
    "#;
        let query: Skill = sqlx::query_as(query_statement)
            .bind(self.id)
            .bind(self.name)
            .bind(self.category)
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

    #[tracing::instrument(name = "Retrieving skill by id from the database", skip(config))]
    async fn find_by_id(config: &AppData, id: i32) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"SELECT * from skills WHERE id = $1"#;
        let skill: Skill = sqlx::query_as(query_statement)
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

impl Skill {
    #[tracing::instrument(name = "Retrieving skills by ids from the database", skip(config))]
    pub async fn find_by_ids(
        config: &AppData,
        skill_ids: &[i32],
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let list = skill_ids
            .iter()
            .fold(String::new(), |str: String, item| -> String {
                if str.is_empty() {
                    format!("'{}'", item)
                } else {
                    format!("{}, '{}'", str, item)
                }
            });
        let query_statement = format!("SELECT * from skills WHERE id IN ({});", list);
        let skills: Vec<Skill> = sqlx::query_as(&query_statement)
            .fetch_all(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(skills)
    }
}

impl Skill {
    #[tracing::instrument(name = "Retrieving all skills from the database", skip(config))]
    pub async fn find_all(config: &AppData) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = r#"
    SELECT * from skills
    "#;
        let skills: Vec<Skill> = sqlx::query_as(query_statement)
            .fetch_all(&mut transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to execute query: {:?}", e);
                e
            })?;

        if let Environment::Dev = config.env {
            transaction.commit().await?;
        }

        Ok(skills)
    }
}
