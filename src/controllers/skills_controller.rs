use crate::configuration::AppData;
use crate::models::{DiaryEntry, Form, Record, Skill, DiaryEntrySkills};

use actix_web::web;
use actix_web::HttpResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct DiaryForm {
    pub entry_date: DateTime<Utc>,
    pub skill_names: Vec<String>,
}



#[allow(clippy::async_yields_async)]
//Adds a new diary entry from an http form data
pub async fn create(
    form: web::Json<DiaryForm>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let diary_form = form.into_inner();
    let diary_entry =  match diary_form
        .save_from_form(&config)
        .await {
            Ok(entry) => entry,
            Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
        };
    let skills_list = diary_form.skill_names;
    let skill_records = Skill::find_by_name(&config, &skills_list);
    let skills = match skill_records.await {
        Ok(skills) => skills,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };
    for skill in skills {
        let diary_entry_skills = DiaryEntrySkills::save_diary_entry_skill(&config, &skill, &diary_entry);
        if diary_entry_skills.await.is_err(){
            return Ok(HttpResponse::InternalServerError().finish());
        }
    };
    Ok(HttpResponse::Created().json(&diary_entry))
}

// pub async fn show(
//     params: web::Path<(String,)>,
//     config: web::Data<AppData>,
// ) -> actix_web::Result<HttpResponse> {
//     let id = &params.0;
//     let uuid_id = match Uuid::parse_str(id) {
//         Ok(uuid_id) => uuid_id,
//         Err(_) => return Ok(HttpResponse::BadRequest().finish()),
//     };
//     match Skill::find_by_id(&config, uuid_id).await {
//         Ok(entry) => Ok(HttpResponse::Ok().json(entry)),
//         Err(_) => Ok(HttpResponse::InternalServerError().finish()),
//     }
// }
