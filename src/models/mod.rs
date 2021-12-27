use async_trait::async_trait;
use uuid::Uuid;

pub mod skills;

pub use skills::*;

use crate::configuration::AppData;

#[async_trait]
pub trait Record {
    async fn save(self, pool: &AppData) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
    async fn find_by_id(pool: &AppData, id: Uuid) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
}
