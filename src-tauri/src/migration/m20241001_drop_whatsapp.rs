use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241001_drop_whatsapp"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(UP_SQL).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

/// Removes the tables of `m20240901_add_whatsapp`, a feature that was dropped.
const UP_SQL: &str = "
DROP TABLE IF EXISTS invoice_whatsapps;
DROP TABLE IF EXISTS whatsapp_account;
";

#[cfg(test)]
mod tests {
    use sea_orm::{ConnectionTrait, Database, Statement};
    use sea_orm_migration::MigratorTrait;

    use crate::migration::Migrator;

    #[tokio::test]
    async fn the_whatsapp_tables_do_not_survive_the_migrations() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        let left = db
            .query_all(Statement::from_string(
                db.get_database_backend(),
                "SELECT name FROM sqlite_master WHERE name LIKE '%whatsapp%'",
            ))
            .await
            .unwrap();
        assert!(left.is_empty());
    }
}
