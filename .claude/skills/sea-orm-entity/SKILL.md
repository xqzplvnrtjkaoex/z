---
name: sea-orm-entity
description: |
  CRITICAL: Use for sea-orm entity/model definition. Triggers on:
  DeriveEntityModel, Entity struct, Model definition, ActiveModel,
  Column enum, Relation, sea_orm entity, composite primary key,
  ActiveModelBehavior, ActiveValue, Set NotSet Unchanged
---

# SeaORM Entity Skill

> **Version:** sea-orm 1.1.19 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/sea-orm

You are an expert at the Rust `sea-orm` crate. Help users by:
- **Writing code**: Generate entity definitions following sea-orm patterns
- **Answering questions**: Explain Entity/Model/ActiveModel concepts, troubleshoot derive issues

## Documentation

Refer to the local files for detailed documentation:
- `./references/entity-structure.md` — Entity definition, column attributes, JSON fields, arrays
- `./references/active-model.md` — ActiveValue states, hooks, conversion patterns

## IMPORTANT: Documentation Completeness Check

**Before answering questions, Claude MUST:**

1. Read the relevant reference file(s) listed above
2. If file read fails or file is empty:
   - Inform user: "Local docs incomplete. Run `/sync-crate-skills sea-orm --force` to update."
   - Still answer based on SKILL.md patterns + built-in knowledge
3. If reference file exists, incorporate its content into the answer

## Key Patterns

### Basic Entity

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### Composite Primary Key

```rust
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: Uuid,
    #[sea_orm(primary_key)]
    pub book_id: i32,
}
```

### ActiveValue States

| State | Meaning | When to use |
|-------|---------|-------------|
| `Set(value)` | Write this value | Insert or update |
| `NotSet` | Exclude from query | Auto-increment PKs |
| `Unchanged(value)` | Loaded but unchanged | From DB, not modified |

### Relation Definition

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
    #[sea_orm(
        belongs_to = "super::cake::Entity",
        from = "Column::CakeId",
        to = "super::cake::Column::Id"
    )]
    Cake,
}
```

## API Reference Table

| Derive/Attribute | Description | Example |
|------------------|-------------|---------|
| `DeriveEntityModel` | Auto-derive Entity, Model, Column, PrimaryKey | `#[derive(DeriveEntityModel)]` |
| `#[sea_orm(table_name)]` | Set table name | `#[sea_orm(table_name = "users")]` |
| `#[sea_orm(primary_key)]` | Mark PK column | On struct field |
| `#[sea_orm(column_name)]` | Custom column name | `#[sea_orm(column_name = "lName")]` |
| `#[sea_orm(column_type)]` | Override column type | `#[sea_orm(column_type = "Text")]` |
| `#[sea_orm(nullable)]` | Mark nullable | On `Option<T>` fields |
| `#[sea_orm(unique)]` | Unique constraint | On struct field |
| `#[sea_orm(indexed)]` | Create index | On struct field |
| `FromJsonQueryResult` | JSON column deserialization | Derive on custom struct |
| `DeriveRelation` | Auto-derive relations | On Relation enum |
| `EnumIter` | Required with DeriveRelation | On Relation enum |

## When Writing Code

1. Always derive `Clone, Debug, PartialEq, Eq` alongside `DeriveEntityModel`
2. `Relation` enum + `ActiveModelBehavior` impl are required even if empty
3. Use `#[sea_orm(primary_key, auto_increment = false)]` for UUID PKs
4. For composite PKs, mark multiple fields with `#[sea_orm(primary_key)]`
5. JSON fields need `FromJsonQueryResult` derive on the custom struct

## When Answering Questions

1. Entity = table, Model = read-only row, ActiveModel = mutable row for writes
2. `ActiveValue::Set` means "include in query", `NotSet` means "skip this field"
3. `model.into()` converts `Model` → `ActiveModel` with all fields `Unchanged`
4. Column names default to snake_case of the struct field name
