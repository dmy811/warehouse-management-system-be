use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    dtos::{ListWarehouseQuery, warehouse_dto::UpdateField},
    errors::AppResult,
    models::{Warehouse, WarehouseWithStats}
};

#[async_trait]
pub trait WarehouseRepositoryTrait: Send + Sync {
    async fn find_all(&self, query: &ListWarehouseQuery) -> AppResult<(Vec<WarehouseWithStats>, i64)>;
    async fn find_by_id(&self, id: i64) -> AppResult<Option<Warehouse>>;
    async fn name_exists(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool>;
    async fn create(&self, name: &str, address: &str, phone: Option<&str>, photo: Option<&str>) -> AppResult<Warehouse>;
    async fn update(&self, id: i64, name: Option<&str>, address: Option<&str>, phone: UpdateField<&str>, photo: UpdateField<&str>) -> AppResult<Option<Warehouse>>;
    async fn soft_delete(&self, id: i64) -> AppResult<bool>;
    async fn update_photo(&self, id: i64, photo_url: &str) -> AppResult<()>;
    async fn clear_photo(&self, id: i64) -> AppResult<()>;
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
    async fn find_all(&self, query: &ListWarehouseQuery) -> AppResult<(Vec<WarehouseWithStats>, i64)> {
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
                w.updated_at,
                COUNT(DISTINCT i.product_id)   AS total_products,
                COUNT(DISTINCT r.id)           AS total_racks
            FROM warehouses w
            LEFT JOIN inventories i ON i.warehouse_id = w.id
            LEFT JOIN racks r       ON r.warehouse_id = w.id AND r.deleted_at IS NULL
            WHERE w.deleted_at IS NULL
              AND ($1::TEXT IS NULL OR (
                  LOWER(w.name)    LIKE $1
               OR LOWER(w.address) LIKE $1
              ))
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
              AND ($1::TEXT IS NULL OR (
                  LOWER(w.name)    LIKE $1
               OR LOWER(w.address) LIKE $1
              ))
        "#;
 
        let total: i64 = sqlx::query_scalar(count_sql)
            .bind(&search_pattern)
            .fetch_one(&self.db)
            .await?;
 
        Ok((items, total))
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Option<Warehouse>> {
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            SELECT * FROM warehouses
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(warehouse)
    }

    async fn name_exists(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool> {
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

    async fn create(&self, name: &str, address: &str, phone: Option<&str>, photo: Option<&str>) -> AppResult<Warehouse> {
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

    async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        address: Option<&str>,
        phone: UpdateField<&str>,
        photo: UpdateField<&str>,
    ) -> AppResult<Option<Warehouse>> {
        // let phone_val: Option<&str> = phone.flatten();
        // let photo_val: Option<&str> = photo.flatten();
        // let clear_phone = matches!(phone, Some(None));
        // let clear_photo = matches!(photo, Some(None));
        let (phone_val, clear_phone) = phone.into_parts();
        let (photo_val, clear_photo) = photo.into_parts();
 
        let warehouse = sqlx::query_as!(
            Warehouse,
            r#"
            UPDATE warehouses SET
                name       = COALESCE($2, name),
                address    = COALESCE($3, address),
                phone      = CASE
                               WHEN $6 = TRUE THEN NULL
                               ELSE COALESCE($4, phone)
                             END,
                photo      = CASE
                               WHEN $7 = TRUE THEN NULL
                               ELSE COALESCE($5, photo)
                             END,
                updated_at = NOW()
            WHERE id = $1
              AND deleted_at IS NULL
            RETURNING *
            "#,
            id,
            name,
            address,
            phone_val,
            photo_val,
            clear_phone,
            clear_photo,
        )
        .fetch_optional(&self.db)
        .await?;
 
        Ok(warehouse)
    }

     async fn soft_delete(&self, id: i64) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE warehouses
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(&self.db)
        .await?;
 
        // rows_affected = 0 means warehouse not found or already deleted
        Ok(result.rows_affected() > 0)
    }

    async fn update_photo(&self, id: i64, photo_url: &str) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE warehouses
            SET photo = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id,
            photo_url
        )
        .execute(&self.db)
        .await?;
 
        Ok(())
    }
 
    async fn clear_photo(&self, id: i64) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE warehouses
            SET photo = NULL, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(&self.db)
        .await?;
 
        Ok(())
    }
}