use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
       manager
            .create_table(
                Table::create()
                    .table(Redirection::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Redirection::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Redirection::LongUrl).string().not_null())
                    .col(ColumnDef::new(Redirection::ShortUrl).string().not_null().unique_key())
                    .col(ColumnDef::new(Redirection::CreationDate).date_time().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(Redirection::ExpirationDate).date_time().null())
                    .col(ColumnDef::new(Redirection::LastAccessDate).date_time().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(Redirection::IpAddress).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Redirection::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Redirection {
    Table,
    Id,
    LongUrl,
    ShortUrl,
    CreationDate,
    ExpirationDate,
    LastAccessDate,
    IpAddress
}
