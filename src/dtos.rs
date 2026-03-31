pub mod auth_dto;
pub mod warehouse_dto;
pub mod user_dto;

pub use auth_dto::{AuthResponse, LoginRequest, RegisterRequest, UserResponse};
pub use warehouse_dto::{
    CreateWarehouseRequest, UpdateWarehouseRequest, WarehouseResponse,
    WarehouseSummary,
};