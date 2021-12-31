use crate::configuration::{AppData, Environment};
use crate::models::Record;
use sqlx::{FromRow, ConnectOptions};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, FromRow)]
pub struct Skill {
    pub id: i32,
    pub name: String
}


#[async_trait]
impl Record for Skill {
    #[tracing::instrument(name = "Saving skill in the database", skip(self, config))]
    async fn save(self, config: &AppData) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("
    INSERT INTO skills (id, name)
    VALUES ($1, $2) RETURNING id, name
    ");
        let query: Skill = sqlx::query_as(&query_statement)
            .bind(self.id)
            .bind(self.name)
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

    //#[tracing::instrument(name = "Retrieving skill by id from the database", skip(config, id))]
    async fn find_by_id(config: &AppData, id: i32) -> Result<Self, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        let query_statement = format!("SELECT * from skills WHERE id = $1");
            let skill: Skill  = sqlx::query_as(&query_statement)
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
    #[tracing::instrument(name = "Retrieving skill id by name from the database", skip(config))]
    pub async fn find_by_name(config: &AppData, skill_names: &[String]) -> Result<Vec<Self>, sqlx::Error> {
        let mut transaction = config.pg_pool.begin().await?;
        println!("{:?}", skill_names);
        let list = skill_names.into_iter().fold(String::new(), |str: String, item|{
            if str.is_empty(){
                format!("'{}'", item).to_string()
            } else {
                format!("{}, '{}'", str, item).to_string()
            }
        });
        let query_statement = format!("SELECT * from skills WHERE name IN ({});", list);
            let skills: Vec<Skill>  = sqlx::query_as(&query_statement)
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
