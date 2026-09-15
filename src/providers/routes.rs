use axum::{Json, Router, extract::State, routing::get};
use serde::Deserialize;
use validator::Validate;

use crate::{
    auth::extractor::AuthUser,
    error::{AppError, AppResult},
    state::AppState,
};

use super::{model::Provider, repo};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(get_me).patch(update_me))
}

async fn get_me(auth: AuthUser, State(state): State<AppState>) -> AppResult<Json<Provider>> {
    let provider = repo::find_by_id(&state.db, auth.provider_id)
        .await?
        .ok_or_else(|| AppError::NotFound("provider not found".to_string()))?;
    Ok(Json(provider))
}

#[derive(Debug, Deserialize, Validate)]
struct UpdateMeRequest {
    #[validate(length(min = 1, max = 200))]
    display_name: Option<String>,
    #[validate(email)]
    email: Option<String>,
    #[validate(length(max = 200))]
    specialty: Option<String>,
    #[validate(length(max = 100))]
    license_number: Option<String>,
}

async fn update_me(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<UpdateMeRequest>,
) -> AppResult<Json<Provider>> {
    body.validate()
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let provider = repo::update(
        &state.db,
        auth.provider_id,
        repo::ProviderUpdate {
            display_name: body.display_name,
            email: body.email,
            specialty: body.specialty,
            license_number: body.license_number,
        },
    )
    .await?;

    Ok(Json(provider))
}
