use sea_orm_migration::MigratorTrait;

mod m20240101_create_schema;
mod m20240201_add_batch_invoicing;
mod m20240301_add_profession;
mod m20240401_add_specialization_and_quantity_option;
mod m20240501_add_ts_submissions;
mod m20240601_add_email;
mod m20240701_add_invoice_amount_options;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        vec![
            Box::new(m20240101_create_schema::Migration),
            Box::new(m20240201_add_batch_invoicing::Migration),
            Box::new(m20240301_add_profession::Migration),
            Box::new(m20240401_add_specialization_and_quantity_option::Migration),
            Box::new(m20240501_add_ts_submissions::Migration),
            Box::new(m20240601_add_email::Migration),
            Box::new(m20240701_add_invoice_amount_options::Migration),
        ]
    }
}
