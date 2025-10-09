mod models;
mod database;
mod handlers;
mod file_manager;
mod scheduler;
mod i18n;
mod auth;
mod middleware;

use axum::{
    routing::{get, post, delete},
    Router,
    middleware::from_fn_with_state,
    http::{StatusCode, header, Uri},
    response::Response,
};
use tower_http::cors::CorsLayer;
use tower_sessions::{SessionManagerLayer, Expiry};
use time::Duration;
use tower_sessions_sqlx_store::SqliteStore;
use std::sync::Arc;
use tokio::spawn;
use tracing::info;
use tracing_subscriber;
use rust_embed::RustEmbed;

use database::create_pool;
use handlers::*;
use scheduler::TaskScheduler;
use auth::initialize_auth_table;
use middleware::{auth_middleware, page_auth_middleware};

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticAssets;

// Handler for embedded static assets
async fn static_handler(uri: Uri) -> Response<axum::body::Body> {
    let path = uri.path().trim_start_matches("/static/");

    match StaticAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(content.data.to_vec()))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("404 Not Found"))
            .unwrap(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting devicl file cleanup manager");

    // Initialize database
    let pool = create_pool().await?;
    info!("Database initialized");

    // Initialize authentication
    initialize_auth_table(&pool).await?;
    info!("Authentication initialized");

    // Start scheduler in background
    let scheduler_pool = Arc::new(pool.clone());
    spawn(async move {
        let scheduler = TaskScheduler::new(scheduler_pool);
        scheduler.start().await;
    });

    // Setup session store
    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24))); // 24 hours

    // Create router
    let app = Router::new()
        // Public routes (no authentication required)
        .route("/login", get(login_page))
        .route("/api/auth/login", post(api_login))
        .route("/api/auth/status", get(api_auth_status))
        .route("/api/messages", get(api_get_messages))
        .route("/api/languages", get(api_get_languages))

        // Protected routes (require authentication)
        .route("/", get(index))
        .route("/api/directories", get(api_get_directories))
        .route("/api/directories", post(api_add_directory))
        .route("/api/directories/:id", delete(api_delete_directory))
        .route("/api/delete-files", post(api_delete_files))
        .route("/api/tasks", get(api_get_tasks))
        .route("/api/tasks", post(api_create_task))
        .route("/api/tasks/:id", delete(api_delete_task))
        .route("/api/auth/logout", post(api_logout))
        .route("/api/auth/change-password", post(api_change_password))
        .layer(from_fn_with_state(pool.clone(), auth_middleware))

        // Static files (embedded)
        .route("/static/*path", get(static_handler))

        // Add middleware layers (order matters: inner layers run first)
        .layer(CorsLayer::permissive())
        .layer(from_fn_with_state(pool.clone(), page_auth_middleware))
        .layer(session_layer)
        .with_state(pool);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
