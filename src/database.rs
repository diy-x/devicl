use sqlx::{SqlitePool, Row};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use std::fs;
use crate::models::*;

pub async fn create_pool() -> Result<SqlitePool> {
    // Create database directory if it doesn't exist
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./devicl.db".to_string());

    // Create parent directory if needed
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Ensure the database file exists
    if !std::path::Path::new(&db_path).exists() {
        fs::File::create(&db_path)?;
    }

    let connection_string = format!("sqlite:{}?mode=rwc", db_path);
    let pool = SqlitePool::connect(&connection_string).await?;
    initialize_database(&pool).await?;
    Ok(pool)
}

pub async fn initialize_database(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS directories (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            description TEXT,
            created_at TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scheduled_tasks (
            id TEXT PRIMARY KEY,
            directory_id TEXT NOT NULL,
            name TEXT NOT NULL,
            cron_expression TEXT NOT NULL,
            task_type TEXT NOT NULL,
            filter_config TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            last_run TEXT,
            next_run TEXT,
            FOREIGN KEY (directory_id) REFERENCES directories (id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn add_directory(pool: &SqlitePool, req: &AddDirectoryRequest) -> Result<Directory> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let directory = Directory {
        id: id.clone(),
        path: req.path.clone(),
        description: req.description.clone(),
        created_at: now,
        enabled: true,
    };

    sqlx::query(
        "INSERT INTO directories (id, path, description, created_at, enabled) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.path)
    .bind(&req.description)
    .bind(now.to_rfc3339())
    .bind(true)
    .execute(pool)
    .await?;

    Ok(directory)
}

pub async fn get_directories(pool: &SqlitePool) -> Result<Vec<Directory>> {
    let rows = sqlx::query("SELECT * FROM directories ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;

    let mut directories = Vec::new();
    for row in rows {
        let created_at_str: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
            .with_timezone(&Utc);

        directories.push(Directory {
            id: row.get("id"),
            path: row.get("path"),
            description: row.get("description"),
            created_at,
            enabled: row.get("enabled"),
        });
    }

    Ok(directories)
}

pub async fn delete_directory(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM directories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_directory_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Directory>> {
    let row = sqlx::query("SELECT * FROM directories WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        let created_at_str: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
            .with_timezone(&Utc);

        Ok(Some(Directory {
            id: row.get("id"),
            path: row.get("path"),
            description: row.get("description"),
            created_at,
            enabled: row.get("enabled"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn create_scheduled_task(pool: &SqlitePool, req: &CreateTaskRequest) -> Result<ScheduledTask> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let filter_config_json = serde_json::to_string(&req.filter_config)?;

    let task = ScheduledTask {
        id: id.clone(),
        directory_id: req.directory_id.clone(),
        name: req.name.clone(),
        cron_expression: req.cron_expression.clone(),
        task_type: req.task_type.clone(),
        filter_config: filter_config_json.clone(),
        enabled: true,
        created_at: now,
        last_run: None,
        next_run: None,
    };

    sqlx::query(
        "INSERT INTO scheduled_tasks (id, directory_id, name, cron_expression, task_type, filter_config, enabled, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.directory_id)
    .bind(&req.name)
    .bind(&req.cron_expression)
    .bind(req.task_type.to_string())
    .bind(filter_config_json)
    .bind(true)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(task)
}

pub async fn get_scheduled_tasks(pool: &SqlitePool) -> Result<Vec<ScheduledTask>> {
    let rows = sqlx::query("SELECT * FROM scheduled_tasks ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;

    let mut tasks = Vec::new();
    for row in rows {
        let created_at_str: String = row.get("created_at");
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?
            .with_timezone(&Utc);

        let last_run = if let Some(last_run_str) = row.get::<Option<String>, _>("last_run") {
            Some(chrono::DateTime::parse_from_rfc3339(&last_run_str)?.with_timezone(&Utc))
        } else {
            None
        };

        let next_run = if let Some(next_run_str) = row.get::<Option<String>, _>("next_run") {
            Some(chrono::DateTime::parse_from_rfc3339(&next_run_str)?.with_timezone(&Utc))
        } else {
            None
        };

        let task_type_str: String = row.get("task_type");
        let task_type = TaskType::from_string(&task_type_str)
            .map_err(|e| anyhow::anyhow!(e))?;

        tasks.push(ScheduledTask {
            id: row.get("id"),
            directory_id: row.get("directory_id"),
            name: row.get("name"),
            cron_expression: row.get("cron_expression"),
            task_type,
            filter_config: row.get("filter_config"),
            enabled: row.get("enabled"),
            created_at,
            last_run,
            next_run,
        });
    }

    Ok(tasks)
}

pub async fn delete_scheduled_task(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM scheduled_tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_task_run_time(pool: &SqlitePool, id: &str, last_run: chrono::DateTime<Utc>, next_run: Option<chrono::DateTime<Utc>>) -> Result<()> {
    sqlx::query("UPDATE scheduled_tasks SET last_run = ?, next_run = ? WHERE id = ?")
        .bind(last_run.to_rfc3339())
        .bind(next_run.map(|t| t.to_rfc3339()))
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn initialize_default_directories(pool: &SqlitePool) -> Result<()> {
    let default_dirs = vec![
        (
            "/home/holomotion/.config/NTSports/HoloMotion/mp4s",
            Some("MP4 video files directory".to_string()),
        ),
        (
            "/home/holomotion/.config/NTSports/HoloMotion/results",
            Some("Results directory".to_string()),
        ),
    ];

    for (path, description) in default_dirs {
        // Check if directory already exists
        let existing = sqlx::query("SELECT id FROM directories WHERE path = ?")
            .bind(path)
            .fetch_optional(pool)
            .await?;

        if existing.is_none() {
            // Create directory if path exists
            if std::path::Path::new(path).exists() {
                let req = AddDirectoryRequest {
                    path: path.to_string(),
                    description,
                };

                match add_directory(pool, &req).await {
                    Ok(_) => {
                        tracing::info!("Initialized default directory: {}", path);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to initialize default directory {}: {}", path, e);
                    }
                }
            } else {
                tracing::info!("Skipping default directory {} (path does not exist)", path);
            }
        }
    }

    Ok(())
}