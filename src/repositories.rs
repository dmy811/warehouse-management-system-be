pub mod auth_repository;
pub mod warehouse_repository;
pub mod user_repository;
pub mod user_role_repository;
pub mod rack_repository;
pub mod user_warehouse_repository;
pub mod roles_repository;

pub use auth_repository::{AuthRepository, AuthRepositoryTrait};
pub use warehouse_repository::{WarehouseRepository, WarehouseRepositoryTrait};