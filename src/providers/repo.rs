use sqlx::PgPool;
use uuid::Uuid;

use super::model::Provider;
use crate::error::{AppError, AppResult};

pub async fn find_by_public_key(db: &PgPool, public_key: &str) -> AppResult<Option<Provider>> {
    let provider = sqlx::query_as::<_, Provider>(
        r#"
        select id, stellar_public_key, display_name, email, specialty, license_number, created_at, updated_at
        from providers
        where stellar_public_key = $1
        "#,
    )
    .bind(public_key)
    .fetch_optional(db)
    .await?;
    Ok(provider)
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> AppResult<Option<Provider>> {
    let provider = sqlx::query_as::<_, Provider>(
        r#"
        select id, stellar_public_key, display_name, email, specialty, license_number, created_at, updated_at
        from providers
        where id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;
    Ok(provider)
}

pub async fn create(db: &PgPool, public_key: &str, display_name: &str) -> AppResult<Provider> {
    sqlx::query_as::<_, Provider>(
        r#"
        insert into providers (stellar_public_key, display_name)
        values ($1, $2)
        returning id, stellar_public_key, display_name, email, specialty, license_number, created_at, updated_at
        "#,
    )
    .bind(public_key)
    .bind(display_name)
    .fetch_one(db)
    .await
    .map_err(map_unique_violation)
}

#[derive(Default)]
pub struct ProviderUpdate {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub specialty: Option<String>,
    pub license_number: Option<String>,
}

pub async fn update(db: &PgPool, id: Uuid, patch: ProviderUpdate) -> AppResult<Provider> {
    sqlx::query_as::<_, Provider>(
        r#"
        update providers set
            display_name = coalesce($2, display_name),
            email = coalesce($3, email),
            specialty = coalesce($4, specialty),
            license_number = coalesce($5, license_number),
            updated_at = now()
        where id = $1
        returning id, stellar_public_key, display_name, email, specialty, license_number, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(patch.display_name)
    .bind(patch.email)
    .bind(patch.specialty)
    .bind(patch.license_number)
    .fetch_one(db)
    .await
    .map_err(map_unique_violation)
}

fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.code().as_deref() == Some("23505")
    {
        return AppError::Conflict("that email is already in use".to_string());
    }
    AppError::Database(err)
}
