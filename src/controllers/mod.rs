use chrono::{DateTime, Utc};
use secrecy::Secret;
use serde::{Deserialize, Serialize};

pub mod credentials_controller;
pub mod diary_entries_controller;
pub mod health_check_controller;
pub mod skills_controller;

#[derive(Deserialize, Serialize, Debug)]
pub struct DiaryForm {
    pub entry_date: DateTime<Utc>,
    pub skill_ids: Vec<i32>,
    pub notes: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    pub email: String,
    pub password: Secret<String>,
}

#[derive(Deserialize, Debug)]
pub struct SignupForm {
    pub email: String,
    pub name: String,
    pub password: Secret<String>,
}
