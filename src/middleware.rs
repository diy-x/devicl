use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use tower_sessions::Session;
use serde_json::json;

use crate::auth::is_authenticated;

// Authentication middleware for API routes
pub async fn auth_middleware(
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let uri = request.uri().path();

    // Allow public routes
    if is_public_route(uri) {
        return next.run(request).await;
    }

    // Check authentication for protected routes
    if !is_authenticated(&session).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Authentication required",
                "code": "UNAUTHORIZED"
            }))
        ).into_response();
    }

    next.run(request).await
}

// Check if a route is public (doesn't require authentication)
fn is_public_route(path: &str) -> bool {
    let public_routes = [
        "/",
        "/login",
        "/api/messages",
        "/api/languages",
        "/api/auth/login",
        "/api/auth/status",
        "/static",
    ];

    // Check exact matches and prefix matches for static files
    public_routes.iter().any(|route| {
        path == *route || (route == &"/static" && path.starts_with("/static/"))
    })
}

// Middleware to ensure user is authenticated for page access
pub async fn page_auth_middleware(
    session: Session,
    request: Request,
    next: Next,
) -> Response {
    let uri = request.uri().path();

    // If it's the login page or static resources, allow access
    if uri == "/login" || uri.starts_with("/static/") || uri.starts_with("/api/") {
        return next.run(request).await;
    }

    // For the main page, check authentication
    if uri == "/" && !is_authenticated(&session).await {
        // Redirect to login page
        return axum::response::Redirect::to("/login").into_response();
    }

    next.run(request).await
}