use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use uuid::Uuid;

use super::jwt::verify_token;
use crate::{error::AppError, state::AppState};

/// Extractor that requires a valid `Authorization: Bearer <jwt>` header,
/// proving the request comes from a provider who has already completed the
/// wallet-signature challenge in `POST /auth/verify`.
pub struct AuthUser {
    pub provider_id: Uuid,
    /// Carried through from the JWT claims for handlers that need it
    /// without a DB round-trip (e.g. future payment-lookup routes).
    #[allow(dead_code)]
    pub stellar_public_key: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing authorization header".to_string()))?;

        let token = header_value.strip_prefix("Bearer ").ok_or_else(|| {
            AppError::Unauthorized("authorization header must use the Bearer scheme".to_string())
        })?;

        let claims = verify_token(&state.config.jwt_secret, token)?;

        Ok(AuthUser {
            provider_id: claims.sub,
            stellar_public_key: claims.stellar_public_key,
        })
    }
}
