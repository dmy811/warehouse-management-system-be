// use axum::{Json, body::Body, extract::{Request, State}, http::StatusCode, middleware::Next, response::Response};

// use crate::{infrastructure::security::{Claims, verify_jwt}, state::AppState, utils::response::{ErrorResponse, error_response}};

// pub async fn require_jwt(
//     State(state): State<AppState>,
//     mut req: Request<Body>, next: Next
// ) -> Result<Response, (StatusCode, Json<ErrorResponse>)>{
//     // cara yang biasa saja
//     // let auth_header = match req.headers().get(axum::http::header::AUTHORIZATION) {
//     //     Some(h) => match h.to_str() {
//     //         Ok(s) => s.to_string(),
//     //         Err(_) => return Err(error_response("Header Authorization invalid".to_string(), StatusCode::UNAUTHORIZED))
//     //     },
//     //     None => return Err(error_response("Authorization header missing".to_string(), StatusCode::UNAUTHORIZED))
//     // };

//     // let token = auth_header.strip_prefix("Bearer ").map(|s| s.trim());
//     // let token = match token {
//     //     Some(t) if !t.is_empty() => t,
//     //     _ => return Err(error_response("The token format must be Bearer <token>".to_string(), StatusCode::UNAUTHORIZED))
//     // };

//     // let claims: Claims = match verify_jwt(&state.jwt, token) {
//     //     Ok(c) => c,
//     //     Err(_) => return Err(error_response("Invalid or expired token".to_string(), StatusCode::UNAUTHORIZED))
//     // };

//     // cara yang lebih ringkas dengan menggunakan ? operator dan idiomatic Rust
//     let auth_header = req
//         .headers()
//         .get(axum::http::header::AUTHORIZATION)
//         .and_then(|h| h.to_str().ok())
//         .ok_or_else(|| {
//             error_response(
//             "Authorization header missing or invalid".to_string(),
//             StatusCode::UNAUTHORIZED,
//             )
//     })?;

//     // kalau gak pake ?, maka jadi harus manual seperti dibawah
//     // let auth_header = match value {
//     //     Ok(v) => v,
//     //     Err(e) => return Err(e),
//     // };

//     let token = auth_header
//         .strip_prefix("Bearer ")
//         .map(str::trim)
//         .filter(|t| !t.is_empty())
//         .ok_or_else(|| {
//             error_response(
//             "The token format must be Bearer <token>".to_string(),
//             StatusCode::UNAUTHORIZED,
//             )
//     })?;

//     let claims = verify_jwt(&state.jwt, token)
//         .map_err(|_| error_response(
//         "Invalid or expired token".to_string(),
//         StatusCode::UNAUTHORIZED,
//     ))?;

//     req.extensions_mut().insert(claims);

//     Ok(next.run(req).await)
// }