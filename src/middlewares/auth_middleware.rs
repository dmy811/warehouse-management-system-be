use axum::{extract::{State}, http::{header, Request}, middleware::Next, response::Response};

use crate::{errors::AppError, state::AppState, utils::jwt::verify_token};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: i64,
    pub role: String
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next
) -> Result<Response, AppError> {
    let token = extract_bearer_token(&req)?;
    let claims = verify_token(token, &state.config.jwt_secret)?;

    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::InvalidToken)?;

    req.extensions_mut().insert(AuthUser {
        id: user_id,
        role: claims.role
    });
    
    Ok(next.run(req).await)
}

fn extract_bearer_token(req: &Request<axum::body::Body>) -> Result<&str, AppError>{
    let header_value = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Unauthorized)?;

    header_value
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)
}

pub fn require_roles(
    allowed: &'static [&'static str]
) -> impl Fn(AuthUser) -> Result<AuthUser, AppError> + Clone {
    move |user: AuthUser| {
        if allowed.contains(&user.role.as_str()){
            Ok(user)
        } else {
            Err(AppError::Forbidden)
        }
    }
}

// let guard = require_roles(&["ADMIN"]);

// let user = AuthUser {
//     role: "STAFF".to_string()
// };

// let result = guard(user);

