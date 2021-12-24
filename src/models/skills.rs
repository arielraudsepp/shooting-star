use crate::controllers::SkillForm;
use crate::Skill;

use uuid::Uuid;
use chrono::Utc;
use sqlx::PgPool;

#[tracing::instrument(name = "Saving data in the database", skip(skill, pool))]
pub async fn insert_entry(pool: &PgPool, skill: &SkillForm) -> Result<Skill, sqlx::Error> {
    let query = sqlx::query_as!(Skill,
        r#"
    INSERT INTO skills (id, name, completed, created_at)
    VALUES ($1, $2, $3, $4) RETURNING id, name, completed, created_at
"#,
        Uuid::new_v4(),
        skill.name,
        skill.completed,
        Utc::now()
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
pub async fn get_entry(pool: &PgPool, id: Uuid) -> Result<Skill, sqlx::Error> {
    let skill = sqlx::query_as!(Skill, "SELECT * from skills WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(skill)
}
