use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Product::Table)
                    .if_not_exists()
                    .col(pk_auto(Product::Id))
                    .col(string(Product::Name))
                    .col(integer(Product::Amount))
                    .check(Expr::col(Product::Amount).gte(0))
                    .col(string(Product::Category))
                    .col(float(Product::Price))
                    .to_owned(),
            )
            .await?;

            let mut insert = Query::insert()
            .into_table(Product::Table)
            .columns([
                Product::Name,
                Product::Amount,
                Product::Category,
                Product::Price,
            ]).to_owned();

            insert.values_panic([
                "mock_name".into(),
                4097.into(),
                "Gardening".into(),
                10.97_f32.into()
            ]);

            manager.exec_stmt(insert).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Product::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum Product {
    Table,
    Id,
    Name,
    Amount,
    Category,
    Price,
}
