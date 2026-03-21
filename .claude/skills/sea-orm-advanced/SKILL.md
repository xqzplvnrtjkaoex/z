---
name: sea-orm-advanced
description: |
  CRITICAL: Use for sea-orm transactions, migrations, setup. Triggers on:
  sea-orm transaction, TransactionTrait, begin commit rollback,
  sea-orm migration, MigrationTrait, SchemaManager, create_table,
  sea_orm_migration, DeriveIden, DatabaseConnection, feature flags,
  common imports, sea-orm setup, connection pool
---

# SeaORM Advanced Skill

> **Version:** sea-orm 1.1.19 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/sea-orm

You are an expert at the Rust `sea-orm` crate. Help users by:
- **Writing code**: Generate transactions, migrations, and configuration
- **Answering questions**: Explain connection setup, feature flags, migration patterns

## Documentation

Refer to the local files for detailed documentation:
- `./references/transactions.md` — Closure-based, begin/commit, nested
- `./references/migrations.md` — MigrationTrait, schema helpers, indexes, foreign keys

## IMPORTANT: Documentation Completeness Check

**Before answering questions, Claude MUST:**

1. Read the relevant reference file(s) listed above
2. If file read fails or file is empty:
   - Inform user: "Local docs incomplete. Run `/sync-crate-skills sea-orm --force` to update."
   - Still answer based on SKILL.md patterns + built-in knowledge
3. If reference file exists, incorporate its content into the answer

## Key Patterns

### Transaction (Closure)

```rust
use sea_orm::TransactionTrait;

db.transaction::<_, (), DbErr>(|txn| {
    Box::pin(async move {
        entity_a::ActiveModel { ... }.save(txn).await?;
        entity_b::ActiveModel { ... }.save(txn).await?;
        Ok(())
    })
}).await?;
```

### Transaction (Begin/Commit)

```rust
let txn = db.begin().await?;
entity::ActiveModel { ... }.insert(&txn).await?;
txn.commit().await?;
// Auto-rollback if txn dropped without commit
```

### Migration

```rust
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Post::Table)
                .if_not_exists()
                .col(pk_auto(Post::Id))
                .col(string(Post::Title))
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Post::Table).to_owned()).await
    }
}
```

## Common Imports

```rust
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait,
    DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use sea_orm::sea_query::OnConflict;  // for upsert
use sea_orm::sea_query::Expr;        // for raw expressions
use sea_orm::PaginatorTrait;          // for pagination
use sea_orm::FromQueryResult;         // for partial model
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `sqlx-postgres` | PostgreSQL backend |
| `sqlx-mysql` | MySQL backend |
| `sqlx-sqlite` | SQLite backend |
| `runtime-tokio-rustls` | Tokio + rustls TLS |
| `macros` | Derive macros |
| `with-json` | JSON column support |
| `with-chrono` | Chrono DateTime support |
| `with-uuid` | UUID type support |
| `mock` | Mock database for testing |

## Schema Helper Shorthand

```rust
use sea_orm_migration::schema::*;

pk_auto(Col::Id)         // integer PK auto-increment
pk_uuid(Col::Id)         // uuid PK
string(Col::Title)       // string NOT NULL
string_null(Col::Title)  // string nullable
integer(Col::Count)      // integer NOT NULL
boolean(Col::Active)     // boolean NOT NULL
timestamp(Col::CreatedAt) // timestamp NOT NULL
```

## When Writing Code

1. Prefer closure-based transactions for simple multi-table writes
2. Use begin/commit when lifetime constraints make closures awkward
3. Transactions auto-rollback on drop (no explicit rollback needed)
4. Migration `down()` must reverse `up()` exactly
5. Always use `.if_not_exists()` on `create_table` and `.to_owned()` on builders

## When Answering Questions

1. `TransactionTrait` is on `DatabaseConnection` and `DatabaseTransaction`
2. Closure transactions need `Box::pin(async move { ... })` syntax
3. `SchemaManager` methods: `create_table`, `drop_table`, `create_index`, `drop_index`
4. `DeriveIden` generates identifiers for table/column names in migrations
5. Schema helpers (`pk_auto`, `string`, etc.) require `sea_orm_migration::schema::*`
