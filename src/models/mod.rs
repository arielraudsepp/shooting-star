use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub mod skills;

pub use skills::*;

#[async_trait]
pub trait Record {
    async fn save(self, pool: &PgPool) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
    async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
}
