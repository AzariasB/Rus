use ::entity::{redirection, redirection::Entity as Redirection};
use sea_orm::*;
use rand::{distributions::Alphanumeric, Rng};
use crate::Query;


pub struct Mutation;

impl Mutation {
    pub async fn create_redirection(
        db: &DbConn,
        long_url: String,
    ) -> Result<redirection::ActiveModel, DbErr> {
        loop {
            let short: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(6)
                .map(char::from)
                .collect();

            let existing = Query::find_redirection_by_short_url(db, short.to_string()).await?;
            if existing == None {
                return redirection::ActiveModel {
                    long_url: Set(long_url),
                    short_url: Set(short),
                    ..Default::default()
                }
                    .save(db)
                    .await
            }
        }
    }

    pub async fn update_redirection_by_id(
        db: &DbConn,
        id: i32,
        form_data: String,
    ) -> Result<redirection::Model, DbErr> {
        let redirection: redirection::ActiveModel = Redirection::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find redirection.".to_owned()))
            .map(Into::into)?;

        redirection::ActiveModel {
            id: redirection.id,
            long_url: Set(form_data),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn delete_redirection(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let redirection: redirection::ActiveModel = Redirection::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find redirection.".to_owned()))
            .map(Into::into)?;

        redirection.delete(db).await
    }

    pub async fn delete_all_redirections(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Redirection::delete_many().exec(db).await
    }
}
