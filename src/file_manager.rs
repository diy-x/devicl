use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use anyhow::Result;
use crate::models::*;

pub async fn delete_files_by_filter(
    directory_path: &str,
    task_type: &TaskType,
    filter_config: &FilterConfig,
    dry_run: bool,
) -> Result<DeleteFilesResponse> {
    let mut response = DeleteFilesResponse {
        files_found: Vec::new(),
        files_deleted: Vec::new(),
        errors: Vec::new(),
        total_files: 0,
        total_deleted: 0,
    };

    let dir_path = Path::new(directory_path);
    if !dir_path.exists() {
        response.errors.push(format!("Directory does not exist: {}", directory_path));
        return Ok(response);
    }

    if !dir_path.is_dir() {
        response.errors.push(format!("Path is not a directory: {}", directory_path));
        return Ok(response);
    }

    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            response.errors.push(format!("Failed to read directory: {}", e));
            return Ok(response);
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                response.errors.push(format!("Failed to read directory entry: {}", e));
                continue;
            }
        };

        let path = entry.path();
        if path.is_file() {
            let file_path_str = path.to_string_lossy().to_string();

            // Check file pattern filter
            if let Some(pattern) = &filter_config.file_pattern {
                if !path.file_name()
                    .map(|name| name.to_string_lossy().contains(pattern))
                    .unwrap_or(false) {
                    continue;
                }
            }

            // Get file metadata
            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(e) => {
                    response.errors.push(format!("Failed to get metadata for {}: {}", file_path_str, e));
                    continue;
                }
            };

            let modified_time = match metadata.modified() {
                Ok(time) => time,
                Err(e) => {
                    response.errors.push(format!("Failed to get modified time for {}: {}", file_path_str, e));
                    continue;
                }
            };

            let modified_datetime: DateTime<Utc> = modified_time.into();

            // Check if file matches the filter criteria
            let should_delete = match task_type {
                TaskType::DeleteOlder => {
                    if let Some(days_old) = filter_config.days_old {
                        let cutoff_date = Utc::now() - chrono::Duration::days(days_old as i64);
                        modified_datetime < cutoff_date
                    } else if let Some(date_from) = filter_config.date_from {
                        modified_datetime < date_from
                    } else {
                        false
                    }
                }
                TaskType::DeleteNewer => {
                    if let Some(days_old) = filter_config.days_old {
                        let cutoff_date = Utc::now() - chrono::Duration::days(days_old as i64);
                        modified_datetime > cutoff_date
                    } else if let Some(date_to) = filter_config.date_to {
                        modified_datetime > date_to
                    } else {
                        false
                    }
                }
                TaskType::DeleteBetween => {
                    let after_start = filter_config.date_from
                        .map(|start| modified_datetime >= start)
                        .unwrap_or(true);
                    let before_end = filter_config.date_to
                        .map(|end| modified_datetime <= end)
                        .unwrap_or(true);
                    after_start && before_end
                }
            };

            if should_delete {
                response.files_found.push(file_path_str.clone());
                response.total_files += 1;

                if !dry_run {
                    match fs::remove_file(&path) {
                        Ok(_) => {
                            response.files_deleted.push(file_path_str);
                            response.total_deleted += 1;
                        }
                        Err(e) => {
                            response.errors.push(format!("Failed to delete {}: {}", file_path_str, e));
                        }
                    }
                }
            }
        }
    }

    if dry_run {
        response.total_deleted = 0;
    }

    Ok(response)
}

pub fn validate_directory_path(path: &str) -> Result<()> {
    let dir_path = Path::new(path);

    if !dir_path.exists() {
        return Err(anyhow::anyhow!("Directory does not exist: {}", path));
    }

    if !dir_path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", path));
    }

    // Check if we can read the directory
    match fs::read_dir(dir_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Cannot read directory: {}", e)),
    }
}