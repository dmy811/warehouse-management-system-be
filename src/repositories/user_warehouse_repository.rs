use async_trait::async_trait;
use sqlx::PgPool;

use crate::errors::AppResult;
#[async_trait]
pub trait UserWarehouseRepositoryTrait: Send + Sync {
    async fn check_assign(&self, user_id: i64, warehouse_id: i64) -> AppResult<bool>;
    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()>;
}

pub struct UserWarehouseRepository {
    db: PgPool
}

impl UserWarehouseRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl UserWarehouseRepositoryTrait for UserWarehouseRepository {
    async fn check_assign(&self, user_id: i64, warehouse_id: i64) -> AppResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM user_warehouses
                WHERE user_id = $1 AND warehouse_id = $2
            )
            "#,
            user_id,
            warehouse_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        Ok(exists)
    }
    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_warehouses (user_id, warehouse_id)
            VALUES ($1, $2)
            "#,
            user_id,
            warehouse_id
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }
}