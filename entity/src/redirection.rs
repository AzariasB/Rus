use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "redirection")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub long_url: String,
    #[sea_orm(unique)]
    pub short_url: String,
    pub creation_date: NaiveDateTime,
    #[sea_orm(nullable)]
    pub expiration_date: Option<NaiveDateTime>,
    pub last_access_date: NaiveDateTime,
    pub ip_address: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
