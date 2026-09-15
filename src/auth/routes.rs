use axum::{Json, Router, extract::State, routing::post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AppResult},
    providers::{model::Provider, repo as providers_repo},
    state::AppState,
};

use super::{challenge, jwt, stellar_sig};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/challenge", post(request_challenge))
        .route("/verify", post(verify_challenge))
}

#[derive(Debug, Deserialize)]
struct ChallengeRequest {
    stellar_public_key: String,
}

#[derive(Debug, Serialize)]
struct ChallengeResponse {
    /// Opaque token the client must echo back (unchanged) in `/verify`.
    nonce: String,
    /// Exact string to pass to Freighter's `signMessage`.
    message: String,
    expires_at: DateTime<Utc>,
}

/// Step 1 of wallet login: the client asks for a one-time message to sign
/// with Freighter (`signMessage`). No password, no custody of funds — the
/// signature is proof of control over the Stellar address.
async fn request_challenge(
    State(state): State<AppState>,
    Json(body): Json<ChallengeRequest>,
) -> AppResult<Json<ChallengeResponse>> {
    stellar_sig::validate_public_key_format(&body.stellar_public_key)
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let issued = challenge::issue_challenge(
        &state.db,
        &body.stellar_public_key,
        state.config.auth_challenge_ttl_seconds,
    )
    .await?;

    Ok(Json(ChallengeResponse {
        nonce: issued.nonce,
        message: issued.message,
        expires_at: issued.expires_at,
    }))
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    stellar_public_key: String,
    nonce: String,
    signature: String,
    /// Only used the first time this wallet authenticates, to seed the new
    /// provider profile with a friendly name.
    display_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerifyResponse {
    token: String,
    provider: Provider,
}

/// Step 2: verify the signed challenge and issue a JWT. The provider row is
/// created on first successful login (wallet-address-as-identity), so there
/// is no separate "sign up" step.
async fn verify_challenge(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> AppResult<Json<VerifyResponse>> {
    stellar_sig::validate_public_key_format(&body.stellar_public_key)
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let message = challenge::challenge_message(&body.nonce);
    stellar_sig::verify_signed_message(&body.stellar_public_key, &message, &body.signature)
        .map_err(|_| AppError::Unauthorized("signature verification failed".to_string()))?;

    challenge::consume_challenge(&state.db, &body.stellar_public_key, &body.nonce).await?;

    let provider = match providers_repo::find_by_public_key(&state.db, &body.stellar_public_key)
        .await?
    {
        Some(provider) => provider,
        None => {
            let display_name = body
                .display_name
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| short_address(&body.stellar_public_key));
            providers_repo::create(&state.db, &body.stellar_public_key, &display_name).await?
        }
    };

    let token = jwt::issue_token(
        &state.config.jwt_secret,
        provider.id,
        &provider.stellar_public_key,
        state.config.jwt_expires_in_seconds,
    )?;

    Ok(Json(VerifyResponse { token, provider }))
}

fn short_address(public_key: &str) -> String {
    if public_key.len() > 10 {
        format!(
            "{}…{}",
            &public_key[..5],
            &public_key[public_key.len() - 5..]
        )
    } else {
        public_key.to_string()
    }
}
