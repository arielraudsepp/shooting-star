use crate::models::{AuthError, validate_credentials};
use crate::configuration::AppData;
use crate::controllers::LoginForm;
use actix_web::error::InternalError;
use actix_web::http::header::LOCATION;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::{web, ResponseError};
use actix_web_flash_messages::FlashMessage;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

//
pub async fn post(
    data: web::Json<LoginForm>,
    config: web::Data<AppData>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let login_data = data.into_inner();

    match validate_credentials(&config, login_data).await {
        Ok(user_id) => {
              Ok(HttpResponse::Ok()
                .finish())
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

fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
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
