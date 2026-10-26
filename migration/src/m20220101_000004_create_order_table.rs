use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000003_create_shopping_cart_table::ShoppingCart;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Order::Table)
                    .if_not_exists()
                    .col(pk_auto(Order::Id))
                    .col(string(Order::Name))
                    .col(string(Order::Status))
                    .col(integer(Order::ShoppingCartFK))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Order::Table, Order::ShoppingCartFK)
                            .to(ShoppingCart::Table, ShoppingCart::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Order::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
pub enum Order {
    Table,
    Id,
    Name,
    Status,
    ShoppingCartFK,
}
