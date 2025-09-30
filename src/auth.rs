use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tower_sessions::Session;

const DEFAULT_PASSWORD: &str = "holomotion";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub id: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
}

pub async fn initialize_auth_table(pool: &SqlitePool) -> Result<()> {
    // Create admin_config table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS admin_config (
            id TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Check if admin config exists, if not create with default password
    let existing = sqlx::query("SELECT COUNT(*) as count FROM admin_config")
        .fetch_one(pool)
        .await?;

    let count: i64 = existing.get("count");
    if count == 0 {
        let id = Uuid::new_v4().to_string();
        let password_hash = hash_password(DEFAULT_PASSWORD)?;
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO admin_config (id, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&password_hash)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub fn hash_password(password: &str) -> Result<String> {
    Ok(hash(password, DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    Ok(verify(password, hash)?)
}

pub async fn get_admin_config(pool: &SqlitePool) -> Result<Option<AdminConfig>> {
    let row = sqlx::query("SELECT * FROM admin_config LIMIT 1")
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
            .with_timezone(&Utc);
        let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)?
            .with_timezone(&Utc);

        Ok(Some(AdminConfig {
            id: row.get("id"),
            password_hash: row.get("password_hash"),
            created_at,
            updated_at,
        }))
    } else {
        Ok(None)
    }
}

pub async fn verify_admin_password(pool: &SqlitePool, password: &str) -> Result<bool> {
    if let Some(config) = get_admin_config(pool).await? {
        Ok(verify_password(password, &config.password_hash)?)
    } else {
        Ok(false)
    }
}

pub async fn change_admin_password(pool: &SqlitePool, old_password: &str, new_password: &str) -> Result<bool> {
    if !verify_admin_password(pool, old_password).await? {
        return Ok(false);
    }

    let new_hash = hash_password(new_password)?;
    let now = Utc::now();

    let result = sqlx::query(
        "UPDATE admin_config SET password_hash = ?, updated_at = ?"
    )
    .bind(&new_hash)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

// Session management
pub async fn is_authenticated(session: &Session) -> bool {
    match session.get::<bool>("authenticated").await {
        Ok(Some(authenticated)) => authenticated,
        _ => false,
    }
}

pub async fn set_authenticated(session: &Session, authenticated: bool) -> Result<()> {
    session.insert("authenticated", authenticated).await?;
    Ok(())
}

pub async fn logout(session: &Session) -> Result<()> {
    session.flush().await?;
    Ok(())
}