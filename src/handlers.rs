use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use tera::{Tera, Context};
use sqlx::SqlitePool;
use std::sync::OnceLock;
use serde::Deserialize;
use rust_embed::RustEmbed;

use crate::models::*;
use crate::database::*;
use crate::file_manager::{delete_files_by_filter, validate_directory_path};
use crate::scheduler::validate_cron_expression;
use crate::i18n::{get_messages, get_supported_languages};
use crate::auth::{verify_admin_password, change_admin_password, is_authenticated, set_authenticated, logout as auth_logout, LoginRequest, ChangePasswordRequest, AuthResponse};
use tower_sessions::Session;

pub type AppState = SqlitePool;

#[derive(Deserialize)]
pub struct LanguageQuery {
    lang: Option<String>,
}

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

static TERA: OnceLock<Tera> = OnceLock::new();

fn get_tera() -> &'static Tera {
    TERA.get_or_init(|| {
        let mut tera = Tera::default();

        // Load all embedded template files
        for file_path in Templates::iter() {
            if let Some(content) = Templates::get(&file_path) {
                if let Ok(template_str) = std::str::from_utf8(content.data.as_ref()) {
                    if let Err(e) = tera.add_raw_template(&file_path, template_str) {
                        eprintln!("Failed to add template '{}': {}", file_path, e);
                    }
                }
            }
        }

        tera
    })
}

// Login page
pub async fn login_page(
    Query(params): Query<LanguageQuery>,
) -> impl IntoResponse {
    let tera = get_tera();
    let mut context = Context::new();

    // Get language (default to 'zh' for Chinese)
    let lang = params.lang.as_deref().unwrap_or("zh");
    let messages = get_messages(lang);
    let supported_languages: Vec<serde_json::Value> = get_supported_languages()
        .into_iter()
        .map(|(code, name)| serde_json::json!({
            "code": code,
            "name": name
        }))
        .collect();

    context.insert("messages", messages);
    context.insert("current_lang", &lang);
    context.insert("supported_languages", &supported_languages);

    match tera.render("login.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            eprintln!("Template render error: {}", e);
            eprintln!("Error details: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
        }
    }
}

// Homepage
pub async fn index(
    State(pool): State<AppState>,
    Query(params): Query<LanguageQuery>,
    _session: Session,
) -> impl IntoResponse {
    let tera = get_tera();
    let mut context = Context::new();

    // Get language (default to 'zh' for Chinese)
    let lang = params.lang.as_deref().unwrap_or("zh");
    let messages = get_messages(lang);
    let supported_languages: Vec<serde_json::Value> = get_supported_languages()
        .into_iter()
        .map(|(code, name)| serde_json::json!({
            "code": code,
            "name": name
        }))
        .collect();

    context.insert("messages", messages);
    context.insert("current_lang", &lang);
    context.insert("supported_languages", &supported_languages);

    match get_directories(&pool).await {
        Ok(directories) => {
            context.insert("directories", &directories);
            match get_scheduled_tasks(&pool).await {
                Ok(tasks) => {
                    context.insert("tasks", &tasks);
                    match tera.render("index.html", &context) {
                        Ok(html) => Html(html).into_response(),
                        Err(e) => {
                            eprintln!("Template render error: {}", e);
            eprintln!("Error details: {:?}", e);
                            fallback_index_html(&directories, &tasks, lang).into_response()
                        }
                    }
                }
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load tasks").into_response(),
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load directories").into_response(),
    }
}

// API endpoints
pub async fn api_add_directory(
    State(pool): State<AppState>,
    Json(req): Json<AddDirectoryRequest>,
) -> impl IntoResponse {
    // Validate directory path
    if let Err(e) = validate_directory_path(&req.path) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response();
    }

    match add_directory(&pool, &req).await {
        Ok(directory) => (StatusCode::CREATED, Json(directory)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to add directory"
        }))).into_response(),
    }
}

pub async fn api_get_directories(State(pool): State<AppState>) -> impl IntoResponse {
    match get_directories(&pool).await {
        Ok(directories) => Json(directories).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to get directories"
        }))).into_response(),
    }
}

pub async fn api_delete_directory(
    State(pool): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match delete_directory(&pool, &id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to delete directory"
        }))).into_response(),
    }
}

pub async fn api_delete_files(
    State(pool): State<AppState>,
    Json(req): Json<DeleteFilesRequest>,
) -> impl IntoResponse {
    // Get directory info
    let directory = match get_directory_by_id(&pool, &req.directory_id).await {
        Ok(Some(dir)) => dir,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Directory not found"
        }))).into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to get directory"
        }))).into_response(),
    };

    match delete_files_by_filter(&directory.path, &req.task_type, &req.filter_config, req.dry_run).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn api_create_task(
    State(pool): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    // Validate cron expression
    if let Err(e) = validate_cron_expression(&req.cron_expression) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response();
    }

    // Validate directory exists
    match get_directory_by_id(&pool, &req.directory_id).await {
        Ok(Some(_)) => {},
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Directory not found"
        }))).into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to validate directory"
        }))).into_response(),
    }

    match create_scheduled_task(&pool, &req).await {
        Ok(task) => (StatusCode::CREATED, Json(task)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to create task"
        }))).into_response(),
    }
}

pub async fn api_get_tasks(State(pool): State<AppState>) -> impl IntoResponse {
    match get_scheduled_tasks(&pool).await {
        Ok(tasks) => Json(tasks).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to get tasks"
        }))).into_response(),
    }
}

pub async fn api_delete_task(
    State(pool): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match delete_scheduled_task(&pool, &id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to delete task"
        }))).into_response(),
    }
}

// API endpoint to get language messages
pub async fn api_get_messages(Query(params): Query<LanguageQuery>) -> impl IntoResponse {
    let lang = params.lang.as_deref().unwrap_or("zh");
    let messages = get_messages(lang);
    Json(messages).into_response()
}

// API endpoint to get supported languages
pub async fn api_get_languages() -> impl IntoResponse {
    let languages = get_supported_languages();
    Json(languages).into_response()
}

// Authentication API endpoints
pub async fn api_login(
    State(pool): State<AppState>,
    session: Session,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match verify_admin_password(&pool, &req.password).await {
        Ok(true) => {
            if let Err(e) = set_authenticated(&session, true).await {
                eprintln!("Failed to set session: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse {
                    success: false,
                    message: "Session error".to_string(),
                })).into_response();
            }

            Json(AuthResponse {
                success: true,
                message: "Login successful".to_string(),
            }).into_response()
        }
        Ok(false) => {
            (StatusCode::UNAUTHORIZED, Json(AuthResponse {
                success: false,
                message: "Invalid password".to_string(),
            })).into_response()
        }
        Err(e) => {
            eprintln!("Login error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse {
                success: false,
                message: "Internal error".to_string(),
            })).into_response()
        }
    }
}

pub async fn api_logout(session: Session) -> impl IntoResponse {
    match auth_logout(&session).await {
        Ok(_) => Json(AuthResponse {
            success: true,
            message: "Logged out successfully".to_string(),
        }).into_response(),
        Err(e) => {
            eprintln!("Logout error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse {
                success: false,
                message: "Logout failed".to_string(),
            })).into_response()
        }
    }
}

pub async fn api_auth_status(session: Session) -> impl IntoResponse {
    Json(serde_json::json!({
        "authenticated": is_authenticated(&session).await
    })).into_response()
}

pub async fn api_change_password(
    State(pool): State<AppState>,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    if req.new_password.len() < 6 {
        return (StatusCode::BAD_REQUEST, Json(AuthResponse {
            success: false,
            message: "Password must be at least 6 characters".to_string(),
        })).into_response();
    }

    match change_admin_password(&pool, &req.old_password, &req.new_password).await {
        Ok(true) => Json(AuthResponse {
            success: true,
            message: "Password changed successfully".to_string(),
        }).into_response(),
        Ok(false) => (StatusCode::UNAUTHORIZED, Json(AuthResponse {
            success: false,
            message: "Current password is incorrect".to_string(),
        })).into_response(),
        Err(e) => {
            eprintln!("Password change error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse {
                success: false,
                message: "Failed to change password".to_string(),
            })).into_response()
        }
    }
}

// Fallback HTML when template engine fails
fn fallback_index_html(directories: &[Directory], tasks: &[ScheduledTask], lang: &str) -> Html<String> {
    let _messages = get_messages(lang);
    let directories_html = directories.iter()
        .map(|d| format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>
                    <button onclick="deleteDirectory('{}')" class="button-danger">删除</button>
                </td>
            </tr>"#,
            d.path,
            d.description.as_deref().unwrap_or(""),
            if d.enabled { "启用" } else { "禁用" },
            d.id
        ))
        .collect::<Vec<_>>()
        .join("\n");

    let tasks_html = tasks.iter()
        .map(|t| format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>
                    <button onclick="deleteTask('{}')" class="button-danger">删除</button>
                </td>
            </tr>"#,
            t.name,
            t.cron_expression,
            t.task_type.to_string(),
            if t.enabled { "启用" } else { "禁用" },
            t.id
        ))
        .collect::<Vec<_>>()
        .join("\n");

    Html(format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>文件清理管理器</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css">
</head>
<body>
    <main class="container">
        <h1>文件清理管理器</h1>

        <section>
            <h2>目录管理</h2>
            <form id="addDirectoryForm">
                <div class="grid">
                    <div>
                        <label for="path">目录路径</label>
                        <input type="text" id="path" name="path" required>
                    </div>
                    <div>
                        <label for="description">描述</label>
                        <input type="text" id="description" name="description">
                    </div>
                    <div>
                        <label>&nbsp;</label>
                        <button type="submit">添加目录</button>
                    </div>
                </div>
            </form>

            <table>
                <thead>
                    <tr>
                        <th>路径</th>
                        <th>描述</th>
                        <th>状态</th>
                        <th>操作</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </section>

        <section>
            <h2>文件删除</h2>
            <form id="deleteFilesForm">
                <div class="grid">
                    <div>
                        <label for="directorySelect">选择目录</label>
                        <select id="directorySelect" name="directory_id" required>
                            <option value="">请选择目录</option>
                            {}
                        </select>
                    </div>
                    <div>
                        <label for="taskType">删除类型</label>
                        <select id="taskType" name="task_type" required>
                            <option value="delete_older">删除早于指定时间的文件</option>
                            <option value="delete_newer">删除晚于指定时间的文件</option>
                            <option value="delete_between">删除指定时间段的文件</option>
                        </select>
                    </div>
                </div>

                <div class="grid">
                    <div>
                        <label for="daysOld">天数（可选）</label>
                        <input type="number" id="daysOld" name="days_old" min="0">
                    </div>
                    <div>
                        <label for="dateFrom">开始日期（可选）</label>
                        <input type="datetime-local" id="dateFrom" name="date_from">
                    </div>
                    <div>
                        <label for="dateTo">结束日期（可选）</label>
                        <input type="datetime-local" id="dateTo" name="date_to">
                    </div>
                </div>

                <div class="grid">
                    <div>
                        <label for="filePattern">文件模式（可选）</label>
                        <input type="text" id="filePattern" name="file_pattern" placeholder="例如: .log">
                    </div>
                    <div>
                        <label>&nbsp;</label>
                        <button type="button" onclick="deleteFiles(true)">预览删除</button>
                        <button type="button" onclick="deleteFiles(false)" class="button-danger">执行删除</button>
                    </div>
                </div>
            </form>

            <div id="deleteResult" style="display: none;">
                <h3>删除结果</h3>
                <pre id="deleteResultContent"></pre>
            </div>
        </section>

        <section>
            <h2>定时任务</h2>
            <form id="createTaskForm">
                <div class="grid">
                    <div>
                        <label for="taskDirectorySelect">选择目录</label>
                        <select id="taskDirectorySelect" name="directory_id" required>
                            <option value="">请选择目录</option>
                            {}
                        </select>
                    </div>
                    <div>
                        <label for="taskName">任务名称</label>
                        <input type="text" id="taskName" name="name" required>
                    </div>
                </div>

                <div class="grid">
                    <div>
                        <label for="cronExpression">Cron表达式</label>
                        <input type="text" id="cronExpression" name="cron_expression" required placeholder="0 0 * * *">
                    </div>
                    <div>
                        <label for="taskTaskType">删除类型</label>
                        <select id="taskTaskType" name="task_type" required>
                            <option value="delete_older">删除早于指定时间的文件</option>
                            <option value="delete_newer">删除晚于指定时间的文件</option>
                            <option value="delete_between">删除指定时间段的文件</option>
                        </select>
                    </div>
                    <div>
                        <label>&nbsp;</label>
                        <button type="submit">创建任务</button>
                    </div>
                </div>
            </form>

            <table>
                <thead>
                    <tr>
                        <th>任务名称</th>
                        <th>Cron表达式</th>
                        <th>类型</th>
                        <th>状态</th>
                        <th>操作</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </section>
    </main>

    <script>
        // API调用函数
        async function addDirectory() {{
            const form = document.getElementById('addDirectoryForm');
            const formData = new FormData(form);
            const data = {{
                path: formData.get('path'),
                description: formData.get('description')
            }};

            try {{
                const response = await fetch('/api/directories', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(data)
                }});

                if (response.ok) {{
                    location.reload();
                }} else {{
                    const error = await response.json();
                    alert('错误: ' + error.error);
                }}
            }} catch (e) {{
                alert('请求失败: ' + e.message);
            }}
        }}

        async function deleteDirectory(id) {{
            if (confirm('确定要删除这个目录吗？')) {{
                try {{
                    const response = await fetch(`/api/directories/${{id}}`, {{
                        method: 'DELETE'
                    }});

                    if (response.ok) {{
                        location.reload();
                    }} else {{
                        alert('删除失败');
                    }}
                }} catch (e) {{
                    alert('请求失败: ' + e.message);
                }}
            }}
        }}

        async function deleteFiles(dryRun) {{
            const form = document.getElementById('deleteFilesForm');
            const formData = new FormData(form);

            const data = {{
                directory_id: formData.get('directory_id'),
                task_type: formData.get('task_type'),
                dry_run: dryRun,
                filter_config: {{
                    days_old: formData.get('days_old') ? parseInt(formData.get('days_old')) : null,
                    date_from: formData.get('date_from') ? new Date(formData.get('date_from')).toISOString() : null,
                    date_to: formData.get('date_to') ? new Date(formData.get('date_to')).toISOString() : null,
                    file_pattern: formData.get('file_pattern') || null
                }}
            }};

            try {{
                const response = await fetch('/api/delete-files', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(data)
                }});

                if (response.ok) {{
                    const result = await response.json();
                    document.getElementById('deleteResult').style.display = 'block';
                    document.getElementById('deleteResultContent').textContent = JSON.stringify(result, null, 2);
                }} else {{
                    const error = await response.json();
                    alert('错误: ' + error.error);
                }}
            }} catch (e) {{
                alert('请求失败: ' + e.message);
            }}
        }}

        async function createTask() {{
            const form = document.getElementById('createTaskForm');
            const formData = new FormData(form);

            const data = {{
                directory_id: formData.get('directory_id'),
                name: formData.get('name'),
                cron_expression: formData.get('cron_expression'),
                task_type: formData.get('task_type'),
                filter_config: {{
                    days_old: 7,
                    date_from: null,
                    date_to: null,
                    file_pattern: null
                }}
            }};

            try {{
                const response = await fetch('/api/tasks', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(data)
                }});

                if (response.ok) {{
                    location.reload();
                }} else {{
                    const error = await response.json();
                    alert('错误: ' + error.error);
                }}
            }} catch (e) {{
                alert('请求失败: ' + e.message);
            }}
        }}

        async function deleteTask(id) {{
            if (confirm('确定要删除这个任务吗？')) {{
                try {{
                    const response = await fetch(`/api/tasks/${{id}}`, {{
                        method: 'DELETE'
                    }});

                    if (response.ok) {{
                        location.reload();
                    }} else {{
                        alert('删除失败');
                    }}
                }} catch (e) {{
                    alert('请求失败: ' + e.message);
                }}
            }}
        }}

        // 表单提交事件
        document.getElementById('addDirectoryForm').addEventListener('submit', function(e) {{
            e.preventDefault();
            addDirectory();
        }});

        document.getElementById('createTaskForm').addEventListener('submit', function(e) {{
            e.preventDefault();
            createTask();
        }});
    </script>
</body>
</html>"#,
        directories_html,
        directories.iter().map(|d| format!(r#"<option value="{}">{}</option>"#, d.id, d.path)).collect::<Vec<_>>().join("\n"),
        directories.iter().map(|d| format!(r#"<option value="{}">{}</option>"#, d.id, d.path)).collect::<Vec<_>>().join("\n"),
        tasks_html
    ))
}