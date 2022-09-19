use crate::configuration::AppData;
use crate::models::{create_user, get_name, validate_credentials, AuthError};
use crate::models::{LoginForm, SignupForm};
use actix_session::Session;
use actix_web::error::InternalError;
use actix_web::http::header::LOCATION;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::{web, ResponseError};
use validator::validate_email;

// Return an opaque 500 while preserving the error's root cause for logging.
fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

pub async fn login(
    data: web::Json<LoginForm>,
    config: web::Data<AppData>,
    session: Session,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let login_data = data.into_inner();

    match validate_credentials(&config, login_data).await {
        Ok(user_id) => {
            session.renew();
            session
                .insert("user_id", user_id)
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;
            Ok(HttpResponse::Ok().json(user_id))
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            Err(login_redirect(e))
        }
    }
}

pub async fn session_name(
    config: web::Data<AppData>,
    session: Session,
) -> Result<HttpResponse, actix_web::Error> {
    let name = if let Some(user_id) = session.get::<i32>("user_id").map_err(e500)? {
        get_name(user_id, &config).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };
    Ok(HttpResponse::Ok().json(name))
}

pub async fn logout(session: Session) -> Result<HttpResponse, actix_web::Error> {
    if session.get::<i32>("user_id").map_err(e500)?.is_none() {
        Ok(HttpResponse::BadRequest().finish())
    } else {
        session.purge();
        Ok(HttpResponse::Ok().finish())
    }
}

fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();
    InternalError::from_response(e, response)
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            LoginError::AuthError(_) => StatusCode::UNAUTHORIZED,
        }
    }
}

pub async fn signup(
    data: web::Json<SignupForm>,
    config: web::Data<AppData>,
) -> actix_web::Result<HttpResponse> {
    let signup_data = data.into_inner();

    if validate_email(&signup_data.email) {
        match create_user(signup_data, &config).await {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(_) => Ok(HttpResponse::InternalServerError().finish()),
        }
    } else {
        return Ok(HttpResponse::BadRequest().json(&signup_data.email));
    }
}
