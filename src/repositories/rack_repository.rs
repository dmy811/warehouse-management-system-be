use async_trait::async_trait;
use sqlx::PgPool;
use crate::{errors::AppResult, models::{Rack, RackWithStats}, response::ListQuery};

#[async_trait]
pub trait RackRepositoryTrait: Send + Sync {
    async fn find_all(&self, warehouse_id: i64, query: &ListQuery) -> AppResult<(Vec<RackWithStats>, i64)>;
    async fn find_by_id(&self, id: i64, warehouse_id: i64) -> AppResult<Option<Rack>>;
    async fn code_exists(&self, code: &str, warehouse_id: i64) -> AppResult<bool>;
    async fn create(
        &self,
        warehouse_id: i64,
        code: &str,
        zone: Option<&str>,
        level: Option<i32>,
        capacity: Option<i64>,
        description: Option<&str>
    ) -> AppResult<Rack>;
    async fn update(
        &self,
        id: i64,
        warehouse_id: i64,
        code:  Option<&str>,
        zone: Option<&str>,
        level: Option<i32>,
        capacity: Option<i64>,
        description: Option<&str>
    ) -> AppResult<Option<Rack>>;
    async fn soft_delete(&self, id: i64, warehouse_id: i64) -> AppResult<bool>;
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
        
    }
    async fn find_by_id(&self, id: i64, warehouse_id: i64) -> AppResult<Option<Rack>>{

    }
    async fn code_exists(&self, code: &str, warehouse_id: i64) -> AppResult<bool>{

    }
    async fn create(
        &self,
        warehouse_id: i64,
        code: &str,
        zone: Option<&str>,
        level: Option<i32>,
        capacity: Option<i64>,
        description: Option<&str>
    ) -> AppResult<Rack>{

    }
    async fn update(
        &self,
        id: i64,
        warehouse_id: i64,
        code:  Option<&str>,
        zone: Option<&str>,
        level: Option<i32>,
        capacity: Option<i64>,
        description: Option<&str>
    ) -> AppResult<Option<Rack>>{

    }
    async fn soft_delete(&self, id: i64, warehouse_id: i64) -> AppResult<bool>{
        
    }
}