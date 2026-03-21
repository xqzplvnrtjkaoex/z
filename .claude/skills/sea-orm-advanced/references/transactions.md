# Transactions Reference

> sea-orm 1.1.19 | Source: https://www.sea-ql.org/SeaORM/docs/1.1.x/advanced-query/transaction

## Closure-Based Transaction (Preferred)

```rust
use sea_orm::TransactionTrait;

db.transaction::<_, (), DbErr>(|txn| {
    Box::pin(async move {
        bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
        .save(txn)
        .await?;

        bakery::ActiveModel {
            name: Set("Top Bakery".to_owned()),
            profit_margin: Set(15.0),
            ..Default::default()
        }
        .save(txn)
        .await?;

        Ok(())
    })
}).await?;
```

**Type parameters**: `db.transaction::<_, ReturnType, DbErr>(|txn| { ... })`

## Begin/Commit Pattern

Use when closure lifetimes are awkward (e.g., borrowing across await points):

```rust
let txn = db.begin().await?;

bakery::ActiveModel {
    name: Set("SeaSide Bakery".to_owned()),
    ..Default::default()
}
.save(&txn)
.await?;

bakery::ActiveModel {
    name: Set("Top Bakery".to_owned()),
    ..Default::default()
}
.save(&txn)
.await?;

txn.commit().await?;
```

## Auto-Rollback

If `txn` is dropped without calling `.commit()`, the transaction is automatically
rolled back. No explicit rollback call needed.

```rust
let txn = db.begin().await?;
entity::ActiveModel { ... }.insert(&txn).await?;
// If this function returns early (via ?) before commit,
// txn is dropped and the transaction rolls back automatically.
txn.commit().await?;
```

## Nested Transactions

SeaORM supports nested transactions via database `SAVEPOINT`:

```rust
db.transaction::<_, (), DbErr>(|txn| {
    Box::pin(async move {
        // Outer transaction work
        entity_a::ActiveModel { ... }.save(txn).await?;

        // Nested transaction
        txn.transaction::<_, (), DbErr>(|inner_txn| {
            Box::pin(async move {
                entity_b::ActiveModel { ... }.save(inner_txn).await?;
                Ok(())
            })
        }).await?;

        Ok(())
    })
}).await?;
```

## Transaction with Config

```rust
use sea_orm::{TransactionTrait, IsolationLevel, AccessMode};

db.transaction_with_config::<_, (), DbErr>(
    |txn| {
        Box::pin(async move {
            // ...
            Ok(())
        })
    },
    Some(IsolationLevel::ReadCommitted),
    Some(AccessMode::ReadWrite),
).await?;

// Or with begin
let txn = db.begin_with_config(
    Some(IsolationLevel::Serializable),
    Some(AccessMode::ReadOnly),
).await?;
```

## Using Transaction Reference

All sea-orm operations accept `&DatabaseTransaction` where they accept
`&DatabaseConnection`, because both implement `ConnectionTrait`:

```rust
let txn = db.begin().await?;

// All these work with &txn
let model = Entity::find_by_id(1).one(&txn).await?;
let am = entity::ActiveModel { ... };
am.insert(&txn).await?;
Entity::update_many().set(am).exec(&txn).await?;
Entity::delete_by_id(1).exec(&txn).await?;

txn.commit().await?;
```
