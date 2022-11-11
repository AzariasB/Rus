use ::entity::{redirection, redirection::Entity as Redirection};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_redirection_by_id(db: &DbConn, id: i32) -> Result<Option<redirection::Model>, DbErr> {
        Redirection::find_by_id(id).one(db).await
    }

    pub async fn find_redirection_by_short_url(db: &DbConn, short_url: String) -> Result<Option<redirection::Model>,DbErr> {
        Redirection::find().filter(redirection::Column::ShortUrl.eq(short_url)).one(db).await
    }

    pub async fn find_redirections_in_page(
        db: &DbConn,
        page: u64,
        redirections_per_page: u64,
    ) -> Result<(Vec<redirection::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Redirection::find()
            .order_by_asc(redirection::Column::Id)
            .paginate(db, redirections_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
