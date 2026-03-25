use std::any;

use crate::{infrastructure::db::DBPool, models::warehouses::{CreateWarehouseRequest, UpdateWarehouseRequest, Warehouse}};

pub async fn create_warehouse(
    pool: &DBPool,
    payload: &CreateWarehouseRequest
) -> anyhow::Result<Warehouse> {
    let created = sqlx::query_as!(
        Warehouse,
        r#"
        INSERT INTO public.warehouses (name, address, photo, phone)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, address, photo, phone, deleted_at, updated_at, created_at
        "#,
        payload.name,
        payload.address,
        payload.photo,
        payload.phone
    )
    .fetch_one(pool)
    .await?;

    Ok(created)
}

pub async fn find_warehouse_by_id(pool: &DBPool, id: i64) -> anyhow::Result<Option<Warehouse>> {
    let warehouse = sqlx::query_as!(
        Warehouse,
        r#"
        SELECT id, name, address, photo, phone, deleted_at, updated_at, created_at
        FROM public.warehouses
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(warehouse)
}

pub async fn find_all_warehouses(pool: &DBPool, limit: i64, offset: i64) -> anyhow::Result<Vec<Warehouse>> {
    let warehouses: Vec<Warehouse> = sqlx::query_as!(
        Warehouse,
        r#"
        SELECT id, name, address, photo, phone, deleted_at, updated_at, created_at
        FROM public.warehouses
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(warehouses)
}

pub async fn count_warehouse(pool: &DBPool) -> anyhow::Result<i64> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM public.warehouses
        WHERE deleted_at IS NULL
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

pub async fn update_warehouse(pool: &DBPool, id: i64, payload: &UpdateWarehouseRequest) -> anyhow::Result<Option<Warehouse>> {
    let updated = sqlx::query_as!(
        Warehouse,
        r#"
        UPDATE public.warehouses
        SET name = $1, address = $2, photo = $3, phone = $4, updated_at = NOW()
        WHERE id = $5 AND deleted_at IS NULL
        RETURNING id, name, address, photo, phone, deleted_at, updated_at, created_at
        "#,
        payload.name,
        payload.address,
        payload.photo,
        payload.phone,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(updated)
}

pub async fn delete_warehouse(pool: &DBPool, id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE public.warehouses
        SET deleted_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}