# Select Operations Reference

> sea-orm 1.1.19 | Source: https://www.sea-ql.org/SeaORM/docs/1.1.x/basic-crud/select

## Find by Primary Key

```rust
let cake: Option<cake::Model> = Cake::find_by_id(1).one(db).await?;

// Composite PK
let filling: Option<cake_filling::Model> =
    CakeFilling::find_by_id((cake_id, filling_id)).one(db).await?;
```

## Find with Filter and Order

```rust
let chocolate: Vec<cake::Model> = Cake::find()
    .filter(cake::Column::Name.contains("chocolate"))
    .order_by_asc(cake::Column::Name)
    .all(db)
    .await?;
```

## Find One

```rust
let cake: Option<cake::Model> = Cake::find()
    .filter(cake::Column::Name.eq("Cheese Cake"))
    .one(db)
    .await?;
```

## ColumnTrait Filter Methods

| Method | SQL | Example |
|--------|-----|---------|
| `.eq(v)` | `= v` | `Column::Id.eq(1)` |
| `.ne(v)` | `!= v` | `Column::Id.ne(1)` |
| `.gt(v)` | `> v` | `Column::Id.gt(5)` |
| `.gte(v)` | `>= v` | `Column::Id.gte(5)` |
| `.lt(v)` | `< v` | `Column::Id.lt(5)` |
| `.lte(v)` | `<= v` | `Column::Id.lte(5)` |
| `.is_in(vec)` | `IN (...)` | `Column::Id.is_in(vec![1,2,3])` |
| `.is_not_in(vec)` | `NOT IN (...)` | `Column::Id.is_not_in(vec![1,2])` |
| `.is_null()` | `IS NULL` | `Column::Name.is_null()` |
| `.is_not_null()` | `IS NOT NULL` | `Column::Name.is_not_null()` |
| `.contains(s)` | `LIKE '%s%'` | `Column::Name.contains("cake")` |
| `.starts_with(s)` | `LIKE 's%'` | `Column::Name.starts_with("Ch")` |
| `.ends_with(s)` | `LIKE '%s'` | `Column::Name.ends_with("ke")` |
| `.between(a, b)` | `BETWEEN a AND b` | `Column::Id.between(1, 10)` |
| `.like(pat)` | `LIKE pat` | `Column::Name.like("Ch%")` |

## Combining Filters

```rust
use sea_orm::Condition;

// AND (default when chaining .filter())
Cake::find()
    .filter(cake::Column::Name.contains("chocolate"))
    .filter(cake::Column::Id.gt(5))
    .all(db)
    .await?;

// OR
Cake::find()
    .filter(
        Condition::any()
            .add(cake::Column::Name.contains("chocolate"))
            .add(cake::Column::Name.contains("vanilla"))
    )
    .all(db)
    .await?;
```

## Pagination

```rust
use sea_orm::PaginatorTrait;

let cake_pages = Cake::find()
    .order_by_asc(cake::Column::Name)
    .paginate(db, 50); // 50 per page

// Fetch page by page
while let Some(cakes) = cake_pages.fetch_and_next().await? {
    // cakes: Vec<cake::Model>
}

// Or fetch a specific page
let page_2: Vec<cake::Model> = Cake::find()
    .order_by_asc(cake::Column::Name)
    .paginate(db, 50)
    .fetch_page(1) // 0-indexed
    .await?;

// Count total items
let count: u64 = Cake::find().count(db).await?;
```

## Manual Offset/Limit (without Paginator)

```rust
use sea_orm::QuerySelect;

let cakes: Vec<cake::Model> = Cake::find()
    .order_by_asc(cake::Column::Id)
    .offset(10)
    .limit(5)
    .all(db)
    .await?;
```

## Partial Model (Column Selection)

```rust
use sea_orm::FromQueryResult;

#[derive(FromQueryResult)]
struct PartialCake {
    name: String,
}

let partial: Vec<PartialCake> = Cake::find()
    .select_only()
    .column(cake::Column::Name)
    .into_model::<PartialCake>()
    .all(db)
    .await?;
```

## Into JSON

```rust
let cake: Option<serde_json::Value> = Cake::find_by_id(1)
    .into_json()
    .one(db)
    .await?;

let cakes: Vec<serde_json::Value> = Cake::find()
    .filter(cake::Column::Name.contains("chocolate"))
    .order_by_asc(cake::Column::Name)
    .into_json()
    .all(db)
    .await?;
```

## Order By

```rust
// Single column
Cake::find().order_by_asc(cake::Column::Name).all(db).await?;
Cake::find().order_by_desc(cake::Column::Id).all(db).await?;

// Multiple columns
Cake::find()
    .order_by_asc(cake::Column::Name)
    .order_by_desc(cake::Column::Id)
    .all(db)
    .await?;

// Random order (via sea_query)
use sea_orm::{QueryOrder, sea_query::{Func, SimpleExpr}};
Cake::find()
    .order_by(SimpleExpr::FunctionCall(Func::random()), Order::Desc)
    .all(db)
    .await?;
```
