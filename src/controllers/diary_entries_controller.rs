use crate::configuration::AppData;
use crate::models::{DiaryEntry, DiaryEntrySkills, Form, Record, Skill};
use crate::controllers::DiaryForm;

use actix_web::web;
use actix_web::HttpResponse;

//Creates a new diary entry from an HTTP form data
pub async fn create(
    form: web::Json<DiaryForm>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let diary_form = form.into_inner();
    let diary_entry = match diary_form.save_from_form(&config).await {
        Ok(entry) => entry,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };
    let skills_id_list = diary_form.skill_ids;
    let skill_records = Skill::find_by_ids(&config, &skills_id_list);
    let skills = match skill_records.await {
        Ok(skills) => skills,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };
    for skill in skills {
        let diary_entry_skills =
            DiaryEntrySkills::save_diary_entry_skill(&config, &skill, &diary_entry);
        if diary_entry_skills.await.is_err() {
            return Ok(HttpResponse::InternalServerError().finish());
        }
    }
    Ok(HttpResponse::Created().json(&diary_entry))
}

// Retrieves diary entry by id
pub async fn show(
    params: web::Path<(String,)>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let id = &params.0;
    let skill_entry_id: i32 = match id.parse() {
        Ok(skill_entry_id) => skill_entry_id,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };
    match DiaryEntry::find_by_id(&config, skill_entry_id).await {
        Ok(entry) => Ok(HttpResponse::Ok().json(entry)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}
