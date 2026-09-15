use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Provider {
    pub id: Uuid,
    pub stellar_public_key: String,
    pub display_name: String,
    pub email: Option<String>,
    pub specialty: Option<String>,
    pub license_number: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
