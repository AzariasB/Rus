use derive_more::{Display, Error};
use redis::RedisError;
use sea_orm::DbErr;

#[derive(Debug, Display, Error)]
pub enum RusError {
    #[display(fmt = "Access forbidden")]
    Forbidden,

    #[display(fmt = "Database error")]
    Database(DbErr),

    #[display(fmt = "Redis error")]
    Redis(RedisError),

    #[display(fmt = "Unknown error")]
    Unknown,
}

impl RusError {
    pub fn name(&self) -> String {
        match self {
            Self::Forbidden => "Forbidden".to_string(),
            Self::Unknown => "Unknown".to_string(),
            Self::Database(details) => format!("Database : {0}", details.to_string()),
            Self::Redis(details) => format!("Redis : {0}", details.to_string()),
        }
    }
}

impl From<DbErr> for RusError {
    fn from(err: DbErr) -> Self {
        RusError::Database(err)
    }
}

impl From<RedisError> for RusError {
    fn from(err: RedisError) -> Self {
        RusError::Redis(err)
    }
}
