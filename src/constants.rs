pub mod roles {
    pub const ADMIN: &str = "ADMIN";
    pub const MANAGER: &str = "MANAGER";
    pub const STAFF: &str = "STAFF";
    pub const VIEWER: &str = "VIEWER";
}

pub mod permissions {
    use crate::constants::roles::*;

    pub const CAN_APPROVE: &[&str] = &[ADMIN, MANAGER];

    pub const CAN_MANAGE_MASTER: &[&str] = &[ADMIN, MANAGER];

    pub const CAN_MANAGE_USERS: &[&str] = &[ADMIN];

    pub const CAN_CREATE_TRANSACTION: &[&str] = &[ADMIN, MANAGER, STAFF];

    pub const ALL_ROLES: &[&str] = &[ADMIN, MANAGER, STAFF, VIEWER];

    // Hanya ADMIN bisa delete warehouse
    // pub async fn delete(...) {
    //     require_roles(permissions::CAN_MANAGE_USERS)(auth_user.clone())?;
    //     ...
    // }

    // ADMIN dan MANAGER bisa approve goods receipt
    // pub async fn approve_receipt(...) {
    //     require_roles(permissions::CAN_APPROVE)(auth_user.clone())?;
    //     ...
    // }

    // services/goods_receipt_service.rs
    // async fn create(&self, req: CreateReceiptRequest, actor: &AuthUser) -> AppResult<...> {

    //     // Kalau STAFF, validasi apakah dia punya akses ke warehouse ini
    //     if actor.role == roles::STAFF {
    //         let has_access = self.user_warehouse_repo
    //             .check_access(actor.id, req.warehouse_id)
    //             .await?;

    //         if !has_access {
    //             return Err(AppError::Forbidden);
    //         }
    //     }

    //     // ADMIN dan MANAGER bisa akses semua warehouse
    //     ...
    // }
}

pub mod pagination {
    pub const DEFAULT_PAGE: i64 = 1;
    pub const DEFAULT_PER_PAGE: i64 = 20;
    pub const MAX_PER_PAGE: i64 = 100;
}

pub mod file_upload {
    pub const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5mb
    pub const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
    pub const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];
    pub const PHOTO_FIELD: &str = "photo";
}

pub mod stock {
    pub const LOW_STOCK_MULTIPLIER: f64 = 1.2;
}

pub mod headers {
    pub const REQUEST_ID: &str = "x-request-id";
    pub const REQUEST_ID_RESPONSE: &str = "x-request-id";
}