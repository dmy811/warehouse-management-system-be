// use axum::{Json, extract::State};
// use axum::http::StatusCode;
// use bcrypt::verify;
// use serde::{Deserialize, Serialize};
// use sqlx::Database;

// use crate::repositories::user_repository::create_user;
// use crate::state::AppState;
// use crate::{infrastructure::{db::DBPool, security::{JwtConfig, sign_jwt}}, repositories::user_repository::find_user_with_role_by_email, utils::response::{ApiResponse, ErrorResponse, error_response, success_response_with_message}};


// #[derive(Deserialize)]
// pub struct LoginRequest {
//     pub email: String, 
//     pub password: String
// }

// #[derive(Deserialize)]
// pub struct RegisterRequest {
//     pub name: String,
//     pub email: String, 
//     pub password: String,
//     pub photo: Option<String>,
//     pub phone: Option<String>
// }

// #[derive(Serialize)]
// pub struct UserInfo {
//     pub id: i64,
//     pub name: String,
//     pub email: String,
//     pub role: String
// }

// #[derive(Serialize)]
// pub struct DataLoginResponse {
//     pub token: String, 
//     pub user: UserInfo
// }

// // cara junior
// // pub async fn login(
// //     State(state): State<AppState>,
// //     Json(payload): Json<LoginRequest>
// // ) -> Result<(StatusCode, Json<ApiResponse<DataLoginResponse>>), (StatusCode, Json<ErrorResponse>)> {
// //     let found = match find_user_with_role_by_email(&state.pool, &payload.email).await {
// //         Ok(result) => result,
// //         Err(_) => return Err(error_response("Database error".to_string(), StatusCode::INTERNAL_SERVER_ERROR))
// //     };

// //     let (user, role) = match found {
// //         Some(tuple) => tuple,
// //         None => return Err(error_response("User not found".to_string(), StatusCode::NOT_FOUND))
// //     };

// //     if !verify(&payload.password, &user.password).unwrap_or(false) {
// //         return Err(error_response("Invalid credentials".to_string(), StatusCode::UNAUTHORIZED))
// //     }
// //     let role_name = role.unwrap_or_else(|| "keeper".to_string());

// //     let token = sign_jwt(&state.jwt, user.id, &user.name, &role_name)
// //         .map_err(|_| error_response("Token generation failed".to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

// //     let user_info = UserInfo {
// //         id: user.id,
// //         name: user.name,
// //         email: user.email,
// //         role: role_name
// //     };

// //     let data_response: DataLoginResponse = DataLoginResponse {
// //         token,
// //         user: user_info
// //     };

// //     Ok(success_response_with_message(data_response, "Login successfull".to_string()))
// // }

// // cara senior yeah
// pub async fn login(
//     State(state): State<AppState>,
//     Json(payload): Json<LoginRequest>
// ) -> Result<(StatusCode, Json<ApiResponse<DataLoginResponse>>), (StatusCode, Json<ErrorResponse>)> {

//     let found = find_user_with_role_by_email(&state.pool, &payload.email)
//         .await
//         .map_err(|_| error_response("Database error".to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

//     let (user, role) = found
//         .ok_or_else(|| error_response("User not found".to_string(), StatusCode::NOT_FOUND))?;

//     let is_valid = verify(&payload.password, &user.password)
//         .map_err(|_| error_response("Password verification failed".to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

//     if !is_valid {
//         return Err(error_response("Invalid credentials".to_string(), StatusCode::UNAUTHORIZED));
//     }

//     let role_name = role.unwrap_or_else(|| "keeper".to_string());

//     let token = sign_jwt(&state.jwt, user.id, &user.name, &role_name)
//         .map_err(|_| error_response("Token generation failed".to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

//     let user_info = UserInfo {
//         id: user.id,
//         name: user.name,
//         email: user.email,
//         role: role_name
//     };

//     let data_response = DataLoginResponse {
//         token,
//         user: user_info
//     };

//     Ok(success_response_with_message(data_response, "Login successfull".to_string()))
// }

// pub async fn register(
//     State(state): State<AppState>,
//     Json(payload): Json<RegisterRequest>
// ) -> Result<(StatusCode, Json<ApiResponse<UserInfo>>), (StatusCode, Json<ErrorResponse>)> {
//     let password_hash = bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST)
//         .map_err(|_| {
//             return error_response("Failed to hash password".to_string(), StatusCode::INTERNAL_SERVER_ERROR)
//         })?;

//     let payload = RegisterRequest {
//         password: password_hash,
//         ..payload
//     };

//     let user = create_user(&state.pool, &payload)
//         .await
//         .map_err(|e| {
//             if let sqlx::Error::Database(db_err) = &e {
//                 if db_err.is_unique_violation() {
//                     return error_response("Email already registered".to_string(), StatusCode::CONFLICT);
//                 }
//             }

//         error_response("Failed to create user".to_string(), StatusCode::INTERNAL_SERVER_ERROR)
//     })?;


//     let user_info = UserInfo {
//         id: user.id,
//         name: user.name,
//         email: user.email,
//         role: "keeper".to_string()
//     };

//     Ok(success_response_with_message(user_info, "User registered successfully".to_string()))
// }