use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create the PostgreSQL enum type first (Pitfall 2)
        manager
            .create_type(
                extension::postgres::Type::create()
                    .as_enum(UserRoleEnum::Table)
                    .values([UserRoleEnum::Owner, UserRoleEnum::Admin, UserRoleEnum::User])
                    .to_owned(),
            )
            .await?;

        // 2. Create users table referencing the enum type
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_uuid(Users::Id))
                    .col(string_len(Users::Handle, 15))
                    .col(string_len(Users::Name, 80)) // 20 chars * ~4 bytes max UTF-8
                    .col(
                        ColumnDef::new(Users::Role)
                            .custom(UserRoleEnum::Table)
                            .not_null()
                            .default("user"),
                    )
                    .col(boolean(Users::IsActive).default(true))
                    .col(timestamp_with_time_zone(Users::CreatedAt))
                    .col(timestamp_with_time_zone(Users::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        // 3. Case-insensitive unique index on LOWER(handle) via raw SQL (Pitfall 3)
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE UNIQUE INDEX \"idx_users_handle_lower\" ON \"users\" (LOWER(\"handle\"))",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop table first, then drop type (reverse of up)
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_type(
                extension::postgres::Type::drop()
                    .name(UserRoleEnum::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Handle,
    Name,
    Role,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

// Separate DeriveIden enum for the PostgreSQL enum type
// to avoid name collision with the Users::Table variant (Research Anti-Pattern note)
#[derive(DeriveIden)]
enum UserRoleEnum {
    Table,
    Owner,
    Admin,
    User,
}
