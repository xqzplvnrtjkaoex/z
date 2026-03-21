# Write Operations Reference

> sea-orm 1.1.19 | Source: https://www.sea-ql.org/SeaORM/docs/1.1.x/basic-crud/insert

## Insert One

```rust
// Method 1: ActiveModel.insert() — returns Model
let pear = fruit::ActiveModel {
    name: Set("Pear".to_owned()),
    ..Default::default()
};
let pear: fruit::Model = pear.insert(db).await?;

// Method 2: Entity::insert() — returns InsertResult
let res: InsertResult = fruit::Entity::insert(pear).exec(db).await?;
assert_eq!(res.last_insert_id, 28);
```

## Insert Many

```rust
let apple = fruit::ActiveModel {
    name: Set("Apple".to_owned()),
    ..Default::default()
};
let orange = fruit::ActiveModel {
    name: Set("Orange".to_owned()),
    ..Default::default()
};
let res: InsertResult = Fruit::insert_many([apple, orange]).exec(db).await?;

// Empty insert — no-op
let res = Bakery::insert_many(std::iter::empty())
    .on_empty_do_nothing()
    .exec(db)
    .await;
assert!(matches!(res, Ok(TryInsertResult::Empty)));
```

## Insert Execution Variants

| Method | Returns | SQL clause | Use case |
|--------|---------|-----------|----------|
| `.exec(db)` | `InsertResult` | Basic INSERT | Need auto-generated PK |
| `.exec_without_returning(db)` | `u64` | INSERT (no RETURNING) | Upserts, bulk inserts |
| `.exec_with_returning(db)` | `Model` | INSERT RETURNING * | Need full inserted row |
| `.exec_with_returning_keys(db)` | `Vec<PK>` | INSERT RETURNING pk | Bulk insert, PKs only |

## ON CONFLICT — DO NOTHING

```rust
use sea_orm::sea_query::OnConflict;

let orange = cake::ActiveModel {
    id: ActiveValue::set(2),
    name: ActiveValue::set("Orange".to_owned()),
};
cake::Entity::insert(orange)
    .on_conflict(
        OnConflict::column(cake::Column::Name)
            .do_nothing()
            .to_owned()
    )
    .exec_without_returning(db)
    .await?;
// SQL: INSERT INTO "cake" ("id", "name") VALUES (2, 'Orange')
//      ON CONFLICT ("name") DO NOTHING
```

## ON CONFLICT — DO UPDATE (single column)

```rust
cake::Entity::insert(orange)
    .on_conflict(
        OnConflict::column(cake::Column::Name)
            .update_column(cake::Column::Name)
            .to_owned()
    )
    .exec(db)
    .await?;
// SQL: ON CONFLICT ("name") DO UPDATE SET "name" = "excluded"."name"
```

## ON CONFLICT — DO UPDATE (composite PK)

```rust
use sea_orm::sea_query::OnConflict;

let history_book = history_books::ActiveModel {
    user_id: Set(history.user_id),
    book_id: Set(history.book_id),
    page: Set(history.page),
    created_at: Set(history.created_at),
    updated_at: Set(history.updated_at),
};
history_books::Entity::insert(history_book)
    .on_conflict(
        OnConflict::columns([
            history_books::Column::UserId,
            history_books::Column::BookId,
        ])
        .update_columns([
            history_books::Column::Page,
            history_books::Column::UpdatedAt,
        ])
        .to_owned(),
    )
    .exec_without_returning(&db)
    .await?;
```

## OnConflict API

| Method | Description |
|--------|-------------|
| `OnConflict::column(col)` | Single conflict target column |
| `OnConflict::columns([cols])` | Multiple conflict target columns |
| `.do_nothing()` | ON CONFLICT DO NOTHING |
| `.update_column(col)` | Update single column from excluded |
| `.update_columns([cols])` | Update multiple columns from excluded |
| `.to_owned()` | Finalize builder (required) |

---

## Update One (from Model)

```rust
let pear: Option<fruit::Model> = Fruit::find_by_id(28).one(db).await?;
let mut pear: fruit::ActiveModel = pear.unwrap().into();
pear.name = Set("Sweet pear".to_owned());
// SQL: UPDATE "fruit" SET "name" = 'Sweet pear' WHERE "id" = 28
let pear: fruit::Model = pear.update(db).await?;
```

## Update Many

```rust
// Using ActiveModel
let update_result: UpdateResult = Fruit::update_many()
    .set(pear)
    .filter(fruit::Column::Id.is_in(vec![1]))
    .exec(db)
    .await?;

// Using column expression
use sea_orm::sea_query::Expr;
Fruit::update_many()
    .col_expr(fruit::Column::CakeId, Expr::value(1))
    .filter(fruit::Column::Name.contains("Apple"))
    .exec(db)
    .await?;
```

## Save (Auto-Upsert by PK)

`save()` checks if PK is `Set` (UPDATE) or `NotSet` (INSERT):

```rust
let banana = fruit::ActiveModel {
    id: NotSet, // will INSERT
    name: Set("Banana".to_owned()),
    ..Default::default()
};
let mut banana = banana.save(db).await?;

banana.name = Set("Banana Mongo".to_owned());
// id is now Set — will UPDATE
let banana = banana.save(db).await?;
```

## Force Update (reset)

```rust
let mut pear: fruit::ActiveModel = model.into();
pear.name = Set("Sweet pear".to_owned());

// Force-include cake_id even though it didn't change
pear.reset(fruit::Column::CakeId);

// Or force-include all fields
pear.reset_all();
```

---

## Delete One (from Model)

```rust
use sea_orm::ModelTrait;

let orange: fruit::Model = Fruit::find_by_id(30).one(db).await?.unwrap();
let res: DeleteResult = orange.delete(db).await?;
assert_eq!(res.rows_affected, 1);
```

## Delete by Primary Key

```rust
let res: DeleteResult = Fruit::delete_by_id(38).exec(db).await?;
```

## Delete Many

```rust
let res: DeleteResult = fruit::Entity::delete_many()
    .filter(fruit::Column::Name.contains("Orange"))
    .exec(db)
    .await?;
```

## Delete with Returning (Postgres)

```rust
let deleted: Vec<order::Model> = order::Entity::delete_many()
    .filter(order::Column::CustomerId.eq(22))
    .exec_with_returning(db)
    .await?;
```
