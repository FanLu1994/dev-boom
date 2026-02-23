# GitHub Actions Workflows

这个仓库使用 GitHub Actions 实现自动化构建和发布。

## Workflows

### 1. CI (`.github/workflows/ci.yml`)

**触发条件:**
- Push 到 `main` 或 `develop` 分支
- 针对 `main` 分支的 Pull Request

**检查项目:**
- 类型检查 (`vue-tsc --noEmit`)
- 测试构建 (`tauri build --debug`)

### 2. Build and Release (`.github/workflows/build-and-release.yml`)

**触发条件:**
- Push tag (例如 `v1.0.0`)
- 手动触发 (在 Actions 页面)

**功能:**
- 在 Windows 上构建应用
- 生成安装包和可执行文件
- 自动创建 GitHub Release
- 上传构建产物

## 如何创建新版本

### 方法 1: 使用 Git Tag（推荐）

```bash
# 创建版本标签
git tag v1.0.0

# 推送标签到远程
git push origin v1.0.0
```

这将自动触发 Release workflow，构建并发布新版本。

### 方法 2: 手动触发

1. 进入 GitHub 仓库页面
2. 点击 "Actions" 标签
3. 选择 "Build and Release" workflow
4. 点击 "Run workflow"
5. 勾选 "Create a new release"
6. 点击 "Run workflow" 按钮

## 版本号规则

- 正式版本: `v1.0.0`, `v1.2.3` 等
- 测试版本: `v1.0.0-beta`, `v2.0.0-rc1` 等（会被标记为 prerelease）
- CI 构建: `v0.1.0-{build_number}`

## 发布产物

每次发布会生成以下文件：

- **dev-boom-setup.exe**: NSIS 安装程序（推荐）
- **dev-boom-portable.exe**: 便携版可执行文件
- **dev-boom-x.x.x.x.msi**: Windows MSI 安装包

## 本地构建

如果需要在本地构建发布版本：

```bash
# 安装依赖
pnpm install

# 构建生产版本
pnpm tauri build

# 构建产物在 src-tauri/target/release/ 目录
```

## 注意事项

1. **第一次构建**: 第一次运行可能需要较长时间下载 Rust 依赖
2. **构建时间**: 完整构建大约需要 10-20 分钟
3. **磁盘空间**: 确保有足够的磁盘空间（至少 5GB）
4. **权限**: 确保仓库有正确的 Actions 权限设置
