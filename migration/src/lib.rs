pub use sea_orm_migration::prelude::*;

mod m20221110_195452_create_redirection_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20221110_195452_create_redirection_table::Migration,
        )]
    }
}
