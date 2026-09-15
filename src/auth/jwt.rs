use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: the provider's id.
    pub sub: Uuid,
    pub stellar_public_key: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn issue_token(
    secret: &str,
    provider_id: Uuid,
    stellar_public_key: &str,
    expires_in_seconds: i64,
) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: provider_id,
        stellar_public_key: stellar_public_key.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(expires_in_seconds)).timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| AppError::Internal(err.into()))
}

pub fn verify_token(secret: &str, token: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("invalid or expired token".to_string()))
}
