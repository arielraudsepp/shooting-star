use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use secrecy::Secret;

pub mod diary_entries_controller;
pub mod health_check_controller;
pub mod skills_controller;
pub mod credentials_controller;

#[derive(Deserialize, Serialize, Debug)]
pub struct DiaryForm {
    pub entry_date: DateTime<Utc>,
    pub skill_ids: Vec<i32>,
    pub notes: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    pub username: String,
    pub password: Secret<String>,
}
