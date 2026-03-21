# ActiveModel Reference

> sea-orm 1.1.19 | Source: https://docs.rs/sea-orm/1.1.19/sea_orm/entity/active_model/

## ActiveValue States

```rust
use sea_orm::ActiveValue::{Set, NotSet, Unchanged};
```

| State | Meaning | SQL behavior |
|-------|---------|--------------|
| `Set(value)` | Field has a value to write | Included in INSERT/UPDATE SET clause |
| `NotSet` | Field excluded from query | Omitted entirely (auto-increment, defaults) |
| `Unchanged(value)` | Loaded but not modified | Omitted from UPDATE SET clause |

### Usage

```rust
// For INSERT — auto-increment PK
let fruit = fruit::ActiveModel {
    id: NotSet,
    name: Set("Apple".to_owned()),
};

// For UPDATE — only name will be in SET clause
let mut fruit: fruit::ActiveModel = model.into(); // all fields Unchanged
fruit.name = Set("New Name".to_owned()); // only this field becomes Set
```

## Model to ActiveModel Conversion

```rust
// From Model — all fields become Unchanged
let model: fruit::Model = Fruit::find_by_id(1).one(db).await?.unwrap();
let am: fruit::ActiveModel = model.into();

// Using IntoActiveModel trait
use sea_orm::IntoActiveModel;
let am = model.into_active_model();
```

## ActiveModelBehavior Hooks

```rust
#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Called when creating a new ActiveModel via `ActiveModel::new()`
    fn new() -> Self {
        Self {
            uuid: Set(Uuid::new_v4()),
            ..ActiveModelTrait::default()
        }
    }

    /// Called before insert (insert=true) or update (insert=false)
    async fn before_save<C>(self, db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.price.as_ref() <= &0.0 {
            Err(DbErr::Custom("Invalid price".to_owned()))
        } else {
            Ok(self)
        }
    }

    /// Called after successful insert or update
    async fn after_save<C>(model: Model, db: &C, insert: bool) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(model)
    }

    /// Called before delete
    async fn before_delete<C>(self, db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(self)
    }

    /// Called after successful delete
    async fn after_delete<C>(self, db: &C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(self)
    }
}
```

## ActiveValue Helper Methods

| Method | Description |
|--------|-------------|
| `ActiveValue::set(v)` | Create `Set(v)` |
| `av.is_set()` | Returns true if `Set` |
| `av.is_not_set()` | Returns true if `NotSet` |
| `av.is_unchanged()` | Returns true if `Unchanged` |
| `av.as_ref()` | Get `&T` (panics if `NotSet`) |
| `av.unwrap()` | Get `T` (panics if `NotSet`) |
| `av.take()` | Take value, leaving `NotSet` |
| `av.reset()` | Change `Unchanged` to `Set` (force update) |

## Default ActiveModel

```rust
// Empty ActiveModel — all fields NotSet
let am = fruit::ActiveModel {
    ..Default::default()
};

// Partial — only set what you need
let am = fruit::ActiveModel {
    name: Set("Apple".to_owned()),
    ..Default::default()
};
```
