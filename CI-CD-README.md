# CI/CD 配置说明

本项目包含了完整的 CI/CD 配置，支持自动化构建、测试和发布。

## GitHub Actions 工作流

### 1. 持续集成 (CI) - `.github/workflows/ci.yml`

**触发条件：**
- 推送到 `main`、`master` 或 `develop` 分支
- 针对这些分支的 Pull Request

**功能：**
- 在多个操作系统（Ubuntu、Windows、macOS）上运行测试
- 代码质量检查（Clippy）
- 代码格式检查（rustfmt）
- 安全审计（cargo-audit）
- 构建检查

### 2. 发布构建 (Release) - `.github/workflows/release.yml`

**触发条件：**
- 推送标签（格式：`v*`，如 `v1.0.0`）
- 手动触发

**支持的平台：**
- Linux: x86_64, aarch64, arm, armv7
- Windows: x86_64
- macOS: x86_64, aarch64 (Apple Silicon)

**功能：**
- 跨平台编译
- 二进制文件压缩（UPX）
- 自动创建 GitHub Release
- 生成 SHA256 校验和

### 3. Docker 构建 - `.github/workflows/docker.yml`

**触发条件：**
- 推送到 `main` 或 `master` 分支
- 推送标签
- Pull Request

**功能：**
- 构建多架构 Docker 镜像（amd64、arm64）
- 发布到 GitHub Container Registry
- 自动标签管理

## 使用方法

### 创建发布版本

1. **创建并推送标签：**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **自动触发：**
   - Release 工作流将自动运行
   - 构建所有支持平台的二进制文件
   - 创建 GitHub Release（草稿状态）
   - 上传构建产物和校验和

3. **发布：**
   - 在 GitHub Releases 页面编辑发布说明
   - 将草稿状态改为正式发布

### Docker 镜像使用

**拉取镜像：**
```bash
# 最新版本
docker pull ghcr.io/your-username/devicl:latest

# 特定版本
docker pull ghcr.io/your-username/devicl:v1.0.0
```

**运行容器：**
```bash
docker run -d \
  --name devicl \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  ghcr.io/your-username/devicl:latest
```

### 本地开发

**运行 CI 检查：**
```bash
# 代码格式化
cargo fmt --all

# 代码检查
cargo clippy -- -D warnings

# 运行测试
cargo test

# 安全审计
cargo audit
```

**本地 Docker 构建：**
```bash
# 构建镜像
docker build -t devicl .

# 运行容器
docker run -p 3000:3000 devicl
```

## 配置要求

### GitHub 仓库设置

1. **启用 GitHub Actions**
2. **设置分支保护规则**（可选）：
   - 要求 PR 通过 CI 检查
   - 要求代码审查

3. **配置 secrets**（如果需要）：
   - 默认使用 `GITHUB_TOKEN`，无需额外配置

### 环境变量

工作流中使用的环境变量：
- `CARGO_TERM_COLOR=always` - 彩色输出
- `RUST_LOG=info` - 日志级别（Docker）
- `DATABASE_URL` - 数据库路径（Docker）

## 自定义配置

### 修改支持的平台

编辑 `.github/workflows/release.yml` 中的 `matrix.include` 部分：

```yaml
matrix:
  include:
    - os: ubuntu-latest
      target: your-target-triple
      exe: devicl
      cross: true/false
```

### 修改 Docker 镜像配置

编辑 `Dockerfile` 以自定义：
- 基础镜像
- 运行时依赖
- 环境变量
- 健康检查

### 添加额外的检查

在 CI 工作流中添加更多步骤：
- 代码覆盖率
- 性能测试
- 集成测试
- 文档生成

## 故障排除

### 常见问题

1. **构建失败**：
   - 检查依赖是否正确安装
   - 确认目标平台支持所有依赖

2. **跨编译失败**：
   - 某些依赖可能不支持特定平台
   - 考虑使用 `--no-default-features`

3. **Docker 构建失败**：
   - 检查 Dockerfile 中的路径
   - 确认所有必需文件都包含在构建上下文中

### 调试技巧

1. **本地测试**：
   ```bash
   # 测试特定平台构建
   cargo build --target x86_64-unknown-linux-musl

   # 使用 cross 进行跨编译
   cross build --target aarch64-unknown-linux-musl
   ```

2. **查看工作流日志**：
   - 在 GitHub Actions 页面查看详细日志
   - 关注错误信息和警告

3. **手动触发**：
   - 使用 `workflow_dispatch` 手动触发工作流
   - 用于测试配置更改