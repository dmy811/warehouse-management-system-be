pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>
) -> AppResult<impl IntoResponse> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let result: AuthResponse = state.services.auth.register(req).await?;

    Ok(ApiResponse::created("Registration successful", result))
}