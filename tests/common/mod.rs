use std::net::SocketAddr;

use reqwest::{Client, Response};
use serde_json::{Value, json};
use sqlx::PgPool;

use tokio::net::TcpListener;
use warehouse_management_system_backend::infrastructure::config::{Config, CloudinaryConfig, AppEnv};
use warehouse_management_system_backend::app;

pub struct TestApp {
    pub base_url: String,
    pub client: Client,
    pub db: PgPool
}

impl TestApp {
    pub async fn new() -> Self {
        dotenvy::from_filename(".env.test").ok();

        let db_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for integration tests");

        let pool = setup_test_db(&db_url).await;

        let config = Config {
            database_url: db_url,
            jwt_secret: "test_secret_key".to_string(),
            jwt_expires_in_secs: 3600,
            app_env: AppEnv::Development,
            cloudinary: CloudinaryConfig {
                cloud_name: "test".to_string(),
                api_key: "test".to_string(),
                api_secret: "test".to_string(),
            }
        };

        let app = app::build_with_pool(pool.clone(), config).await;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Self {
            base_url,
            client: Client::new(),
            db: pool
        }

    }

    pub async fn post(&self, path: &str, body: Value) -> Response {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .json(&body)
            .send()
            .await
            .expect("Failed to send POST request")
    }

     pub async fn post_authed(&self, path: &str, body: Value, token: &str) -> Response {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .expect("Failed to send POST request")
    }

    pub async fn get_authed(&self, path: &str, token: &str) -> Response {
        self.client
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to send GET request")
    }

    pub async fn patch_authed(&self, path: &str, body: Value, token: &str) -> Response {
        self.client
            .patch(format!("{}{}", self.base_url, path))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .expect("Failed to send PATCH request")
    }

    pub async fn delete_authed(&self, path: &str, token: &str) -> Response {
        self.client
            .delete(format!("{}{}", self.base_url, path))
            .bearer_auth(token)
            .send()
            .await
            .expect("Failed to send DELETE request")
    }



    pub async fn register_user(&self, email: &str, password: &str) -> String {
        let resp = self.post(
            "/api/v1/auth/register",
            json!({
                "name": "Test User",
                "email": email,
                "password": password,
            }),
        ).await;
 
        assert_eq!(resp.status(), 201, "Registration failed");
        let body: Value = resp.json().await.unwrap();
        body["data"]["access_token"]
            .as_str()
            .expect("No access_token in response")
            .to_string()
    }

    pub async fn assign_role(&self, user_id: i64, role_name: &str) {
        sqlx::query!(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            SELECT $1, r.id FROM roles r WHERE r.name = $2
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#,
            user_id,
            role_name
        )
        .execute(&self.db)
        .await
        .expect("Failed to assign role");
    }

    pub async fn get_user_id(&self, token: &str) -> i64 {
        let resp = self.get_authed("/api/v1/auth/me", token).await;
        let body: Value = resp.json().await.unwrap();
        body["data"]["id"].as_i64().unwrap()
    }

    pub async fn create_warehouse(&self, name: &str, token: &str) -> i64 {
        let resp = self.post_authed(
            "/api/v1/warehouses",
            json!({
                "name": name,
                "address": "Jl. Test No. 1, Jakarta",
            }),
            token,
        ).await;
 
        assert_eq!(resp.status(), 201, "Warehouse creation failed");
        let body: Value = resp.json().await.unwrap();
        body["data"]["id"].as_i64().unwrap()
    }


}

async fn setup_test_db(db_url: &str) -> PgPool {
    let pool = PgPool::connect(db_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");

    pool
}