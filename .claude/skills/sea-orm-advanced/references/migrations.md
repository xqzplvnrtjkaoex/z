# Migrations Reference

> sea-orm 1.1.19 | Source: https://www.sea-ql.org/SeaORM/docs/1.1.x/migration/writing-migration

## Migration Structure

```rust
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(pk_auto(Post::Id))
                    .col(string(Post::Title))
                    .col(string(Post::Text))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-post_title")
                    .table(Post::Table)
                    .col(Post::Title)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_index(
            Index::drop().name("idx-post_title").to_owned()
        ).await?;
        manager.drop_table(
            Table::drop().table(Post::Table).to_owned()
        ).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    Text,
}
```

## DeriveIden

Generates sea-query identifiers from enum variants. Converts PascalCase to snake_case:

```rust
#[derive(DeriveIden)]
enum Post {
    Table,      // "post"
    Id,         // "id"
    Title,      // "title"
    CreatedAt,  // "created_at"
}
```

## Schema Helper Shorthand

```rust
use sea_orm_migration::schema::*;

// Primary keys
pk_auto(Col::Id)            // integer, PK, auto-increment
pk_uuid(Col::Id)            // uuid, PK

// String types
string(Col::Title)          // varchar, NOT NULL
string_null(Col::Title)     // varchar, nullable
string_len(Col::Title, 255) // varchar(255), NOT NULL

// Numeric types
integer(Col::Count)         // integer, NOT NULL
integer_null(Col::Count)    // integer, nullable
big_integer(Col::Total)     // bigint, NOT NULL
small_integer(Col::Rank)    // smallint, NOT NULL
float(Col::Score)           // float, NOT NULL
double(Col::Precise)        // double, NOT NULL

// Other types
boolean(Col::Active)        // boolean, NOT NULL
boolean_null(Col::Flag)     // boolean, nullable
text(Col::Body)             // text, NOT NULL
text_null(Col::Notes)       // text, nullable
timestamp(Col::CreatedAt)   // timestamp, NOT NULL
timestamp_null(Col::Deleted) // timestamp, nullable
uuid(Col::Uid)              // uuid, NOT NULL
uuid_null(Col::Ref)         // uuid, nullable
json(Col::Data)             // json, NOT NULL
json_null(Col::Meta)        // json, nullable
```

## Verbose ColumnDef (when helpers aren't enough)

```rust
ColumnDef::new(Post::Id)
    .integer()
    .not_null()
    .auto_increment()
    .primary_key()

ColumnDef::new(Post::Title)
    .string()
    .not_null()
    .default("untitled")

ColumnDef::new(Post::Score)
    .decimal_len(10, 2)
    .not_null()
```

## Create Index

```rust
manager.create_index(
    Index::create()
        .if_not_exists()
        .name("idx-post_title")
        .table(Post::Table)
        .col(Post::Title)
        .to_owned(),
).await?;

// Unique index
manager.create_index(
    Index::create()
        .if_not_exists()
        .name("idx-user_email")
        .table(User::Table)
        .col(User::Email)
        .unique()
        .to_owned(),
).await?;

// Composite index
manager.create_index(
    Index::create()
        .if_not_exists()
        .name("idx-history_user_book")
        .table(History::Table)
        .col(History::UserId)
        .col(History::BookId)
        .to_owned(),
).await?;
```

## Create Foreign Key

```rust
manager.create_foreign_key(
    ForeignKey::create()
        .name("fk-fruit_cake_id")
        .from(Fruit::Table, Fruit::CakeId)
        .to(Cake::Table, Cake::Id)
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::Cascade)
        .to_owned(),
).await?;
```

## Alter Table

```rust
// Add column
manager.alter_table(
    Table::alter()
        .table(Post::Table)
        .add_column(string_null(Post::Category))
        .to_owned(),
).await?;

// Drop column
manager.alter_table(
    Table::alter()
        .table(Post::Table)
        .drop_column(Post::Category)
        .to_owned(),
).await?;

// Rename column
manager.alter_table(
    Table::alter()
        .table(Post::Table)
        .rename_column(Post::Title, Post::Name)
        .to_owned(),
).await?;
```

## SchemaManager Methods

| Method | Description |
|--------|-------------|
| `create_table(stmt)` | CREATE TABLE |
| `drop_table(stmt)` | DROP TABLE |
| `alter_table(stmt)` | ALTER TABLE |
| `create_index(stmt)` | CREATE INDEX |
| `drop_index(stmt)` | DROP INDEX |
| `create_foreign_key(stmt)` | ADD FOREIGN KEY |
| `drop_foreign_key(stmt)` | DROP FOREIGN KEY |
| `create_type(stmt)` | CREATE TYPE (Postgres enum) |
| `drop_type(stmt)` | DROP TYPE |
| `has_table(name)` | Check if table exists |
| `has_column(table, col)` | Check if column exists |

## Migrator Setup

```rust
// lib.rs
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_post_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_post_table::Migration),
        ]
    }
}
```

## Running Migrations

```rust
use sea_orm_migration::MigratorTrait;

// Apply all pending migrations
Migrator::up(db, None).await?;

// Rollback last migration
Migrator::down(db, Some(1)).await?;

// Check migration status
Migrator::status(db).await?;
```
