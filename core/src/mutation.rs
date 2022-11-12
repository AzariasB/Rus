use ::entity::{redirection, redirection::Entity as Redirection};
use sea_orm::*;
use rand::{distributions::Alphanumeric, Rng};
use crate::Query;

const SHORT_URL_LENGTH: usize = 6;


pub struct Mutation;

pub struct CreateMutation {
    long_url: String,
    short_url: String,
    ip_address: String,
}

pub struct UpdateMutation {
    id: i32,
    long_url: String,
}

impl UpdateMutation {
    pub fn new(id: i32, long_url: String) -> UpdateMutation {
        UpdateMutation { id, long_url }
    }
}

fn generate_random_string(str_size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(str_size)
        .map(char::from)
        .collect()
}

impl CreateMutation {
    pub fn new(long_url: String, ip_address: String) -> CreateMutation {
        CreateMutation {
            long_url,
            ip_address,
            short_url: generate_random_string(SHORT_URL_LENGTH),
        }
    }

    fn regenerate_short_url(&mut self) {
        self.short_url = generate_random_string(SHORT_URL_LENGTH);
    }
}

impl Mutation {
    pub async fn create_redirection(
        db: &DbConn,
        mut create: CreateMutation,
    ) -> Result<redirection::ActiveModel, DbErr> {
        loop {
            let existing = Query::find_redirection_by_short_url(db, create.short_url.to_string()).await?;
            if existing == None {
                return redirection::ActiveModel {
                    long_url: Set(create.long_url),
                    short_url: Set(create.short_url),
                    ip_address: Set(create.ip_address),
                    ..Default::default()
                }
                    .save(db)
                    .await;
            }
            create.regenerate_short_url();
        }
    }

    pub async fn update_redirection_by_id(
        db: &DbConn,
        update: UpdateMutation,
    ) -> Result<redirection::Model, DbErr> {
        let redirection: redirection::ActiveModel = Redirection::find_by_id(update.id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find redirection.".to_owned()))
            .map(Into::into)?;

        redirection::ActiveModel {
            id: redirection.id,
            long_url: Set(update.long_url),
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
