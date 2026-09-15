mod auth;
mod config;
mod error;
mod providers;
mod state;

use std::{sync::Arc, time::Duration};

use axum::{Json, Router, http::Method, routing::get};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use config::Config;
use state::AppStateInner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,healthtaki_backend=debug".into()),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!(network = %config.stellar_network, "configured Stellar network");

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let port = config.port;
    let cors_origin = config.cors_allowed_origin.clone();
    let state: state::AppState = Arc::new(AppStateInner { db, config });

    let cors = CorsLayer::new()
        .allow_origin(cors_origin.parse::<axum::http::HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .nest("/auth", auth::routes::router())
        .nest("/providers", providers::routes::router())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(15),
        ))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
