use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{Expr, Func},
};
use uuid::Uuid;

use crate::{
    domain::{
        error::repository_error::RepositoryError,
        types::{role::UserRole as DomainUserRole, user::User},
    },
    schema::users,
};

pub struct PostgresUserRepository {
    db: DatabaseConnection,
}

impl PostgresUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

// --- Role conversions ---

impl From<users::UserRole> for DomainUserRole {
    fn from(role: users::UserRole) -> Self {
        match role {
            users::UserRole::Owner => DomainUserRole::Owner,
            users::UserRole::Admin => DomainUserRole::Admin,
            users::UserRole::User => DomainUserRole::User,
        }
    }
}

impl From<DomainUserRole> for users::UserRole {
    fn from(role: DomainUserRole) -> Self {
        match role {
            DomainUserRole::Owner => users::UserRole::Owner,
            DomainUserRole::Admin => users::UserRole::Admin,
            DomainUserRole::User => users::UserRole::User,
        }
    }
}

// --- Model-to-domain conversion ---

impl From<users::Model> for User {
    fn from(model: users::Model) -> Self {
        Self {
            id: model.id,
            handle: model.handle,
            name: model.name,
            role: model.role.into(),
            is_active: model.is_active,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }
}

// --- UserRepository implementation ---

use crate::domain::ports::user_repository::UserRepository;

impl UserRepository for PostgresUserRepository {
    async fn save(&self, user: &User) -> Result<User, RepositoryError> {
        let active_model = users::ActiveModel {
            id: Set(user.id),
            handle: Set(user.handle.clone()),
            name: Set(user.name.clone()),
            role: Set(user.role.into()),
            is_active: Set(user.is_active),
            created_at: Set(user.created_at.into()),
            updated_at: Set(user.updated_at.into()),
        };

        let model = users::Entity::insert(active_model)
            .exec_with_returning(&self.db)
            .await?;

        Ok(model.into())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, RepositoryError> {
        let model = users::Entity::find_by_id(id).one(&self.db).await?;

        Ok(model.map(Into::into))
    }

    async fn find_by_handle(&self, handle: &str) -> Result<Option<User>, RepositoryError> {
        let model = users::Entity::find()
            .filter(
                Expr::expr(Func::lower(Expr::col(users::Column::Handle))).eq(handle.to_lowercase()),
            )
            .one(&self.db)
            .await?;

        Ok(model.map(Into::into))
    }

    async fn list(
        &self,
        limit: u64,
        cursor: Option<(DateTime<Utc>, Uuid)>,
        include_inactive: bool,
    ) -> Result<Vec<User>, RepositoryError> {
        let mut query = users::Entity::find()
            .order_by_desc(users::Column::CreatedAt)
            .order_by_desc(users::Column::Id);

        if !include_inactive {
            query = query.filter(users::Column::IsActive.eq(true));
        }

        if let Some((cursor_time, cursor_id)) = cursor {
            let cursor_time_fixed: chrono::DateTime<chrono::FixedOffset> = cursor_time.into();
            query = query.filter(
                sea_orm::Condition::any()
                    .add(users::Column::CreatedAt.lt(cursor_time_fixed))
                    .add(
                        sea_orm::Condition::all()
                            .add(users::Column::CreatedAt.eq(cursor_time_fixed))
                            .add(users::Column::Id.lt(cursor_id)),
                    ),
            );
        }

        let models = query.limit(limit).all(&self.db).await?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn update(&self, user: &User) -> Result<User, RepositoryError> {
        let active_model = users::ActiveModel {
            id: Set(user.id),
            handle: Set(user.handle.clone()),
            name: Set(user.name.clone()),
            role: Set(user.role.into()),
            is_active: Set(user.is_active),
            created_at: Set(user.created_at.into()),
            updated_at: Set(user.updated_at.into()),
        };

        let model = active_model.update(&self.db).await?;

        Ok(model.into())
    }
}
