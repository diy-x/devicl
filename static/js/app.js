// 全局状态管理
class AppState {
    constructor() {
        this.directories = [];
        this.tasks = [];
        this.loading = false;
        this.currentLanguage = this.getLanguageFromURL() || localStorage.getItem('language') || 'zh';
        this.messages = {};
    }

    setLoading(loading) {
        this.loading = loading;
        document.body.classList.toggle('loading', loading);
    }

    getLanguageFromURL() {
        const urlParams = new URLSearchParams(window.location.search);
        return urlParams.get('lang');
    }

    async setLanguage(lang) {
        this.currentLanguage = lang;
        localStorage.setItem('language', lang);

        // Load messages for the new language
        await this.loadMessages();

        // Update URL
        const url = new URL(window.location);
        url.searchParams.set('lang', lang);
        window.history.pushState({}, '', url);

        // Reload page to apply new language
        window.location.reload();
    }

    async loadMessages() {
        try {
            const response = await fetch(`/api/messages?lang=${this.currentLanguage}`);
            this.messages = await response.json();
        } catch (error) {
            console.error('Failed to load messages:', error);
        }
    }
}

const appState = new AppState();

// 语言切换功能
function changeLanguage(language) {
    appState.setLanguage(language);
}

// 通知系统
function showNotification(message, type = 'info', duration = 5000) {
    const notification = document.getElementById('notification');
    notification.textContent = message;
    notification.className = `notification ${type}`;
    notification.style.display = 'block';

    setTimeout(() => {
        notification.style.display = 'none';
    }, duration);
}

// 获取本地化消息
function getMessage(key) {
    return appState.messages[key] || key;
}

// API 调用封装
class ApiClient {
    static async request(url, options = {}) {
        try {
            appState.setLoading(true);
            const response = await fetch(url, {
                headers: {
                    'Content-Type': 'application/json',
                    ...options.headers
                },
                ...options
            });

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || `HTTP ${response.status}`);
            }

            return data;
        } catch (error) {
            showNotification(`${getMessage('request_failed')}: ${error.message}`, 'error');
            throw error;
        } finally {
            appState.setLoading(false);
        }
    }

    static async get(url) {
        return this.request(url);
    }

    static async post(url, data) {
        return this.request(url, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    }

    static async delete(url) {
        return this.request(url, { method: 'DELETE' });
    }
}

// 目录管理
async function addDirectory() {
    const form = document.getElementById('addDirectoryForm');
    const formData = new FormData(form);

    const data = {
        path: formData.get('path'),
        description: formData.get('description') || null
    };

    try {
        await ApiClient.post('/api/directories', data);
        showNotification(getMessage('directory_added_success'), 'success');
        form.reset();
        await loadDirectories();
    } catch (error) {
        // Error already handled in ApiClient
    }
}

async function deleteDirectory(id) {
    if (!confirm(getMessage('delete_directory_confirm'))) {
        return;
    }

    try {
        await ApiClient.delete(`/api/directories/${id}`);
        showNotification(getMessage('directory_deleted_success'), 'success');
        await loadDirectories();
        await loadTasks();
    } catch (error) {
        // Error already handled in ApiClient
    }
}

async function loadDirectories() {
    try {
        const directories = await ApiClient.get('/api/directories');
        appState.directories = directories;
        updateDirectoriesTable(directories);
        updateDirectorySelects(directories);
    } catch (error) {
        // Error already handled in ApiClient
    }
}

function updateDirectoriesTable(directories) {
    const tbody = document.getElementById('directoriesTableBody');
    if (!tbody) return;

    tbody.innerHTML = directories.map(dir => `
        <tr>
            <td><code>${escapeHtml(dir.path)}</code></td>
            <td>${escapeHtml(dir.description || '')}</td>
            <td>
                <span class="status-${dir.enabled ? 'enabled' : 'disabled'}">
                    ${dir.enabled ? '✅ 启用' : '❌ 禁用'}
                </span>
            </td>
            <td>${formatDateTime(dir.created_at)}</td>
            <td>
                <button onclick="deleteDirectory('${dir.id}')" class="secondary outline">删除</button>
            </td>
        </tr>
    `).join('');
}

function updateDirectorySelects(directories) {
    const selects = ['directorySelect', 'taskDirectorySelect'];

    selects.forEach(selectId => {
        const select = document.getElementById(selectId);
        if (!select) return;

        // 保存当前选中值
        const currentValue = select.value;

        select.innerHTML = `<option value="">${getMessage('select_directory')}</option>` +
            directories.map(dir =>
                `<option value="${dir.id}">${escapeHtml(dir.path)}</option>`
            ).join('');

        // 恢复选中值
        if (currentValue) {
            select.value = currentValue;
        }
    });
}

// 标签页切换
function switchTimeTab(tab) {
    // 切换标签按钮状态
    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.remove('active');
        btn.style.borderBottomColor = 'transparent';
        btn.style.color = 'var(--muted-color)';
    });

    const activeTab = document.getElementById(`tab${tab === 'byDays' ? 'ByDays' : 'ByDate'}`);
    activeTab.classList.add('active');
    activeTab.style.borderBottomColor = 'var(--primary)';
    activeTab.style.color = 'var(--primary)';

    // 切换内容区域
    document.getElementById('timeTabByDays').style.display = tab === 'byDays' ? 'block' : 'none';
    document.getElementById('timeTabByDate').style.display = tab === 'byDate' ? 'block' : 'none';
}

// 更新日期字段显示
function updateDateFields() {
    const taskType = document.getElementById('taskTypeByDate').value;
    const singleDateField = document.getElementById('singleDateField');
    const rangeDateFields = document.getElementById('rangeDateFields');
    const singleDateLabel = document.getElementById('singleDateLabel');

    if (taskType === 'delete_between') {
        // 显示区间选择
        singleDateField.style.display = 'none';
        rangeDateFields.style.display = 'grid';
    } else {
        // 显示单个日期选择
        singleDateField.style.display = 'grid';
        rangeDateFields.style.display = 'none';

        // 更新标签文本
        if (taskType === 'delete_older') {
            singleDateLabel.textContent = getMessage('before_date') || '早于此时间';
        } else if (taskType === 'delete_newer') {
            singleDateLabel.textContent = getMessage('after_date') || '晚于此时间';
        }
    }
}

// 文件删除操作
async function deleteFiles(dryRun) {
    const form = document.getElementById('deleteFilesForm');
    const formData = new FormData(form);

    if (!formData.get('directory_id')) {
        showNotification(getMessage('select_directory_warning'), 'warning');
        return;
    }

    // 检查当前激活的标签页
    const byDaysTab = document.getElementById('timeTabByDays');
    const isByDaysMode = byDaysTab.style.display !== 'none';

    let taskType;
    let filterConfig;

    if (isByDaysMode) {
        // 按天数模式 - 固定为删除早于N天的文件
        const daysOld = document.getElementById('daysOld').value;
        if (!daysOld || parseInt(daysOld) < 1) {
            showNotification('请输入有效的天数（至少为1天）', 'warning');
            return;
        }

        taskType = 'delete_older';  // 固定为删除早于
        filterConfig = {
            days_old: parseInt(daysOld),
            date_from: null,
            date_to: null,
            file_pattern: formData.get('file_pattern') || null
        };
    } else {
        // 按时间模式
        taskType = document.getElementById('taskTypeByDate').value;

        if (taskType === 'delete_between') {
            const dateFrom = document.getElementById('dateFrom').value;
            const dateTo = document.getElementById('dateTo').value;

            if (!dateFrom || !dateTo) {
                showNotification('请选择开始和结束时间', 'warning');
                return;
            }

            filterConfig = {
                days_old: null,
                date_from: new Date(dateFrom).toISOString(),
                date_to: new Date(dateTo).toISOString(),
                file_pattern: formData.get('file_pattern') || null
            };
        } else {
            const singleDate = document.getElementById('singleDate').value;

            if (!singleDate) {
                showNotification('请选择时间', 'warning');
                return;
            }

            filterConfig = {
                days_old: null,
                date_from: taskType === 'delete_older' ? new Date(singleDate).toISOString() : null,
                date_to: taskType === 'delete_newer' ? new Date(singleDate).toISOString() : null,
                file_pattern: formData.get('file_pattern') || null
            };
        }
    }

    const data = {
        directory_id: formData.get('directory_id'),
        task_type: taskType,
        dry_run: dryRun,
        filter_config: filterConfig
    };

    try {
        const result = await ApiClient.post('/api/delete-files', data);

        const resultDiv = document.getElementById('deleteResult');
        const resultContent = document.getElementById('deleteResultContent');

        resultDiv.style.display = 'block';
        resultContent.textContent = JSON.stringify(result, null, 2);

        const message = dryRun
            ? `预览完成：找到 ${result.total_files} 个文件`
            : `删除完成：共删除 ${result.total_deleted} 个文件`;

        showNotification(message, dryRun ? 'info' : 'success');

        // 滚动到结果区域
        resultDiv.scrollIntoView({ behavior: 'smooth' });
    } catch (error) {
        // Error already handled in ApiClient
    }
}

// 定时任务标签页切换
function switchTaskTimeTab(tab) {
    // 切换标签按钮状态
    const tabButtons = document.querySelectorAll('#taskTabByDays, #taskTabByDate');
    tabButtons.forEach(btn => {
        btn.classList.remove('active');
        btn.style.borderBottomColor = 'transparent';
        btn.style.color = 'var(--muted-color)';
    });

    const activeTab = document.getElementById(`task${tab === 'byDays' ? 'TabByDays' : 'TabByDate'}`);
    activeTab.classList.add('active');
    activeTab.style.borderBottomColor = 'var(--primary)';
    activeTab.style.color = 'var(--primary)';

    // 切换内容区域
    document.getElementById('taskTimeTabByDays').style.display = tab === 'byDays' ? 'block' : 'none';
    document.getElementById('taskTimeTabByDate').style.display = tab === 'byDate' ? 'block' : 'none';
}

// 更新任务日期字段显示
function updateTaskDateFields() {
    const taskType = document.getElementById('taskTypeByDate').value;
    const singleDateField = document.getElementById('taskSingleDateField');
    const rangeDateFields = document.getElementById('taskRangeDateFields');
    const singleDateLabel = document.getElementById('taskSingleDateLabel');

    if (taskType === 'delete_between') {
        // 显示区间选择
        singleDateField.style.display = 'none';
        rangeDateFields.style.display = 'grid';
    } else {
        // 显示单个日期选择
        singleDateField.style.display = 'grid';
        rangeDateFields.style.display = 'none';

        // 更新标签文本
        if (taskType === 'delete_older') {
            singleDateLabel.textContent = getMessage('before_date') || '早于此时间';
        } else if (taskType === 'delete_newer') {
            singleDateLabel.textContent = getMessage('after_date') || '晚于此时间';
        }
    }
}

// 定时任务管理
async function createTask() {
    const form = document.getElementById('createTaskForm');
    const formData = new FormData(form);

    if (!formData.get('directory_id')) {
        showNotification(getMessage('select_directory_warning'), 'warning');
        return;
    }

    // 检查当前激活的标签页
    const byDaysTab = document.getElementById('taskTimeTabByDays');
    const isByDaysMode = byDaysTab.style.display !== 'none';

    let taskType;
    let filterConfig;

    if (isByDaysMode) {
        // 按天数模式 - 固定为删除早于N天的文件
        const daysOld = document.getElementById('taskDaysOld').value;
        if (!daysOld || parseInt(daysOld) < 1) {
            showNotification('请输入有效的天数（至少为1天）', 'warning');
            return;
        }

        taskType = 'delete_older';  // 固定为删除早于
        filterConfig = {
            days_old: parseInt(daysOld),
            date_from: null,
            date_to: null,
            file_pattern: formData.get('task_file_pattern') || null
        };
    } else {
        // 按时间模式
        taskType = document.getElementById('taskTypeByDate').value;

        if (taskType === 'delete_between') {
            const dateFrom = document.getElementById('taskDateFrom').value;
            const dateTo = document.getElementById('taskDateTo').value;

            if (!dateFrom || !dateTo) {
                showNotification('请选择开始和结束时间', 'warning');
                return;
            }

            filterConfig = {
                days_old: null,
                date_from: new Date(dateFrom).toISOString(),
                date_to: new Date(dateTo).toISOString(),
                file_pattern: formData.get('task_file_pattern') || null
            };
        } else {
            const singleDate = document.getElementById('taskSingleDate').value;

            if (!singleDate) {
                showNotification('请选择时间', 'warning');
                return;
            }

            filterConfig = {
                days_old: null,
                date_from: taskType === 'delete_older' ? new Date(singleDate).toISOString() : null,
                date_to: taskType === 'delete_newer' ? new Date(singleDate).toISOString() : null,
                file_pattern: formData.get('task_file_pattern') || null
            };
        }
    }

    const data = {
        directory_id: formData.get('directory_id'),
        name: formData.get('name'),
        cron_expression: formData.get('cron_expression'),
        task_type: taskType,
        filter_config: filterConfig
    };

    try {
        await ApiClient.post('/api/tasks', data);
        showNotification('定时任务创建成功！', 'success');
        form.reset();
        // 重置标签页到默认状态
        switchTaskTimeTab('byDays');
        await loadTasks();
    } catch (error) {
        // Error already handled in ApiClient
    }
}

async function deleteTask(id) {
    if (!confirm('确定要删除这个定时任务吗？')) {
        return;
    }

    try {
        await ApiClient.delete(`/api/tasks/${id}`);
        showNotification('定时任务删除成功！', 'success');
        await loadTasks();
    } catch (error) {
        // Error already handled in ApiClient
    }
}

async function loadTasks() {
    try {
        const tasks = await ApiClient.get('/api/tasks');
        appState.tasks = tasks;
        updateTasksTable(tasks);
    } catch (error) {
        // Error already handled in ApiClient
    }
}

function updateTasksTable(tasks) {
    const tbody = document.getElementById('tasksTableBody');
    if (!tbody) return;

    tbody.innerHTML = tasks.map(task => {
        const directory = appState.directories.find(d => d.id === task.directory_id);
        const directoryPath = directory ? directory.path : task.directory_id;

        return `
            <tr>
                <td><strong>${escapeHtml(task.name)}</strong></td>
                <td><code>${escapeHtml(directoryPath)}</code></td>
                <td><code>${escapeHtml(task.cron_expression)}</code></td>
                <td>${getTaskTypeLabel(task.task_type)}</td>
                <td>
                    <span class="status-${task.enabled ? 'enabled' : 'disabled'}">
                        ${task.enabled ? '✅ 启用' : '❌ 禁用'}
                    </span>
                </td>
                <td>
                    ${task.last_run ? formatDateTime(task.last_run) : '<em>从未运行</em>'}
                </td>
                <td>
                    <button onclick="deleteTask('${task.id}')" class="secondary outline">删除</button>
                </td>
            </tr>
        `;
    }).join('');
}

// 工具函数
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatDateTime(dateString) {
    const date = new Date(dateString);
    return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
    });
}

function getTaskTypeLabel(taskType) {
    const labels = {
        'delete_older': '删除早于',
        'delete_newer': '删除晚于',
        'delete_between': '删除区间'
    };
    return labels[taskType] || taskType;
}

// 表单验证
function validateCronExpression(expression) {
    // 简单的cron表达式验证
    const parts = expression.trim().split(/\s+/);
    return parts.length === 5;
}

// 事件监听
document.addEventListener('DOMContentLoaded', function() {
    // 加载初始数据
    loadDirectories();
    loadTasks();

    // 表单提交事件
    const addDirectoryForm = document.getElementById('addDirectoryForm');
    if (addDirectoryForm) {
        addDirectoryForm.addEventListener('submit', function(e) {
            e.preventDefault();
            addDirectory();
        });
    }

    const createTaskForm = document.getElementById('createTaskForm');
    if (createTaskForm) {
        createTaskForm.addEventListener('submit', function(e) {
            e.preventDefault();

            const cronExpression = document.getElementById('cronExpression').value;
            if (!validateCronExpression(cronExpression)) {
                showNotification('Cron表达式格式不正确，应该包含5个部分（分 时 日 月 周）', 'warning');
                return;
            }

            createTask();
        });
    }

    // Cron表达式提示
    const cronInput = document.getElementById('cronExpression');
    if (cronInput) {
        cronInput.addEventListener('blur', function() {
            if (this.value && !validateCronExpression(this.value)) {
                showNotification('Cron表达式可能不正确，请检查格式', 'warning', 3000);
            }
        });
    }

    // 定期刷新任务状态（每30秒）
    setInterval(() => {
        if (!appState.loading) {
            loadTasks();
        }
    }, 30000);
});

// 键盘快捷键
document.addEventListener('keydown', function(e) {
    // Ctrl+R 刷新数据
    if (e.ctrlKey && e.key === 'r') {
        e.preventDefault();
        loadDirectories();
        loadTasks();
        showNotification('数据已刷新', 'info', 2000);
    }
});