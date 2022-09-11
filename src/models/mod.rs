use async_trait::async_trait;

pub mod credentials;
pub mod diary_entries;
pub mod diary_entries_skills;
pub mod skills;

pub use credentials::*;
pub use diary_entries::*;
pub use diary_entries_skills::*;
pub use skills::*;

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
    async fn save_from_form(self, pool: &AppData, user_id: &i32) -> Result<T, sqlx::Error>
    where
        T: Sized;
}
