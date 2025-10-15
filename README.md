# Devicl - 文件清理管理器

## 本工具解决了什么

离线版本的客户端，视频因为不会上传，所以需要进行清理。

一个基于 Rust 开发的文件清理管理工具，支持定时任务和 Web 界面管理。

## 功能特性

- 🗂️ **目录管理**：管理需要清理的目录
- ⏰ **定时任务**：使用 Cron 表达式设置定时清理任务
- 🎯 **灵活过滤**：支持按时间、文件模式等条件过滤文件
- 🌐 **Web 界面**：友好的 Web 管理界面
- 🔐 **密码保护**：管理员密码保护
- 🌍 **多语言**：支持中英文界面
- 🔄 **实时预览**：删除前可预览将要删除的文件

## 快速开始

### 方式一：使用构建脚本（推荐）

1. **运行构建脚本**
   ```bash
   ./build.sh
   ```
   脚本会询问要构建的目标架构（x86_64 / aarch64 / 两者都构建）

2. **提取发布包**
   ```bash
   cd release
   tar -xzf devicl-x86_64-linux.tar.gz  # 或 devicl-aarch64-linux.tar.gz
   cd devicl-x86_64-linux
   ```

3. **安装**
   ```bash
   sudo ./install.sh
   ```

### 方式二：手动构建安装

1. **编译项目**
   ```bash
   # x86_64
   cargo build --release

   # ARM64
   cargo build --release --target aarch64-unknown-linux-gnu
   ```

2. **安装**
   ```bash
   sudo ./install.sh
   ```

## 使用说明

### 访问应用

安装完成后，在浏览器中访问：
```
http://localhost:3000
```

默认密码：`holomotion`

⚠️ **重要**：首次登录后请立即修改密码！

### 服务管理

```bash
# 启动服务
sudo systemctl start devicl

# 停止服务
sudo systemctl stop devicl

# 重启服务
sudo systemctl restart devicl

# 查看状态
sudo systemctl status devicl

# 查看日志
sudo journalctl -u devicl -f
```

## 文件说明

- `build.sh` - 构建和打包脚本
- `install.sh` - 一键安装脚本
- `uninstall.sh` - 卸载脚本
- `devicl.service` - systemd 服务配置文件
- `INSTALL.md` - 详细安装和使用文档

## 系统要求

- Linux 系统（支持 systemd）
- root 权限（安装时）
- 架构：x86_64 或 aarch64 (ARM64)

## 配置文件位置

安装后的文件位置：
- 程序：`/opt/devicl/devicl`
- 数据库：`/opt/devicl/devicl.db`
- 服务文件：`/etc/systemd/system/devicl.service`

## 开发

### 构建依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# ARM64 交叉编译工具（如需要）
sudo pacman -S aarch64-linux-gnu-gcc  # Arch Linux
# 或
sudo apt install gcc-aarch64-linux-gnu  # Debian/Ubuntu
```

### 本地运行

```bash
cargo run
```

### 运行测试

```bash
cargo test
```

## 卸载

```bash
sudo ./uninstall.sh
```

## 许可证

MIT

## 贡献

欢迎提交 Issue 和 Pull Request！
