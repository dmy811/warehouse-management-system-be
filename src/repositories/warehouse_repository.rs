use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::AppResult,
    models::{UserWithRole, Warehouse, WarehouseWithStats},
    response::ListQuery
};

#[async_trait]
pub trait WarehouseRepositoryTrait: Send + Sync {
    async fn find_all_warehouses(&self, query: &ListQuery) -> AppResult<(Vec<WarehouseWithStats>, i64)>;
    async fn find_warehouse_by_id(&self, id: i64) -> AppResult<Option<Warehouse>>;
    async fn check_name_exists(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool>;
    async fn create_warehouse(&self, name: &str, address: &str, phone: Option<&str>, photo: Option<&str>) -> AppResult<Warehouse>;
    async fn update_warehouse(&self, warehouse_id: i64, name: Option<&str>, address: Option<&str>, phone: Option<&str>, photo: Option<&str>) -> AppResult<Option<Warehouse>>;
    async fn warehouse_soft_delete(&self, warehouse_id: i64) -> AppResult<bool>;
    async fn warehouse_hard_delete(&self, warehouse_id: i64) -> AppResult<bool>;
    async fn update_warehouse_photo(&self, warehouse_id: i64, photo_url: &str) -> AppResult<()>;
    async fn clear_warehouse_photo(&self, warehouse_id: i64) -> AppResult<()>;
    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()>;
    async fn check_existing_warehouse_in_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<bool>;
    async fn check_user_existing(&self, user_id: i64) -> AppResult<bool>;
}

pub struct WarehouseRepository {
    db: PgPool
}

impl WarehouseRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl WarehouseRepositoryTrait for WarehouseRepository {
    async fn find_all_warehouses(&self, query: &ListQuery) -> AppResult<(Vec<WarehouseWithStats>, i64)> {
        let search_pattern = query
            .search
            .as_ref()
            .map(|s| format!("%{}%", s.to_lowercase()));
 
        let sort_col = query.sort_column();
        let sort_dir = query.sort_direction();
 
        let sql = format!(
            r#"
            SELECT
                w.id,
                w.name,
                w.address,
                w.phone,
                w.photo,
                w.deleted_at,
                w.created_at,
                w.updated_at,warehouse_id
                COUNT(DISTINCT i.product_id)   AS total_products,
                COUNT(DISTINCT r.id)           AS total_racks
            FROM warehouses w
            LEFT JOIN inventories i ON i.warehouse_id = w.id
            LEFT JOIN racks r       ON r.warehouse_id = w.id AND r.deleted_at IS NULL
            WHERE w.deleted_at IS NULL
              AND ($1::TEXT IS NULL OR (LOWER(w.name) LIKE $1 OR LOWER(w.address) LIKE $1))
            GROUP BY w.id
            ORDER BY {sort_col} {sort_dir}
            LIMIT $2
            OFFSET $3
            "#
        );
 
        let items = sqlx::query_as::<_, WarehouseWithStats>(&sql)
            .bind(&search_pattern)
            .bind(query.per_page())
            .bind(query.offset())
            .fetch_all(&self.db)
            .await?;

        let count_sql = r#"
            SELECT COUNT(*)
            FROM warehouses w
            WHERE w.deleted_at IS NULL
              AND ($1::TEXT IS NULL OR (LOWER(w.name) LIKE $1 OR LOWER(w.address) LIKE $1))
        "#;
 
        let total: i64 = sqlx::query_scalar(count_sql)
            .bind(&search_pattern)
            .fetch_one(&self.db)
            .await?;
 
        Ok((items, total))
    }

    async fn find_warehouse_by_id(&self, warehouse_id: i64) -> AppResult<Option<Warehouse>> {
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            SELECT * FROM warehouses
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            warehouse_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(warehouse)
    }

    async fn check_name_exists(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM warehouses
                WHERE LOWER(name) = LOWER($1)
                  AND deleted_at IS NULL
                  AND ($2::BIGINT IS NULL OR id != $2)
            )
            "#,
            name,
            exclude_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);
 
        Ok(exists)
    }

    async fn create_warehouse(&self, name: &str, address: &str, phone: Option<&str>, photo: Option<&str>) -> AppResult<Warehouse> {
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            INSERT INTO warehouses (name, address, photo, phone)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            name,
            address,
            phone,
            photo
        )
        .fetch_one(&self.db)
        .await?;
 
        Ok(warehouse)

    }

    async fn update_warehouse(
        &self,
        warehouse_id: i64,
        name: Option<&str>,
        address: Option<&str>,
        phone: Option<&str>,
        photo: Option<&str>,
    ) -> AppResult<Option<Warehouse>> {
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            UPDATE warehouses SET
                name       = COALESCE($2, name),
                address    = COALESCE($3, address),
                phone      = COALESCE($4, phone),
                photo      = COALESCE($5, photo),
                updated_at = NOW()
            WHERE id = $1
            AND deleted_at IS NULL
            RETURNING *
            "#,
            warehouse_id,
            name,
            address,
            phone,
            photo
        )
        .fetch_optional(&self.db)
        .await?;
 
        Ok(warehouse)
    }

     async fn warehouse_soft_delete(&self, warehouse_id: i64) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE warehouses
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            warehouse_id
        )
        .execute(&self.db)
        .await?;
 
        // rows_affected = 0 means warehouse not found or already deleted
        Ok(result.rows_affected() > 0)
    }

        async fn warehouse_hard_delete(&self, warehouse_id: i64) -> AppResult<bool> {
            let result = sqlx::query!(
                r#"
                DELETE FROM warehouses
                WHERE id = $1
                "#,
                warehouse_id
            )
            .execute(&self.db)
            .await?;

            Ok(result.rows_affected() > 0)
        }

    async fn update_warehouse_photo(&self, warehouse_id: i64, photo_url: &str) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE warehouses
            SET photo = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            warehouse_id,
            photo_url
        )
        .execute(&self.db)
        .await?;
 
        Ok(())
    }
 
    async fn clear_warehouse_photo(&self, warehouse_id: i64) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE warehouses
            SET photo = NULL, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            warehouse_id
        )
        .execute(&self.db)
        .await?;
 
        Ok(())
    }

    async fn assign_warehouse_to_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT into user_warehouses (user_id, warehouse_id)
            VALUES ($1, $2)
            "#,
            user_id,
            warehouse_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn check_existing_warehouse_in_user(&self, user_id: i64, warehouse_id: i64) -> AppResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
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

    async fn check_user_existing(&self, user_id: i64) -> AppResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE id = $1 AND deleted_at IS NULL
            )
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);

        Ok(exists)
    }
}