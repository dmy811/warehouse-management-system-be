use async_trait::async_trait;
use sqlx::PgPool;
use crate::{errors::AppResult, models::{Rack, RackWithStats, params::rack_params::{CreateRackParams, UpdateRackParams}}, response::ListQuery};

#[async_trait]
pub trait RackRepositoryTrait: Send + Sync {
    async fn find_all(&self, warehouse_id: i64, query: &ListQuery) -> AppResult<(Vec<RackWithStats>, i64)>;
    async fn find_by_id(&self, id: i64, warehouse_id: i64) -> AppResult<Option<RackWithStats>>;
    async fn code_exists(&self, code: &str, warehouse_id: i64, exclude_id: Option<i64>) -> AppResult<bool>;
    async fn create(&self, params: CreateRackParams<'_>) -> AppResult<Rack>;
    async fn update(
        &self,
        id: i64,
        warehouse_id: i64,
        params: UpdateRackParams<'_>
    ) -> AppResult<Option<Rack>>;
    async fn soft_delete(&self, id: i64, warehouse_id: i64) -> AppResult<()>;
}

pub struct RackRepository {
    db: PgPool
}

impl RackRepository {
    pub fn new(db: PgPool) -> Self {
        Self {
            db
        }
    }
}

#[async_trait]
impl RackRepositoryTrait for RackRepository {
    async fn find_all(&self, warehouse_id: i64, query: &ListQuery) -> AppResult<(Vec<RackWithStats>, i64)>{
        let like_query = query
            .search
            .as_ref()
            .map(|s| format!("%{}%", s.to_lowercase()));

        let sort_col = match query.sort_by(){
            "code" => "r.code",
            "zone" => "r.zone",
            "level" => "r.level",
            "updated_at" => "r.updated_at",
            _ => "r.created_at",
        };

        let sort_dir = query.sort_direction();

        let sql_like = format!(
            r#"
            SELECT
                r.id,
                r.warehouse_id,
                r.code,
                r.zone,
                r.level,
                r.description,
                r.capacity,
                r.deleted_at,
                r.created_at,
                r.updated_at,
            COALESCE(SUM(ir.quantity), 0)::BIGINT AS "used_capacity!",
            COUNT(DISTINCT i.product_id)::BIGINT AS "total_products!"
            FROM racks r
            LEFT JOIN inventory_racks ir ON ir.rack_id = r.id
            LEFT JOIN inventories i ON i.id = ir.inventory_id
            WHERE r.warehouse_id = $1
                AND r.deleted_at IS NULL
                AND ($2::TEXT IS NULL OR (LOWER(r.code) LIKE $2 OR LOWER(r.zone) LIKE $2))
            GROUP BY r.id
            ORDER BY {sort_col} {sort_dir}
            LIMIT $3
            OFFSET $4
            "#
        );

        let items = sqlx::query_as::<_, RackWithStats>(&sql_like)
            .bind(warehouse_id)
            .bind(&like_query)
            .bind(query.per_page())
            .bind(query.offset())
            .fetch_all(&self.db)
            .await?;

        let count_sql = r#"
            SELECT COUNT(*)
            FROM racks r
            WHERE r.warehouse_id = $1
                AND r.deleted_at IS NULL
                AND ($2::TEXT IS NULL OR (LOWER(r.code) LIKE $2 OR LOWER(r.zone) LIKE $2))
        "#;

        let total: i64 = sqlx::query_scalar(count_sql)
            .bind(warehouse_id)
            .bind(&like_query)
            .fetch_one(&self.db)
            .await?;

        Ok((items, total))
    }

    async fn find_by_id(&self, id: i64, warehouse_id: i64) -> AppResult<Option<RackWithStats>>{
        let rack = sqlx::query_as!(
            RackWithStats,
            r#"
            SELECT
                r.id,
                r.warehouse_id,
                r.code,
                r.zone,
                r.level,
                r.description,
                r.capacity,
                r.deleted_at,
                r.created_at,
                r.updated_at,
                COALESCE(SUM(ir.quantity), 0)::BIGINT AS "used_capacity!",
                COUNT(DISTINCT i.product_id)::BIGINT AS "total_products!"
            FROM racks r
            LEFT JOIN inventory_racks ir ON ir.rack_id = r.id
            LEFT JOIN inventories i ON i.id = ir.inventory_id
            WHERE r.id = $1 AND r.warehouse_id = $2 AND r.deleted_at IS NULL
            GROUP BY r.id
            "#,
            id,
            warehouse_id
        ).fetch_optional(&self.db)
        .await?;

        Ok(rack)
    }
    
    async fn code_exists(
        &self,
        code: &str,
        warehouse_id: i64,
        exclude_id: Option<i64>,
    ) -> AppResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM racks
                WHERE LOWER(code) = LOWER($1)
                  AND warehouse_id = $2
                  AND deleted_at IS NULL
                  AND ($3::BIGINT IS NULL OR id != $3)
            )
            "#,
            code,
            warehouse_id,
            exclude_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(false);
 
        Ok(exists)
    }

    async fn create(&self, params: CreateRackParams<'_>) -> AppResult<Rack>{
        let rack = sqlx::query_as!(
            Rack,
            r#"
            INSERT INTO racks (warehouse_id, code, zone, level, capacity, description)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, warehouse_id, code, zone, level, description, capacity, deleted_at, created_at, updated_at
            "#,
            params.warehouse_id,
            params.code,
            params.zone,
            params.level,
            params.capacity,
            params.description
        )
        .fetch_one(&self.db)
        .await?;

        Ok(rack)
    }

    async fn update(
        &self,
        id: i64,
        warehouse_id: i64,
        params: UpdateRackParams<'_>
    ) -> AppResult<Option<Rack>>{
        let rack = sqlx::query_as!(
            Rack,
            r#"
            UPDATE racks SET
                code          = COALESCE($3, code),
                zone          = COALESCE($4, zone),
                level         = COALESCE($5, level),
                capacity      = COALESCE($6, capacity),
                description   = COALESCE($7, description),
                updated_at = NOW()
            WHERE id = $1
                AND warehouse_id = $2
                AND deleted_at IS NULL
            RETURNING id, warehouse_id, code, zone, level, description, capacity, deleted_at, created_at, updated_at
            "#,
            id,
            warehouse_id,
            params.code,
            params.zone,
            params.level,
            params.capacity,
            params.description
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(rack)
    }
    async fn soft_delete(&self, id: i64, warehouse_id: i64) -> AppResult<()>{
        sqlx::query!(
            r#"
            UPDATE racks
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND warehouse_id = $2 AND deleted_at IS NULL
            "#,
            id,
            warehouse_id
        )
        .execute(&self.db)
        .await?;
 
        Ok(())
    }
}