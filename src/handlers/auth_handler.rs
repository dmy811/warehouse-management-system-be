use axum::{Extension, Json, extract::State, response::IntoResponse};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::Value;
use time::Duration;
use validator::Validate;

use crate::{dtos::{AuthResponse, LoginRequest, UserResponse, auth_dto::UpdatePasswordRequest, user_dto::UpdateUserRequest}, errors::{AppError, AppResult}, middlewares::AuthUser, response::ApiResponse, state::AppState};

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
 
    let (auth_response, refresh_token): (AuthResponse, String) = state.services.auth.login(req).await?;
    let jar = build_refresh_cookie(&state, refresh_token);
 
    Ok((jar, ApiResponse::ok("Login successful", auth_response)))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar
) -> AppResult<impl IntoResponse> {
    let cookie_name=  &state.config.cookie.name;

    let refresh_token = jar
        .get(cookie_name)
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let (auth_response, new_refresh_token) = state.services.auth.refresh(&refresh_token).await?;
    let jar = build_refresh_cookie(&state, new_refresh_token);

    Ok((jar, ApiResponse::ok("Refresh token successful", auth_response)))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar
) -> AppResult<impl IntoResponse> {
    let cookie_name = &state.config.cookie.name;

    let refresh_token = jar
        .get(cookie_name)
        .map(|c| c.value().to_string())
        .unwrap_or_default();

    state.services.auth.logout(&refresh_token).await?;

    let cleared_cookie = Cookie::build((cookie_name.clone(), ""))
        .path(state.config.cookie.path.clone())
        .max_age(Duration::seconds(0))
        .http_only(true)
        .secure(state.config.cookie.secure)
        .build();
 
    let new_jar = jar.add(cleared_cookie);

    Ok((new_jar, ApiResponse::ok("Logout successful", Value::Null)))
}


pub async fn get_profile(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>
) -> AppResult<impl IntoResponse> {
    let user: UserResponse = state.services.auth.get_profile(auth_user.id).await?;

    Ok(ApiResponse::ok("User retrieved", user))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<impl IntoResponse> {
    req.validate_is_empty()?;
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: UserResponse = state.services.auth.update_profile(auth_user.id, req).await?;

    Ok(ApiResponse::ok("Update successfull", result))
}

pub async fn change_profile_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdatePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    state.services.auth.update_profile_password(auth_user.id, req).await?;

    Ok(ApiResponse::ok("Update password successfull", Value::Null))
}

fn build_refresh_cookie(state: &AppState, refresh_token: String) -> CookieJar {
    let cfg = &state.config.cookie;

    let same_site = match cfg.same_site.as_str() {
        "Strict" => SameSite::Strict,
        "Lax" => SameSite::Lax,
        "None" => SameSite::None,
        _ => SameSite::Strict
    };

    let mut cookie = Cookie::build((cfg.name.clone(), refresh_token))
        .path(cfg.path.clone())
        .max_age(Duration::seconds(cfg.max_age_seconds))
        .http_only(cfg.http_only)
        .secure(cfg.secure)
        .same_site(same_site);

    if let Some(ref domain) = cfg.domain {
        cookie = cookie.domain(domain.clone());
    }

    CookieJar::new().add(cookie.build())
    
}