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
    async fn find_warehouse_by_id(&self, id: i64) -> AppResult<Option<WarehouseWithStats>>;
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

// Kenapa di function parameter &str bisa tanpa 'a?
/* misal kode ini 
async fn create(
    &self,
    name: &str,
    email: &str,
) -> AppResult<User>;

Yang sebenarnya compiler baca:

async fn create<'a, 'b, 'c>(
    &'a self,
    name: &'b str,
    email: &'c str,
) -> AppResult<User>;

Rust punya aturan bernama Lifetime Elision — compiler otomatis menyimpulkan lifetime yang obvious tanpa ditulis eksplisit.

Compiler assign lifetime berbeda ke setiap parameter secara otomatis. Ini aman karena:
Output function tidak mengandung referensi ke parameter manapun. Return type-nya AppResult<User> — User adalah owned struct, bukan referensi. Jadi compiler tidak perlu tahu mana lifetime yang harus di-carry ke return value.

Kapan lifetime elision TIDAK bisa bekerja
Lifetime elision gagal ketika return type mengandung referensi dan compiler tidak bisa tahu referensi itu dari mana asalnya:

function parameter mekanismenya begitu, berbeda jika parameternya pakai struct
Function parameter → lifetime-nya PASTI berakhir saat function return
                     compiler sudah tahu ini, tidak perlu diberitahu

Struct field       → struct bisa hidup seberapa lama saja
                     compiler tidak tahu kapan struct di-drop
                     harus diberitahu eksplisit lewat 'a

kenapa di repository pakai &str bukan String adalah karena Di dalam function create, itu hanya membaca name untuk dimasukkan ke SQL query. Tidak perlu memilikinya. Meminjam (&str) sudah cukup dan jauh lebih efisien.
kalai pakai String ada alokasi memory baru harus gunakan req.name.clone()  // clone = alokasi baru di heap
Dengan &str — tidak ada alokasi  &req.name  // hanya pinjam, tidak ada alokasi
*/

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
        // let like_query = query
        //     .search
        //     .as_ref()
        //     .map(|s| format!("%{}%", s.to_lowercase()));

        let fts_query = query
            .search
            .as_ref()
            .map(|s| {
                s.split_whitespace()
                    .filter(|w| !w.is_empty())
                    .map(|w| format!("{}:*", w.to_lowercase()))
                    .collect::<Vec<_>>()
                    .join(" & ")
            });
 
        let sort_col = match query.sort_by() {
            "name" => "w.name",
            "updated_at" => "w.updated_at",
            _ => "w.created_at"
        };

        let sort_dir = query.sort_direction();
 
        // let sql_like = format!(
        //     r#"
        //     SELECT
        //         w.id,
        //         w.name,
        //         w.address,
        //         w.phone,
        //         w.photo,
        //         w.deleted_at,
        //         w.created_at,
        //         w.updated_at,
        //         COUNT(DISTINCT i.product_id)   AS total_products,
        //         COUNT(DISTINCT r.id)           AS total_racks
        //     FROM warehouses w
        //     LEFT JOIN inventories i ON i.warehouse_id = w.id
        //     LEFT JOIN racks r       ON r.warehouse_id = w.id AND r.deleted_at IS NULL
        //     WHERE w.deleted_at IS NULL
        //       AND ($1::TEXT IS NULL OR (LOWER(w.name) LIKE $1 OR LOWER(w.address) LIKE $1))
        //     GROUP BY w.id
        //     ORDER BY {sort_col} {sort_dir}
        //     LIMIT $2
        //     OFFSET $3
        //     "#
        // );

        let sql_fts = format!(
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
                COUNT(DISTINCT i.product_id) AS total_products,
                COUNT(DISTINCT r.id) AS total_racks
            FROM warehouses w
            LEFT JOIN inventories i ON i.warehouse_id = w.id
            LEFT JOIN racks r ON r.warehouse_id = w.id AND r.deleted_at IS NULL
            WHERE w.deleted_at IS NULL
                AND ($1::TEXT IS NULL OR w.search_vector @@ to_tsquery('simple', $1))
            GROUP BY w.id
            ORDER BY
                CASE WHEN $1 IS NOT NULL
                    THEN ts_rank(w.search_vector, to_tsquery('simple', $1))
                    ELSE 0
                END DESC,
                {sort_col} {sort_dir}
            LIMIT $2
            OFFSET $3
            "#
        );
 
        let items = sqlx::query_as::<_, WarehouseWithStats>(&sql_fts)
            .bind(&fts_query)
            .bind(query.per_page())
            .bind(query.offset())
            .fetch_all(&self.db)
            .await?;

        let count_sql = r#"
            SELECT COUNT(*)
            FROM warehouses w
            WHERE w.deleted_at IS NULL
              AND ($1::TEXT IS NULL OR w.search_vector @@ to_tsquery('simple', $1))
        "#;
 
        let total: i64 = sqlx::query_scalar(count_sql)
            .bind(&fts_query)
            .fetch_one(&self.db)
            .await?;
 
        Ok((items, total))
    }

    async fn find_warehouse_by_id(&self, warehouse_id: i64) -> AppResult<Option<WarehouseWithStats>> {
        let warehouse = sqlx::query_as!(
            WarehouseWithStats,
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
                COUNT(DISTINCT i.product_id) AS total_products,
                COUNT(DISTINCT r.id) AS total_racks
            FROM warehouses w
            LEFT JOIN inventories i ON i.warehouse_id = w.id
            LEFT JOIN racks r ON r.warehouse_id = w.id AND r.deleted_at IS NULL
            WHERE w.id = $1 AND w.deleted_at IS NULL
            GROUP BY w.id
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
            RETURNING id, name, address, phone, photo, deleted_at, created_at, updated_at
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
            RETURNING id, name, address, phone, photo, deleted_at, created_at, updated_at
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