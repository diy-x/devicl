use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Directory {
    pub id: String,
    pub path: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledTask {
    pub id: String,
    pub directory_id: String,
    pub name: String,
    pub cron_expression: String,
    pub task_type: TaskType,
    pub filter_config: String, // JSON string storing filter configuration
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    DeleteOlder,
    DeleteNewer,
    DeleteBetween,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub days_old: Option<i32>,
    pub file_pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddDirectoryRequest {
    pub path: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub directory_id: String,
    pub name: String,
    pub cron_expression: String,
    pub task_type: TaskType,
    pub filter_config: FilterConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFilesRequest {
    pub directory_id: String,
    pub task_type: TaskType,
    pub filter_config: FilterConfig,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFilesResponse {
    pub files_found: Vec<String>,
    pub files_deleted: Vec<String>,
    pub errors: Vec<String>,
    pub total_files: usize,
    pub total_deleted: usize,
}

impl TaskType {
    pub fn to_string(&self) -> String {
        match self {
            TaskType::DeleteOlder => "delete_older".to_string(),
            TaskType::DeleteNewer => "delete_newer".to_string(),
            TaskType::DeleteBetween => "delete_between".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        match s {
            "delete_older" => Ok(TaskType::DeleteOlder),
            "delete_newer" => Ok(TaskType::DeleteNewer),
            "delete_between" => Ok(TaskType::DeleteBetween),
            _ => Err("Invalid task type"),
        }
    }
}