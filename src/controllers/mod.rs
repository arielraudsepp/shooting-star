use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod diary_entries_controller;
pub mod health_check_controller;
pub mod skills_controller;

#[derive(Deserialize, Serialize, Debug)]
pub struct DiaryForm {
    pub entry_date: DateTime<Utc>,
    pub skill_ids: Vec<i32>,
}
