# SillyTavern Launcher (酒馆启动器)

[中文](#中文) | [English](#english)

---

<a id="中文"></a>

# SillyTavern Launcher GUI (酒馆启动器) (中文)

酒馆启动器（SillyTavern Launcher）是一个专为 SillyTavern 打造的跨平台桌面客户端应用。它基于 Tauri、Vue 3 和 TypeScript 构建，提供了一个简单、高效且现代的图形界面，帮助用户轻松管理和运行 SillyTavern 环境。

## 🚀 核心功能

- **跨平台支持**：全面支持 Windows、macOS 和 Linux 操作系统。
- **一键启动**：快速配置并启动你的 SillyTavern 环境。
- **插件管理**：轻松安装、更新和管理 SillyTavern 的各种插件与扩展。
- **现代 UI**：基于 Vue 3 和 TailwindCSS 打造的简洁美观的用户界面。
- **高性能**：后端采用 Rust 编写，确保极低的内存占用和极快的运行速度。

## 🛠️ 技术栈

- **前端框架**：Vue 3, TypeScript, Vite
- **UI 组件/样式**：TailwindCSS, DaisyUI, Lucide Icons
- **后端/桌面框架**：Rust, Tauri v2
- **包管理工具**：Bun（推荐）

## 📁 项目目录结构

```text
SillyTavern-Launcher/
├── src/                # 前端 Vue 源码
│   ├── assets/         # 静态资源（图片、Logo等）
│   ├── components/     # 可复用的 Vue 组件
│   ├── layouts/        # 布局组件（如自定义标题栏 Oheader）
│   ├── lib/            # 工具函数与共享逻辑
│   ├── router/         # Vue Router 路由配置
│   └── views/          # 页面视图组件
├── src-tauri/          # 后端 Rust 源码 (Tauri)
│   ├── src/            # Rust 应用核心逻辑
│   └── tauri.conf.json # Tauri 配置文件
├── data/               # 软件数据目录
│   ├── config.json     # 软件配置文件
│   ├── node/           # Node 核心环境目录
│   ├── sillytavern/    # SillyTavern 核心程序目录
│   ├── plugins/        # 待安装的插件目录
│   └── logs/           # 应用运行日志目录
└── package.json        # 前端依赖与脚本配置
```

## 💻 开发与运行

### 环境准备

- [Node.js](https://nodejs.org/) (或 [Bun](https://bun.sh/) - 强烈推荐)
- [Rust](https://www.rust-lang.org/) (包含 Cargo)
- Tauri 运行所需的系统依赖 (请参考 [Tauri 官方文档](https://v2.tauri.app/start/prerequisites/))

### 安装依赖

本项目默认推荐使用 `bun` 进行依赖管理，以获得更快的安装速度。

```bash
# 安装前端依赖
bun install
```

### 运行开发环境

```bash
# 启动 Vite 开发服务器并打开 Tauri 桌面窗口
bun run tauri dev
```

### 打包构建

```bash
# 构建并打包生产环境的安装程序
bun run tauri build
```

## 📝 代码与开发规范

- **代码质量**：使用 Vue 3 组合式 API (Composition API) 和严格的 TypeScript 类型定义。
- **UI 界面规范**：
  - UI 界面内容必须放置在 `<Oheader></Oheader>` 组件内，以支持自定义窗口边框。
  - 弹窗和全局消息必须放置在 `<Oheader>` 的 `<template #Modal></template>` 插槽中。
  - 图标统一使用 `lucide-vue-next` 和 `@phosphor-icons/vue` 图标库。
  - 严格使用已安装的 UI 库 (TailwindCSS/DaisyUI) 和现有样式，禁止随意编写自定义 CSS 或修改原有风格，保持整体风格一致。
- **架构与跨平台规范**：
  - 所有与系统交互的操作、外部接口请求必须通过 Rust 后端编写函数供前端调用，前端仅负责接收和渲染数据（避免跨域问题）。
  - 开发功能时需确保在 Windows、macOS 和 Linux 上均能正常运行，并为特定平台差异设置降级/回退机制。
- **注释与检查**：写完代码后需及时进行测试，避免代码重复，并使用标准的注释规范对复杂逻辑进行说明。

## 🤝 参与贡献

欢迎提交 Issue 报告问题或提出新功能建议，也非常欢迎提交 Pull Request 参与代码贡献！

## 📄 开源许可

本项目遵循 MIT 开源许可证。

---

<a id="english"></a>

# SillyTavern Launcher GUI (English)

SillyTavern Launcher is a cross-platform desktop application designed specifically for SillyTavern. Built with Tauri, Vue 3, and TypeScript, it provides a simple, efficient, and modern graphical interface to help users easily manage and run their SillyTavern environments.

## 🚀 Features

- **Cross-Platform Support**: Runs on Windows, macOS, and Linux.
- **One-Click Startup**: Quickly configure and launch your SillyTavern environment.
- **Plugin Management**: Easily install and manage SillyTavern plugins and extensions.
- **Modern UI**: Clean and beautiful user interface built with Vue 3 and TailwindCSS.
- **High Performance**: The backend is powered by Rust, ensuring low memory usage and fast execution.

## 🛠️ Tech Stack

- **Frontend**: Vue 3, TypeScript, Vite, TailwindCSS, DaisyUI, Lucide Icons
- **Backend**: Rust, Tauri v2
- **Package Manager**: Bun (Recommended)

## 📁 Project Structure

```text
SillyTavern-Launcher/
├── src/                # Frontend Vue source code
│   ├── assets/         # Static assets (images, logos)
│   ├── components/     # Reusable Vue components
│   ├── layouts/        # Layout components (e.g., Header)
│   ├── lib/            # Utility functions and shared logic
│   ├── router/         # Vue Router configuration
│   └── views/          # Page views
├── src-tauri/          # Backend Rust source code (Tauri)
│   ├── src/            # Rust application logic
│   └── tauri.conf.json # Tauri configuration file
├── data/               # App data directory
│   ├── config.json     # App configuration
│   ├── node/           # Node core directory
│   ├── sillytavern/    # SillyTavern core directory
│   ├── plugins/        # SillyTavern plugins to be installed
│   └── logs/           # Application logs
└── package.json        # Frontend dependencies and scripts
```

## 💻 Development Setup

### Prerequisites

- [Node.js](https://nodejs.org/) (or [Bun](https://bun.sh/) - Recommended)
- [Rust](https://www.rust-lang.org/) (with Cargo)
- Necessary system dependencies for Tauri (Refer to [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/))

### Install Dependencies

It is highly recommended to use `bun` for managing dependencies in this project.

```bash
# Install frontend dependencies
bun install
```

### Run Development Server

```bash
# Start Vite dev server and Tauri window
bun run tauri dev
```

### Build for Production

```bash
# Build the application installer
bun run tauri build
```

## 📝 Coding Standards & Guidelines

- **Code Quality**: Follow Vue 3 Composition API best practices and strict TypeScript typing.
- **UI Guidelines**:
  - All global dialogs/messages should be placed inside `<template #Modal>` in `<Oheader>`.
  - Main UI interfaces should be wrapped within `<Oheader></Oheader>` to support custom window borders.
  - Use `lucide-vue-next` and `@phosphor-icons/vue` for icons.
  - Rely on existing UI libraries (TailwindCSS/DaisyUI); do not write custom CSS unless absolutely necessary.
- **Architecture**:
  - Frontend only receives data; all system-level interactions and API requests must be routed through Rust backend commands to avoid CORS and security issues.
  - Ensure cross-platform compatibility with fallback mechanisms.

## 🤝 Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the issues page.

## 📄 License

This project is open-source and available under the terms of the MIT License.
