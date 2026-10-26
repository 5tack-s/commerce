use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_user_table::User;
use crate::m20220101_000002_create_product_table::Product;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ShoppingCart::Table)
                    .if_not_exists()
                    .col(pk_auto(ShoppingCart::Id))
                    .col(timestamp_with_time_zone(ShoppingCart::CreatedAt))
                    .col(integer(ShoppingCart::UserFK))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ShoppingCart::Table, ShoppingCart::UserFK)
                            .to(User::Table, User::Id),
                    )
                    .col(integer(ShoppingCart::ProductFK))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ShoppingCart::Table, ShoppingCart::ProductFK)
                            .to(Product::Table, Product::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ShoppingCart::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum ShoppingCart {
    Table,
    Id,
    CreatedAt,
    UserFK,
    ProductFK,
}
