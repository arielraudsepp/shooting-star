use async_trait::async_trait;

pub mod skills;
pub mod diary_entries_skills;
pub mod diary_entries;

pub use skills::*;
pub use diary_entries_skills::*;
pub use diary_entries::*;

use crate::configuration::AppData;

#[async_trait]
pub trait Record {
    async fn save(self, pool: &AppData) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
    async fn find_by_id(pool: &AppData, id: i32) -> Result<Self, sqlx::Error>
    where
        Self: Sized;
}

#[async_trait]
pub trait Form<T> {
    async fn save_from_form(&self, pool: &AppData) -> Result<T, sqlx::Error>
    where
        T: Sized;
}
