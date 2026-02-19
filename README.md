# dev-boom

一个基于 **Tauri 2 + Vue 3 + TypeScript** 的项目管理器桌面应用（Windows 优先）。

## 当前功能

- 项目自动扫描导入（选择扫描根目录 + 扫描深度）
- 项目搜索（名称 / 路径 / 标签）
- 收藏筛选
- 项目卡片快捷操作：启动、打开文件夹、移除
- IDE 手动配置（添加 IDE、参数模板）
- 无边框沉浸式标题栏
- 浅色 / 深色主题切换

## 技术栈

- 前端：Vue 3 + TypeScript + Vite
- 桌面框架：Tauri 2
- 后端：Rust
- 插件：
  - `@tauri-apps/plugin-dialog`
  - `@tauri-apps/plugin-opener`

## 本地开发

```bash
pnpm install
pnpm tauri dev
```

## 构建

```bash
pnpm build
pnpm tauri build
```

## 项目结构（简）

- `src/components`：UI 组件
- `src/composables`：状态和业务逻辑
- `src/api`：Tauri invoke 封装
- `src/types`：前端类型定义
- `src-tauri`：Rust 后端与 Tauri 配置
