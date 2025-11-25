use sea_orm::{ConnectionTrait, Schema};
use sea_orm_migration::prelude::*;
use sea_orm_migration::MigrationTrait;

use super::entity;

#[derive(Debug, Clone, Copy)]
pub struct TabMigration;

impl MigrationName for TabMigration {
    fn name(&self) -> &str {
        "m20240101_000003_create_tabs_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for TabMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        let schema = Schema::new(backend);

        let mut create_tabs = schema.create_table_from_entity(entity::Entity);
        create_tabs.if_not_exists();

        db.execute(backend.build(&create_tabs))
            .await
            .map_err(|e| DbErr::Custom(format!("failed to create tabs table: {}", e)))?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(entity::Entity).to_owned())
            .await
    }
}
