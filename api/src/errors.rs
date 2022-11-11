use actix_web::{error::ResponseError, http::StatusCode, HttpResponse, http::header::ContentType};
use derive_more::{Display, Error};
use migration::DbErr;

#[derive(Debug, Display, Error)]
pub enum RusError {
    #[display(fmt = "Not found")]
    NotFound,

    #[display(fmt = "Access forbidden")]
    Forbidden,

    #[display(fmt = "Database error")]
    Database(migration::DbErr),

    #[display(fmt = "Unknown error")]
    Unknown,
}

impl RusError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "NotFound".to_string(),
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::Database(details) => format!("Database : {0}", details.to_string())
        }
    }
}

impl From<migration::DbErr> for RusError {
    fn from(err: DbErr) -> Self {
        RusError::Database(err)
    }
}

impl ResponseError for RusError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_err) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        HttpResponse::build(status_code).insert_header(ContentType::html()).body(self.to_string())
    }
}
