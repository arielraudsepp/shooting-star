use serde::{Deserialize, Serialize};

use crate::models::DiaryEntryForm;

pub mod credentials_controller;
pub mod diary_entries_controller;
pub mod health_check_controller;
pub mod skills_controller;

#[derive(Deserialize, Serialize, Debug)]
pub struct DiaryForm {
    pub entry_form: DiaryEntryForm,
    pub skill_ids: Vec<i32>,
}
