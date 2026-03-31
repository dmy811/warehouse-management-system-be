mod common;

use common::TestApp;
use serde_json::json;


// --- POST /api/v1/auth/register ---
#[tokio::test]
async fn test_register_success_returns_201_with_token(){
    let app = TestApp::new().await;

    let resp = app.post("/api/v1/auth/register", json!({
        "name": "Budi Santoso",
        "email": "budi@example.com",
        "password": "Password123",
    })).await;

    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = resp.json().await.unwrap();

    assert_eq!(body["success"], true);
    assert!(body["data"]["access_token"].is_string());
    assert_eq!(body["data"]["token_type"], "Bearer");
    assert_eq!(body["data"]["user"]["email"], "budi@example.com");
 
    let body_str = body.to_string();
    assert!(!body_str.contains("Password123"));
    assert!(!body_str.contains("$argon2"));
}

#[tokio::test]
async fn test_register_duplicate_email_returns_409() {
    let app = TestApp::new().await;
 
    let payload = json!({
        "name": "User One",
        "email": "duplicate@example.com",
        "password": "Password123",
    });
 
    // First registration succeeds
    let resp1 = app.post("/api/v1/auth/register", payload.clone()).await;
    assert_eq!(resp1.status(), 201);
 
    // Second registration with same email fails
    let resp2 = app.post("/api/v1/auth/register", payload).await;
    assert_eq!(resp2.status(), 409);
 
    let body: serde_json::Value = resp2.json().await.unwrap();
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "CONFLICT");
}

#[tokio::test]
async fn test_register_invalid_email_returns_422() {
    let app = TestApp::new().await;
 
    let resp = app.post("/api/v1/auth/register", json!({
        "name": "Test",
        "email": "not-an-email",
        "password": "Password123",
    })).await;
 
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_register_missing_required_fields_returns_422() {
    let app = TestApp::new().await;
 
    let resp = app.post("/api/v1/auth/register", json!({
        "name": "Test",
    })).await;
 
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
async fn test_register_short_name_returns_422() {
    let app = TestApp::new().await;
 
    let resp = app.post("/api/v1/auth/register", json!({
        "name": "A", // min 2 chars
        "email": "test@example.com",
        "password": "Password123",
    })).await;
 
    assert_eq!(resp.status(), 422);
}

// --- POST /api/v1/auth/login ---
#[tokio::test]
async fn test_login_success_returns_200_with_token() {
    let app = TestApp::new().await;
    app.register_user("login_ok@example.com", "Password123").await;
 
    let resp = app.post("/api/v1/auth/login", json!({
        "email": "login_ok@example.com",
        "password": "Password123",
    })).await;
 
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(body["data"]["access_token"].is_string());
}

#[tokio::test]
async fn test_login_wrong_password_returns_401() {
    let app = TestApp::new().await;
    app.register_user("wrongpw@example.com", "Password123").await;
 
    let resp = app.post("/api/v1/auth/login", json!({
        "email": "wrongpw@example.com",
        "password": "WrongPassword",
    })).await;
 
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
}

#[tokio::test]
async fn test_login_nonexistent_user_returns_401() {
    let app = TestApp::new().await;
 
    let resp = app.post("/api/v1/auth/login", json!({
        "email": "ghost@example.com",
        "password": "Password123",
    })).await;
 
    assert_eq!(resp.status(), 401);
}
 
 #[tokio::test]
async fn test_login_wrong_email_and_wrong_password_return_identical_errors() {
    let app = TestApp::new().await;
    app.register_user("exists@example.com", "Password123").await;
 
    let err_no_user = app.post("/api/v1/auth/login", json!({
        "email": "doesnotexist@example.com",
        "password": "Password123",
    })).await.json::<serde_json::Value>().await.unwrap();
 
    let err_wrong_pw = app.post("/api/v1/auth/login", json!({
        "email": "exists@example.com",
        "password": "WrongPassword",
    })).await.json::<serde_json::Value>().await.unwrap();
 
    assert_eq!(
        err_no_user["error"]["code"],
        err_wrong_pw["error"]["code"],
        "Error codes must be identical to prevent user enumeration"
    );
    assert_eq!(
        err_no_user["error"]["message"],
        err_wrong_pw["error"]["message"],
        "Error messages must be identical to prevent user enumeration"
    );
}

// --- POST /api/v1/auth/me ---
#[tokio::test]
async fn test_me_with_valid_token_returns_200() {
    let app = TestApp::new().await;
    let token = app.register_user("me_test@example.com", "Password123").await;
 
    let resp = app.get_authed("/api/v1/auth/me", &token).await;
 
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["email"], "me_test@example.com");
}

#[tokio::test]
async fn test_me_without_token_returns_401() {
    let app = TestApp::new().await;
 
    let resp = app.client
        .get(format!("{}/api/v1/auth/me", app.base_url))
        .send()
        .await
        .unwrap();
 
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_me_with_invalid_token_returns_401() {
    let app = TestApp::new().await;
 
    let resp = app.get_authed("/api/v1/auth/me", "invalid.jwt.token").await;
 
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_TOKEN");
}

#[tokio::test]
async fn test_me_response_never_contains_password() {
    let app = TestApp::new().await;
    let token = app.register_user("nopw@example.com", "Password123").await;
 
    let resp = app.get_authed("/api/v1/auth/me", &token).await;
    let body_str = resp.text().await.unwrap();
 
    assert!(!body_str.contains("password"));
    assert!(!body_str.contains("$argon2"));
    assert!(!body_str.contains("Password123"));
}

#[tokio::test]
async fn test_response_includes_x_request_id_header() {
    let app = TestApp::new().await;
 
    let resp = app.post("/api/v1/auth/login", json!({
        "email": "anyone@example.com",
        "password": "Password123",
    })).await;
 
    assert!(
        resp.headers().contains_key("x-request-id"),
        "Response must include x-request-id header"
    );
}