use actix_http::body::BoxBody;
use actix_http::StatusCode;
use actix_web::error::ResponseError;
use actix_web::http::header::ContentType;
use actix_web::HttpResponse;

use migration::DbErr;
use rus_core::derive_more::{Display, Error};
use rus_core::errors::RusError;
use rus_core::redis::RedisError;

#[derive(Debug, Display, Error)]
pub enum ApiError {
    #[display(fmt = "Interal error")]
    Core(RusError),
    // NotFound,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ApiError::Core(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // ApiError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

impl From<DbErr> for ApiError {
    fn from(err: DbErr) -> Self {
        ApiError::Core(RusError::from(err))
    }
}

impl From<RedisError> for ApiError {
    fn from(err: RedisError) -> Self {
        ApiError::Core(RusError::from(err))
    }
}
