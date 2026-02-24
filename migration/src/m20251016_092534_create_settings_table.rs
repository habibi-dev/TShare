use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Setting::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Setting::Id)
                            .big_unsigned()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Setting::MetaKey)
                            .string()
                            .string_len(120)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Setting::MetaValue).text().null())
                    .col(
                        ColumnDef::new(Setting::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Setting::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Setting::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Setting {
    Table,
    Id,
    MetaKey,
    MetaValue,
    CreatedAt,
    UpdatedAt,
}
