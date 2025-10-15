# Devicl 安装指南

## 系统要求

- Linux 系统 (支持 systemd)
- root 权限
- 已编译好的二进制文件

## 快速安装

### 1. 编译项目

**x86_64 架构：**
```bash
cargo build --release
```

**ARM64/aarch64 架构：**
```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

### 2. 运行安装脚本

```bash
sudo ./install.sh
```

脚本会自动：
- 检测系统架构
- 创建安装目录 `/opt/devicl`
- 复制二进制文件和资源文件
- 安装 systemd 服务
- 询问是否立即启动服务

## 服务管理

### 启动服务
```bash
sudo systemctl start devicl
```

### 停止服务
```bash
sudo systemctl stop devicl
```

### 重启服务
```bash
sudo systemctl restart devicl
```

### 查看服务状态
```bash
sudo systemctl status devicl
```

### 开机自启
```bash
sudo systemctl enable devicl
```

### 禁用开机自启
```bash
sudo systemctl disable devicl
```

### 查看日志
```bash
# 实时查看日志
sudo journalctl -u devicl -f

# 查看最近 50 条日志
sudo journalctl -u devicl -n 50

# 查看今天的日志
sudo journalctl -u devicl --since today
```

## 访问应用

安装完成后，应用将在以下地址运行：

```
http://localhost:3000
```

**默认密码：** `holomotion`

⚠️ **重要：** 首次登录后请立即修改默认密码！

## 配置文件位置

- 二进制文件: `/opt/devicl/devicl`
- 数据库: `/opt/devicl/devicl.db`
- systemd 服务文件: `/etc/systemd/system/devicl.service`

## 卸载

运行卸载脚本：

```bash
sudo ./uninstall.sh
```

卸载脚本会：
- 停止并禁用服务
- 删除 systemd 服务文件
- 询问是否删除应用数据

## 手动配置

### 修改端口

编辑 systemd 服务文件：

```bash
sudo nano /etc/systemd/system/devicl.service
```

在 `[Service]` 部分添加环境变量（示例）：

```ini
Environment="PORT=8080"
```

然后重新加载并重启：

```bash
sudo systemctl daemon-reload
sudo systemctl restart devicl
```

### 修改日志级别

编辑服务文件中的 `RUST_LOG` 环境变量：

```ini
Environment="RUST_LOG=debug"  # 可选: error, warn, info, debug, trace
```

## 故障排查

### 服务启动失败

1. 查看详细日志：
   ```bash
   sudo journalctl -u devicl -n 100 --no-pager
   ```

2. 检查端口占用：
   ```bash
   sudo netstat -tlnp | grep 3000
   ```

3. 检查文件权限：
   ```bash
   ls -la /opt/devicl/
   ```

### 无法访问网页

1. 检查服务是否运行：
   ```bash
   sudo systemctl status devicl
   ```

2. 检查防火墙：
   ```bash
   sudo firewall-cmd --list-ports  # CentOS/RHEL
   sudo ufw status                  # Ubuntu/Debian
   ```

3. 测试本地连接：
   ```bash
   curl http://localhost:3000
   ```

## 升级

1. 停止服务：
   ```bash
   sudo systemctl stop devicl
   ```

2. 备份数据库（可选）：
   ```bash
   sudo cp /opt/devicl/devicl.db /opt/devicl/devicl.db.backup
   ```

3. 编译新版本并运行安装脚本：
   ```bash
   cargo build --release
   sudo ./install.sh
   ```

4. 启动服务：
   ```bash
   sudo systemctl start devicl
   ```

## 安全建议

1. **修改默认密码**：首次登录后立即在设置中修改密码
2. **使用 HTTPS**：在生产环境中建议使用反向代理（如 Nginx）提供 HTTPS
3. **防火墙配置**：只允许必要的 IP 访问
4. **定期备份**：定期备份 `/opt/devicl/devicl.db`

## 使用反向代理（Nginx 示例）

创建 Nginx 配置文件：

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 支持

如有问题，请查看：
- 项目 GitHub Issues
- 日志文件：`sudo journalctl -u devicl`
