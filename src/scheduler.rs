use std::sync::Arc;
use tokio::time::{sleep, Duration};
use sqlx::SqlitePool;
use chrono::Utc;
use anyhow::Result;
use cron::Schedule;
use std::str::FromStr;
use tracing::{info, error, warn};

use crate::models::*;
use crate::database::{get_scheduled_tasks, update_task_run_time, get_directory_by_id};
use crate::file_manager::delete_files_by_filter;

pub struct TaskScheduler {
    pool: Arc<SqlitePool>,
}

impl TaskScheduler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn start(&self) {
        info!("Starting task scheduler");

        loop {
            if let Err(e) = self.run_scheduled_tasks().await {
                error!("Error running scheduled tasks: {}", e);
            }

            // Check every minute
            sleep(Duration::from_secs(60)).await;
        }
    }

    async fn run_scheduled_tasks(&self) -> Result<()> {
        let tasks = get_scheduled_tasks(&self.pool).await?;
        let now = Utc::now();

        for task in tasks {
            if !task.enabled {
                continue;
            }

            // Parse cron expression
            let schedule = match Schedule::from_str(&task.cron_expression) {
                Ok(schedule) => schedule,
                Err(e) => {
                    error!("Invalid cron expression for task {}: {}", task.name, e);
                    continue;
                }
            };

            // Check if task should run
            let should_run = if let Some(last_run) = task.last_run {
                // Find next occurrence after last run
                let next_after_last = schedule.after(&last_run).next();
                if let Some(next_time) = next_after_last {
                    now >= next_time
                } else {
                    false
                }
            } else {
                // Never run before, check if we should run now
                let next_occurrence = schedule.upcoming(Utc).next();
                if let Some(next_time) = next_occurrence {
                    now >= next_time
                } else {
                    false
                }
            };

            if should_run {
                info!("Running scheduled task: {}", task.name);

                if let Err(e) = self.execute_task(&task).await {
                    error!("Failed to execute task {}: {}", task.name, e);
                } else {
                    // Update last run time and calculate next run
                    let next_run = schedule.upcoming(Utc).next();
                    if let Err(e) = update_task_run_time(&self.pool, &task.id, now, next_run).await {
                        error!("Failed to update task run time for {}: {}", task.name, e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn execute_task(&self, task: &ScheduledTask) -> Result<()> {
        // Get directory info
        let directory = get_directory_by_id(&self.pool, &task.directory_id).await?
            .ok_or_else(|| anyhow::anyhow!("Directory not found for task {}", task.name))?;

        if !directory.enabled {
            warn!("Skipping task {} because directory is disabled", task.name);
            return Ok(());
        }

        // Parse filter config
        let filter_config: FilterConfig = serde_json::from_str(&task.filter_config)?;

        // Execute file deletion
        let result = delete_files_by_filter(
            &directory.path,
            &task.task_type,
            &filter_config,
            false, // Not a dry run for scheduled tasks
        ).await?;

        info!(
            "Task {} completed: {} files found, {} files deleted, {} errors",
            task.name,
            result.total_files,
            result.total_deleted,
            result.errors.len()
        );

        if !result.errors.is_empty() {
            for error in &result.errors {
                error!("Task {} error: {}", task.name, error);
            }
        }

        Ok(())
    }
}

pub fn validate_cron_expression(cron_expr: &str) -> Result<()> {
    Schedule::from_str(cron_expr)
        .map_err(|e| anyhow::anyhow!("Invalid cron expression: {}", e))?;
    Ok(())
}