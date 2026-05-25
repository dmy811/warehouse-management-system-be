use std::sync::Arc;

use sqlx::PgPool;
use deadpool_redis::Pool as RedisPool;

use crate::{infrastructure::{cloudinary::CloudinaryClient, config::Config}, services::container::ServiceContainer};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool, // sqlx sudah thread safe dan sudah pakai Arc secara internal
    pub redis: Arc<RedisPool>,
    pub config: Arc<Config>,
    pub cloudinary: Arc<CloudinaryClient>,
    pub services: ServiceContainer
}

// tujuan Arc itu agar thread-safe, jika tidak maka setiap pakai harus clone, kalau banyak akan mahal. dengan Arc hanya share pointer jadi murah
// Arc digunakan untuk memungkinkan satu data dimiliki oleh banyak bagian program (multi ownership) secara aman di multi-thread, tanpa harus copy data.
// jadi 1 data banyak referensi
// clone() biasa yang terjadi: semua isi struct di-copy, alokasi memory baru, bisa mahal
// clone() pada Arc yang terjadi: hanya tambah counter (atomic +1), tidak copy data, super cepat
impl AppState {
    pub fn new(db: PgPool, redis_pool: RedisPool, config: Config) -> Self {
        let redis = Arc::new(redis_pool);
        let config = Arc::new(config);
        let cloudinary = Arc::new(CloudinaryClient::new(config.cloudinary.clone()));
        let services = ServiceContainer::new(&db, &config, &redis);
        Self {
            db,
            redis,
            config,
            cloudinary,
            services
        }
    }
}