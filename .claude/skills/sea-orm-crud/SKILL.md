---
name: sea-orm-crud
description: |
  CRITICAL: Use for sea-orm CRUD operations. Triggers on:
  sea-orm insert, select, update, delete, find_by_id, filter,
  order_by, insert_many, update_many, delete_many, on_conflict,
  upsert, exec_without_returning, exec_with_returning, save,
  OnConflict, do_nothing, update_column, paginate
---

# SeaORM CRUD Skill

> **Version:** sea-orm 1.1.19 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/sea-orm

You are an expert at the Rust `sea-orm` crate. Help users by:
- **Writing code**: Generate CRUD operations following sea-orm patterns
- **Answering questions**: Explain query building, upsert patterns, execution variants

## Documentation

Refer to the local files for detailed documentation:
- `./references/select-operations.md` — Find, filter, order, pagination, partial model
- `./references/write-operations.md` — Insert, update, delete, OnConflict upsert

## IMPORTANT: Documentation Completeness Check

**Before answering questions, Claude MUST:**

1. Read the relevant reference file(s) listed above
2. If file read fails or file is empty:
   - Inform user: "Local docs incomplete. Run `/sync-crate-skills sea-orm --force` to update."
   - Still answer based on SKILL.md patterns + built-in knowledge
3. If reference file exists, incorporate its content into the answer

## Key Patterns

### Select

```rust
// By PK
let cake: Option<cake::Model> = Cake::find_by_id(1).one(db).await?;

// With filter + order
let cakes: Vec<cake::Model> = Cake::find()
    .filter(cake::Column::Name.contains("chocolate"))
    .order_by_asc(cake::Column::Name)
    .all(db)
    .await?;
```

### Insert with ON CONFLICT (Upsert)

```rust
use sea_orm::sea_query::OnConflict;

my_table::Entity::insert(active_model)
    .on_conflict(
        OnConflict::columns([my_table::Column::UserId, my_table::Column::BookId])
            .update_columns([my_table::Column::Page, my_table::Column::UpdatedAt])
            .to_owned(),
    )
    .exec_without_returning(db)
    .await?;
```

### Update from Model

```rust
let mut am: fruit::ActiveModel = model.into();
am.name = Set("New Name".to_owned());
let updated: fruit::Model = am.update(db).await?;
```

### Delete

```rust
let res: DeleteResult = Fruit::delete_by_id(38).exec(db).await?;
```

## API Reference Table

| Operation | Method | Returns |
|-----------|--------|---------|
| Find by PK | `Entity::find_by_id(pk).one(db)` | `Option<Model>` |
| Find all | `Entity::find().all(db)` | `Vec<Model>` |
| Find one | `Entity::find().one(db)` | `Option<Model>` |
| Insert one | `am.insert(db)` | `Model` |
| Insert (result) | `Entity::insert(am).exec(db)` | `InsertResult` |
| Insert many | `Entity::insert_many([...]).exec(db)` | `InsertResult` |
| Insert no return | `Entity::insert(am).exec_without_returning(db)` | `u64` |
| Insert returning | `Entity::insert(am).exec_with_returning(db)` | `Model` |
| Update one | `am.update(db)` | `Model` |
| Update many | `Entity::update_many().set(am).filter(...).exec(db)` | `UpdateResult` |
| Save (upsert) | `am.save(db)` | `ActiveModel` |
| Delete one | `model.delete(db)` | `DeleteResult` |
| Delete by PK | `Entity::delete_by_id(pk).exec(db)` | `DeleteResult` |
| Delete many | `Entity::delete_many().filter(...).exec(db)` | `DeleteResult` |

## Deprecated Patterns (Don't Use)

| Deprecated | Correct | Notes |
|------------|---------|-------|
| SELECT + match for upsert | `Entity::insert().on_conflict().exec_without_returning()` | Single atomic query, no race condition |
| `am.save(db)` for known upsert | `Entity::insert().on_conflict()` | `save()` requires PK knowledge; ON CONFLICT is explicit |

## When Writing Code

1. Prefer `exec_without_returning` for upserts where you don't need the result
2. Use `OnConflict::columns([...])` for composite PK conflict targets
3. Always call `.to_owned()` on `OnConflict` builder before passing to `.on_conflict()`
4. `filter()` uses `ColumnTrait` methods: `.eq()`, `.contains()`, `.is_in()`, `.lt()`, `.gt()`
5. `order_by_asc` / `order_by_desc` take a `Column` variant

## When Answering Questions

1. `exec()` returns `InsertResult { last_insert_id }`, not the Model
2. `exec_with_returning()` returns the Model (uses Postgres RETURNING)
3. `exec_without_returning()` returns `u64` (rows affected)
4. `save()` checks PK: `NotSet` -> INSERT, `Set` -> UPDATE
5. `on_conflict` requires `sea_orm::sea_query::OnConflict` import
