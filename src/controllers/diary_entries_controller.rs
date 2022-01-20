use crate::configuration::AppData;
use crate::controllers::DiaryForm;
use crate::models::{DateRangeRequest, DiaryEntry, DiaryEntrySkills, Form, Record, Skill};

use actix_web::web;
use actix_web::HttpResponse;

//Creates a new diary entry from an Json
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
    if skills_id_list.is_empty() {
        return Ok(HttpResponse::Created().json(&diary_entry));
    };

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

//Retrieves diary entries between two dates, or all if no dates
pub async fn index(
    query: web::Query<DateRangeRequest>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let date_range: DateRangeRequest = query.into_inner();

    match DiaryEntry::find_by_date_range(&config, date_range).await {
        Ok(entries) => Ok(HttpResponse::Ok().json(entries)),
        Err(_) => Ok(HttpResponse::BadRequest().finish()),
    }
}

//Retrieves all diary_entry_skills for a particular diary_entry id
pub async fn index_skills(
    params: web::Path<(String,)>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let id = &params.0;
    let skill_entry_id: i32 = match id.parse() {
        Ok(skill_entry_id) => skill_entry_id,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };
    match DiaryEntrySkills::find_skills_by_diary_id(&config, skill_entry_id).await {
        Ok(records) => Ok(HttpResponse::Ok().json(records)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}
