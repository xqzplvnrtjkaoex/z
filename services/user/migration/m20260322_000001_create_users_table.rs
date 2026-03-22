use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create the PostgreSQL enum type first (Pitfall 2)
        // UserRoleType::Table generates "user_role_type" from DeriveIden, so we
        // override the name using a raw SQL statement to ensure the type is named
        // "user_role" as expected by the sea-orm entity's #[sea_orm(enum_name = "user_role")].
        manager
            .get_connection()
            .execute_unprepared("CREATE TYPE \"user_role\" AS ENUM ('owner', 'admin', 'user')")
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
                            .custom(Alias::new("user_role"))
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
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS \"user_role\"")
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
