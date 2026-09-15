use chrono::{DateTime, Duration, Utc};
use rand::RngExt;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Build the exact human-readable string the wallet is asked to sign. The
/// backend reconstructs this same string when verifying, so only the nonce
/// needs to be persisted.
pub fn challenge_message(nonce: &str) -> String {
    format!(
        "HealthTaki authentication request\n\
         Nonce: {nonce}\n\
         This request will not trigger a blockchain transaction or cost any fees."
    )
}

fn generate_nonce() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    hex::encode(bytes)
}

pub struct IssuedChallenge {
    pub nonce: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn issue_challenge(
    db: &PgPool,
    stellar_public_key: &str,
    ttl_seconds: i64,
) -> AppResult<IssuedChallenge> {
    let nonce = generate_nonce();
    let expires_at = Utc::now() + Duration::seconds(ttl_seconds);

    sqlx::query(
        r#"
        insert into auth_challenges (stellar_public_key, nonce, expires_at)
        values ($1, $2, $3)
        "#,
    )
    .bind(stellar_public_key)
    .bind(&nonce)
    .bind(expires_at)
    .execute(db)
    .await?;

    Ok(IssuedChallenge {
        message: challenge_message(&nonce),
        nonce,
        expires_at,
    })
}

/// Atomically consume a challenge: it must exist, match the claimed public
/// key, not be expired, and not already have been used. Marking it consumed
/// in the same statement that reads it prevents a signature from being
/// replayed by issuing two concurrent verify requests.
pub async fn consume_challenge(
    db: &PgPool,
    stellar_public_key: &str,
    nonce: &str,
) -> AppResult<()> {
    let result = sqlx::query(
        r#"
        update auth_challenges
        set consumed_at = now()
        where nonce = $1
          and stellar_public_key = $2
          and consumed_at is null
          and expires_at > now()
        "#,
    )
    .bind(nonce)
    .bind(stellar_public_key)
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Unauthorized(
            "challenge is missing, expired, or already used".to_string(),
        ));
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn delete_expired(db: &PgPool) -> AppResult<u64> {
    let result = sqlx::query("delete from auth_challenges where expires_at < now() - interval '1 day'")
        .execute(db)
        .await?;
    Ok(result.rows_affected())
}

#[allow(dead_code)]
pub fn new_id() -> Uuid {
    Uuid::new_v4()
}
