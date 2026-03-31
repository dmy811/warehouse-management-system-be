mod common;
use common::{TestApp};
use serde_json::json;


// Helper: register an admin user and return their token
async fn admin_token(app: &TestApp) -> String {
    let token = app.register_user("admin@wms.com", "Password123").await;
    let user_id = app.get_user_id(&token).await;
    app.assign_role(user_id, "ADMIN").await;

    // Re-login to get a token that carries the ADMIN role in claims
    let resp = app.post("/api/v1/auth/login", json!({
        "email": "admin@wms.com",
        "password": "Password123",
    })).await;
    let body: serde_json::Value = resp.json().await.unwrap();
    body["data"]["access_token"].as_str().unwrap().to_string()
}

async fn staff_token(app: &TestApp) -> String {
    let token = app.register_user("staff@wms.com", "Password123").await;
    let user_id = app.get_user_id(&token).await;
    app.assign_role(user_id, "STAFF").await;

    let resp = app.post("/api/v1/auth/login", json!({
        "email": "staff@wms.com",
        "password": "Password123",
    })).await;
    let body: serde_json::Value = resp.json().await.unwrap();
    body["data"]["access_token"].as_str().unwrap().to_string()
}

// ── POST /api/v1/warehouses ───────────────────────────────────────────────────

#[tokio::test]
async fn test_create_warehouse_as_admin_returns_201() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    let resp = app.post_authed("/api/v1/warehouses", json!({
        "name": "Gudang Utama Jakarta",
        "address": "Jl. Raya Bekasi KM 18, Jakarta Timur",
        "phone": "02183214321",
    }), &token).await;

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["id"].is_number());
    assert_eq!(body["data"]["name"], "Gudang Utama Jakarta");
    assert_eq!(body["data"]["address"], "Jl. Raya Bekasi KM 18, Jakarta Timur");
    // deleted_at must never appear in response
    assert!(body["data"]["deleted_at"].is_null());
}

#[tokio::test]
async fn test_create_warehouse_as_staff_returns_403() {
    let app = TestApp::new().await;
    let token = staff_token(&app).await;

    let resp = app.post_authed("/api/v1/warehouses", json!({
        "name": "Gudang Staff",
        "address": "Jl. Test No. 1",
    }), &token).await;

    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "FORBIDDEN");
}

#[tokio::test]
async fn test_create_warehouse_without_auth_returns_401() {
    let app = TestApp::new().await;

    let resp = app.client
        .post(format!("{}/api/v1/warehouses", app.base_url))
        .json(&json!({ "name": "Test", "address": "Jl. Test" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_create_warehouse_duplicate_name_returns_409() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    let payload = json!({
        "name": "Gudang Duplikat",
        "address": "Jl. Test No. 1",
    });

    let resp1 = app.post_authed("/api/v1/warehouses", payload.clone(), &token).await;
    assert_eq!(resp1.status(), 201);

    let resp2 = app.post_authed("/api/v1/warehouses", payload, &token).await;
    assert_eq!(resp2.status(), 409);
    let body: serde_json::Value = resp2.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CONFLICT");
}

#[tokio::test]
async fn test_create_warehouse_case_insensitive_name_uniqueness() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    app.post_authed("/api/v1/warehouses", json!({
        "name": "Gudang Bandung",
        "address": "Jl. Sudirman No. 1",
    }), &token).await;

    // Different case, same name — should also conflict
    let resp = app.post_authed("/api/v1/warehouses", json!({
        "name": "gudang bandung",
        "address": "Jl. Sudirman No. 2",
    }), &token).await;

    assert_eq!(resp.status(), 409);
}

#[tokio::test]
async fn test_create_warehouse_short_name_returns_422() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    let resp = app.post_authed("/api/v1/warehouses", json!({
        "name": "A", // min 2 chars
        "address": "Jl. Test No. 1",
    }), &token).await;

    assert_eq!(resp.status(), 422);
}

// ── GET /api/v1/warehouses ────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_warehouses_returns_paginated_response() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    // Seed 3 warehouses
    for i in 1..=3 {
        app.create_warehouse(&format!("Gudang {}", i), &token).await;
    }

    let resp = app.get_authed("/api/v1/warehouses", &token).await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    assert!(body["meta"]["total"].as_i64().unwrap() >= 3);
    assert!(body["meta"]["page"].is_number());
    assert!(body["meta"]["per_page"].is_number());
    assert!(body["meta"]["total_pages"].is_number());
}

#[tokio::test]
async fn test_list_warehouses_pagination_works() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    for i in 1..=5 {
        app.create_warehouse(&format!("Gudang Paging {}", i), &token).await;
    }

    let resp = app.client
        .get(format!("{}/api/v1/warehouses?page=1&per_page=2", app.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["meta"]["per_page"], 2);
}

#[tokio::test]
async fn test_list_warehouses_search_filters_results() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    app.create_warehouse("Gudang Surabaya", &token).await;
    app.create_warehouse("Gudang Bandung", &token).await;

    let resp = app.client
        .get(format!("{}/api/v1/warehouses?search=surabaya", app.base_url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let items = body["data"].as_array().unwrap();

    assert!(items.iter().all(|w| {
        w["name"].as_str().unwrap().to_lowercase().contains("surabaya")
    }));
}

// ── GET /api/v1/warehouses/:id ────────────────────────────────────────────────

#[tokio::test]
async fn test_get_warehouse_by_id_returns_200() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;
    let id = app.create_warehouse("Gudang Detail", &token).await;

    let resp = app.get_authed(&format!("/api/v1/warehouses/{}", id), &token).await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["id"], id);
    assert_eq!(body["data"]["name"], "Gudang Detail");
}

#[tokio::test]
async fn test_get_nonexistent_warehouse_returns_404() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    let resp = app.get_authed("/api/v1/warehouses/999999", &token).await;

    assert_eq!(resp.status(), 404);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

// ── PATCH /api/v1/warehouses/:id ─────────────────────────────────────────────

#[tokio::test]
async fn test_update_warehouse_partial_fields() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;
    let id = app.create_warehouse("Gudang Lama", &token).await;

    let resp = app.patch_authed(
        &format!("/api/v1/warehouses/{}", id),
        json!({ "name": "Gudang Baru" }),
        &token,
    ).await;

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["name"], "Gudang Baru");
}

#[tokio::test]
async fn test_update_warehouse_empty_body_returns_422() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;
    let id = app.create_warehouse("Gudang Empty Patch", &token).await;

    let resp = app.patch_authed(
        &format!("/api/v1/warehouses/{}", id),
        json!({}),
        &token,
    ).await;

    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn test_update_nonexistent_warehouse_returns_404() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;

    let resp = app.patch_authed(
        "/api/v1/warehouses/999999",
        json!({ "name": "Ghost Warehouse" }),
        &token,
    ).await;

    assert_eq!(resp.status(), 404);
}

// ── DELETE /api/v1/warehouses/:id ────────────────────────────────────────────

#[tokio::test]
async fn test_delete_warehouse_as_admin_returns_204() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;
    let id = app.create_warehouse("Gudang Hapus", &token).await;

    let resp = app.delete_authed(
        &format!("/api/v1/warehouses/{}", id),
        &token,
    ).await;

    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn test_deleted_warehouse_not_accessible_via_get() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;
    let id = app.create_warehouse("Gudang Soft Delete", &token).await;

    // Delete it
    app.delete_authed(&format!("/api/v1/warehouses/{}", id), &token).await;

    // Should now return 404
    let resp = app.get_authed(&format!("/api/v1/warehouses/{}", id), &token).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_soft_delete_does_not_remove_db_row() {
    let app = TestApp::new().await;
    let token = admin_token(&app).await;
    let id = app.create_warehouse("Gudang Soft Delete Check", &token).await;

    app.delete_authed(&format!("/api/v1/warehouses/{}", id), &token).await;

    // Row still exists in DB, just with deleted_at set
    let row = sqlx::query!(
        "SELECT deleted_at FROM warehouses WHERE id = $1",
        id
    )
    .fetch_one(&app.db)
    .await
    .expect("Row should still exist after soft delete");

    assert!(row.deleted_at.is_some(), "deleted_at should be set after soft delete");
}

#[tokio::test]
async fn test_delete_warehouse_as_staff_returns_403() {
    let app = TestApp::new().await;
    let admin = admin_token(&app).await;
    let staff = staff_token(&app).await;

    let id = app.create_warehouse("Gudang Staff Delete", &admin).await;

    let resp = app.delete_authed(
        &format!("/api/v1/warehouses/{}", id),
        &staff,
    ).await;

    assert_eq!(resp.status(), 403);
}