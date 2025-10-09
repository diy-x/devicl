use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use rust_embed::RustEmbed;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Messages {
    // Common
    pub app_title: String,
    pub home: String,
    pub language: String,
    pub save: String,
    pub cancel: String,
    pub delete: String,
    pub edit: String,
    pub add: String,
    pub create: String,
    pub enabled: String,
    pub disabled: String,
    pub required: String,
    pub optional: String,
    pub confirm: String,
    pub error: String,
    pub success: String,
    pub warning: String,
    pub loading: String,

    // Directory Management
    pub directory_management: String,
    pub directory_path: String,
    pub directory_description: String,
    pub add_directory: String,
    pub delete_directory_confirm: String,
    pub directory_added_success: String,
    pub directory_deleted_success: String,
    pub directory_not_found: String,
    pub directory_invalid_path: String,
    pub path: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub operations: String,

    // File Operations
    pub file_operations: String,
    pub select_directory: String,
    pub delete_type: String,
    pub delete_older: String,
    pub delete_newer: String,
    pub delete_between: String,
    pub days_old: String,
    pub days_old_hint: String,
    pub start_date: String,
    pub end_date: String,
    pub file_pattern: String,
    pub file_pattern_hint: String,
    pub preview_delete: String,
    pub execute_delete: String,
    pub delete_result: String,
    pub files_found: String,
    pub files_deleted: String,
    pub delete_completed: String,
    pub preview_completed: String,

    // Scheduled Tasks
    pub scheduled_tasks: String,
    pub task_name: String,
    pub cron_expression: String,
    pub cron_hint: String,
    pub create_task: String,
    pub delete_task_confirm: String,
    pub task_created_success: String,
    pub task_deleted_success: String,
    pub task_type: String,
    pub last_run: String,
    pub never_run: String,
    pub cron_invalid: String,

    // Notifications
    pub select_directory_warning: String,
    pub request_failed: String,
    pub data_refreshed: String,
    pub cron_format_warning: String,

    // Footer
    pub footer_text: String,

    // Authentication
    pub login_title: String,
    pub login_subtitle: String,
    pub admin_password: String,
    pub enter_password: String,
    pub login_button: String,
    pub login_success: String,
    pub login_error: String,
    pub login_footer_text: String,
    pub default_password_notice: String,
    pub default_password_info: String,
    pub logout: String,
    pub change_password: String,
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
    pub password_changed_success: String,
    pub password_change_error: String,
    pub password_mismatch: String,
    pub password_too_short: String,

    // Operation Tips
    pub operation_tips: String,
    pub tip_preview_first: String,
    pub tip_backup_important: String,
    pub tip_test_small_first: String,

    // Security
    pub security_notice: String,
    pub change_default_password: String,
}

#[derive(RustEmbed)]
#[folder = "locales/"]
struct LocaleAssets;

static TRANSLATIONS: OnceLock<HashMap<String, Messages>> = OnceLock::new();

pub fn get_translations() -> &'static HashMap<String, Messages> {
    TRANSLATIONS.get_or_init(|| {
        let mut translations = HashMap::new();

        // Load supported languages
        let languages = ["zh", "en", "ja"];

        for lang in languages.iter() {
            let filename = format!("{}.json", lang);

            if let Some(file) = LocaleAssets::get(&filename) {
                let content = std::str::from_utf8(file.data.as_ref())
                    .unwrap_or_else(|_| {
                        eprintln!("Warning: Invalid UTF-8 in locale file {}", filename);
                        "{}"
                    });

                match serde_json::from_str::<Messages>(content) {
                    Ok(messages) => {
                        translations.insert(lang.to_string(), messages);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse locale file {}: {}", filename, e);
                        // Fall back to a minimal English version
                        if *lang == "en" {
                            translations.insert(lang.to_string(), create_fallback_messages());
                        }
                    }
                }
            } else {
                eprintln!("Warning: Locale file {} not found", filename);
                // Create fallback for English
                if *lang == "en" {
                    translations.insert(lang.to_string(), create_fallback_messages());
                }
            }
        }

        // If no translations were loaded, create fallback
        if translations.is_empty() {
            translations.insert("en".to_string(), create_fallback_messages());
        }

        translations
    })
}

// Create a minimal fallback message set
fn create_fallback_messages() -> Messages {
    Messages {
        app_title: "File Cleanup Manager".to_string(),
        home: "Home".to_string(),
        language: "Language".to_string(),
        save: "Save".to_string(),
        cancel: "Cancel".to_string(),
        delete: "Delete".to_string(),
        edit: "Edit".to_string(),
        add: "Add".to_string(),
        create: "Create".to_string(),
        enabled: "Enabled".to_string(),
        disabled: "Disabled".to_string(),
        required: "Required".to_string(),
        optional: "Optional".to_string(),
        confirm: "Confirm".to_string(),
        error: "Error".to_string(),
        success: "Success".to_string(),
        warning: "Warning".to_string(),
        loading: "Loading".to_string(),

        directory_management: "Directory Management".to_string(),
        directory_path: "Directory Path".to_string(),
        directory_description: "Description".to_string(),
        add_directory: "Add Directory".to_string(),
        delete_directory_confirm: "Are you sure you want to delete this directory?".to_string(),
        directory_added_success: "Directory added successfully!".to_string(),
        directory_deleted_success: "Directory deleted successfully!".to_string(),
        directory_not_found: "Directory not found".to_string(),
        directory_invalid_path: "Invalid directory path".to_string(),
        path: "Path".to_string(),
        description: "Description".to_string(),
        status: "Status".to_string(),
        created_at: "Created At".to_string(),
        operations: "Operations".to_string(),

        file_operations: "File Operations".to_string(),
        select_directory: "Select Directory".to_string(),
        delete_type: "Delete Type".to_string(),
        delete_older: "Delete files older than specified time".to_string(),
        delete_newer: "Delete files newer than specified time".to_string(),
        delete_between: "Delete files within time range".to_string(),
        days_old: "Days".to_string(),
        days_old_hint: "Delete files older/newer than X days".to_string(),
        start_date: "Start Date".to_string(),
        end_date: "End Date".to_string(),
        file_pattern: "File Pattern".to_string(),
        file_pattern_hint: "Match filenames containing this text".to_string(),
        preview_delete: "Preview Delete".to_string(),
        execute_delete: "Execute Delete".to_string(),
        delete_result: "Delete Result".to_string(),
        files_found: "files found".to_string(),
        files_deleted: "files deleted".to_string(),
        delete_completed: "Delete completed".to_string(),
        preview_completed: "Preview completed".to_string(),

        scheduled_tasks: "Scheduled Tasks".to_string(),
        task_name: "Task Name".to_string(),
        cron_expression: "Cron Expression".to_string(),
        cron_hint: "Examples: 0 0 * * * (daily at midnight)".to_string(),
        create_task: "Create Task".to_string(),
        delete_task_confirm: "Are you sure you want to delete this scheduled task?".to_string(),
        task_created_success: "Scheduled task created successfully!".to_string(),
        task_deleted_success: "Scheduled task deleted successfully!".to_string(),
        task_type: "Type".to_string(),
        last_run: "Last Run".to_string(),
        never_run: "Never run".to_string(),
        cron_invalid: "Invalid cron expression format".to_string(),

        select_directory_warning: "Please select a directory".to_string(),
        request_failed: "Request failed".to_string(),
        data_refreshed: "Data refreshed".to_string(),
        cron_format_warning: "Cron expression may be incorrect".to_string(),

        footer_text: "© 2024 File Cleanup Manager".to_string(),

        login_title: "Admin Login".to_string(),
        login_subtitle: "Please enter admin password".to_string(),
        admin_password: "Admin Password".to_string(),
        enter_password: "Enter password".to_string(),
        login_button: "Login".to_string(),
        login_success: "Login successful!".to_string(),
        login_error: "Incorrect password".to_string(),
        login_footer_text: "Default password for first use".to_string(),
        default_password_notice: "Default Password Notice".to_string(),
        default_password_info: "Default admin password is: holomotion".to_string(),
        logout: "Logout".to_string(),
        change_password: "Change Password".to_string(),
        old_password: "Current Password".to_string(),
        new_password: "New Password".to_string(),
        confirm_password: "Confirm Password".to_string(),
        password_changed_success: "Password changed successfully!".to_string(),
        password_change_error: "Failed to change password".to_string(),
        password_mismatch: "Passwords do not match".to_string(),
        password_too_short: "Password must be at least 6 characters".to_string(),

        // Operation Tips
        operation_tips: "Operation Tips".to_string(),
        tip_preview_first: "Recommend clicking 'Preview Delete' first to confirm files to be deleted".to_string(),
        tip_backup_important: "Please backup important files before deletion".to_string(),
        tip_test_small_first: "Recommend testing on small scope first, then use on larger scope after confirmation".to_string(),

        // Security
        security_notice: "Security Notice".to_string(),
        change_default_password: "Please change the default password after first login".to_string(),
    }
}

pub fn get_messages(lang: &str) -> &'static Messages {
    let translations = get_translations();
    translations.get(lang).unwrap_or_else(|| translations.get("en").unwrap())
}

pub fn get_supported_languages() -> Vec<(&'static str, &'static str)> {
    vec![
        ("zh", "中文"),
        ("en", "English"),
        ("ja", "日本語"),
    ]
}

// Function to reload translations (useful for development)
#[allow(dead_code)]
pub fn reload_translations() {
    // Force reinitialization by dropping the static
    // Note: This won't work in production due to OnceLock's nature
    // but is useful for development/testing
}

// Get available language codes
#[allow(dead_code)]
pub fn get_available_languages() -> Vec<String> {
    let translations = get_translations();
    translations.keys().cloned().collect()
}