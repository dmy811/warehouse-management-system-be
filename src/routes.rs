pub mod auth_routes;
pub mod health_routes;
pub mod warehouse_routes;

pub use auth_routes::{auth_public_routes, auth_protected_routes};
pub use health_routes::health_routes;
pub use warehouse_routes::warehouse_routes;