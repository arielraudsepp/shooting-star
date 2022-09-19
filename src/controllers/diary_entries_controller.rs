use crate::configuration::AppData;
use crate::controllers::DiaryForm;
use crate::models::{
    DateRangeRequest, DiaryEntry, DiaryEntryForm, DiaryEntrySkills, Record, Skill,
};

use actix_session::Session;
use actix_web::web;
use actix_web::HttpResponse;

//Creates a new diary entry from an Json

pub async fn create(
    form: web::Json<DiaryForm>,
    config: web::Data<AppData>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let user_id = match session.get::<i32>("user_id") {
        Ok(user_id) => match user_id {
            Some(user_id) => user_id,
            None => return Ok(HttpResponse::InternalServerError().finish()),
        },
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let diary_form = form.into_inner();
    let diary_entry =
        match DiaryEntryForm::save_from_form(user_id, &config, diary_form.entry_form).await {
            Ok(entry) => entry,
            Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
        };
    let skills_list = diary_form.skill_ids;
    if skills_list.is_empty() {
        return Ok(HttpResponse::Created().json(&diary_entry));
    };

    let skill_records = Skill::find_by_ids(&config, &skills_list);
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

// Updates diary entry and diary_entry_skills for a date
pub async fn update(
    form: web::Json<DiaryForm>,
    params: web::Path<(String,)>,
    config: web::Data<AppData>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let diary_form = form.into_inner();
    let user_id = match session.get::<i32>("user_id") {
        Ok(user_id) => match user_id {
            Some(user_id) => user_id,
            None => return Ok(HttpResponse::InternalServerError().finish()),
        },
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let id = &params.0;
    let entry_id: i32 = id.parse().unwrap();
    let entry = DiaryEntry::find_by_id(&config, entry_id);
    let diary_entry = match entry.await {
        Ok(diary_entry) => diary_entry,
        Err(_) => return Ok(HttpResponse::NotFound().finish()),
    };
    let diary_entry_form = diary_form.entry_form;
    let updated_entry = match DiaryEntryForm::update_diary_entry(
        &diary_entry.id,
        user_id,
        diary_entry_form,
        &config,
    )
    .await
    {
        Ok(entry) => entry,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };
    let delete_diary_entry_skills = DiaryEntrySkills::delete(&config, &updated_entry);
    match delete_diary_entry_skills.await {
        Ok(deleted_diary_entry_skills) => deleted_diary_entry_skills,
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };
    let skills_id_list = diary_form.skill_ids;
    if skills_id_list.is_empty() {
        return Ok(HttpResponse::Created().json(&diary_entry));
    };

    let skill_records = Skill::find_by_ids(&config, &skills_id_list);
    let skills = match skill_records.await {
        Ok(skills) => skills,
        Err(_) => return Ok(HttpResponse::NotFound().finish()),
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

// Retrieves diary entry by date
pub async fn show(
    params: web::Path<(String,)>,
    config: web::Data<AppData>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let date = &params.0;
    let diary_entry_date: sqlx::types::chrono::NaiveDate = match date.parse() {
        Ok(entry_date) => entry_date,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };
    let user_id = match session.get::<i32>("user_id") {
        Ok(user_id) => match user_id {
            Some(user_id) => user_id,
            None => return Ok(HttpResponse::InternalServerError().finish()),
        },
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    match DiaryEntry::find_by_date(&config, diary_entry_date, &user_id).await {
        Ok(entry) => Ok(HttpResponse::Ok().json(entry)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

//Retrieves all diary_entry_skills for a particular diary_entry date and user
pub async fn show_skills(
    params: web::Path<(String,)>,
    config: web::Data<AppData>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let date = &params.0;
    let diary_entry_date: sqlx::types::chrono::NaiveDate = match date.parse() {
        Ok(entry_date) => entry_date,
        Err(_) => return Ok(HttpResponse::BadRequest().finish()),
    };
    let user_id = match session.get::<i32>("user_id") {
        Ok(user_id) => match user_id {
            Some(user_id) => user_id,
            None => return Ok(HttpResponse::InternalServerError().finish()),
        },
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    match DiaryEntrySkills::find_diary_entry_skills_by_date(&config, diary_entry_date, &user_id)
        .await
    {
        Ok(records) => Ok(HttpResponse::Ok().json(records)),
        Err(_) => Ok(HttpResponse::NotFound().finish()),
    }
}

//Retrieves diary entries between two dates, or all if no dates
pub async fn index(
    query: web::Query<DateRangeRequest>,
    config: web::Data<AppData>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let date_range: DateRangeRequest = query.into_inner();
    let user_id = match session.get::<i32>("user_id") {
        Ok(user_id) => match user_id {
            Some(user_id) => user_id,
            None => return Ok(HttpResponse::InternalServerError().finish()),
        },
        Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
    };

    let diary_entries =
        match DiaryEntry::find_by_date_range_user(&config, date_range, &user_id).await {
            Ok(entries) => entries,
            Err(_) => return Ok(HttpResponse::BadRequest().finish()),
        };
    let mut updated_diary_entries: Vec<DiaryEntry> = Vec::new();
    for diary_entry in diary_entries {
        let created: i64 = sqlx::types::chrono::DateTime::timestamp(&diary_entry.created_at);
        let updated: i64 = sqlx::types::chrono::DateTime::timestamp(&diary_entry.updated_at);
        if created != updated {
            updated_diary_entries.push(diary_entry);
        }
    }
    return Ok(HttpResponse::Ok().json(updated_diary_entries));
}
