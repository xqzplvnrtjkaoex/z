# Entity Structure Reference

> sea-orm 1.1.19 | Source: https://www.sea-ql.org/SeaORM/docs/1.1.x/generate-entity/entity-structure

## Complete Entity Definition

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
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

## Column Attributes

```rust
// Custom column name
#[sea_orm(column_name = "lAsTnAmE")]
pub last_name: String,

// Column type override
#[sea_orm(column_type = "Text")]
pub description: String,

// Multiple attributes
#[sea_orm(column_type = "Text", default_value = "Sam", unique, indexed, nullable)]
pub name: Option<String>,
```

## Table-Level Attributes

```rust
// Rename all columns to camelCase
#[sea_orm(table_name = "user", rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: i32,
    first_name: String,  // maps to "firstName"
}
```

## Composite Primary Key

```rust
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake_filling")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub cake_id: i32,
    #[sea_orm(primary_key)]
    pub filling_id: i32,
}
```

## UUID Primary Key

```rust
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub email: String,
}
```

## JSON Field with Custom Struct

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct KeyValue {
    pub id: i32,
    pub name: String,
    pub price: f32,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "json_struct")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub json_value: KeyValue,
    pub json_value_opt: Option<KeyValue>,
}
```

## PostgreSQL Array Type

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "collection")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub integers: Vec<i32>,
    pub integers_opt: Option<Vec<i32>>,
    pub floats: Vec<f32>,
    pub doubles: Vec<f64>,
    pub strings: Vec<String>,
}
```

## Relation Types

### has_many

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}
```

### belongs_to

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cake::Entity",
        from = "Column::CakeId",
        to = "super::cake::Column::Id"
    )]
    Cake,
}
```

### has_one

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::cake_topping::Entity")]
    CakeTopping,
}
```

## Required Derives Summary

| Type | Required Derives |
|------|-----------------|
| Model struct | `Clone, Debug, PartialEq, Eq, DeriveEntityModel` |
| Relation enum | `Copy, Clone, Debug, EnumIter, DeriveRelation` |
| JSON field struct | `Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult` |
