use sea_orm_migration::MigratorTrait;

mod m20240101_create_schema;
mod m20240201_add_batch_invoicing;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(m20240101_create_schema::Migration),
            Box::new(m20240201_add_batch_invoicing::Migration),
        ]
    }
}
