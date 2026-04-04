<div align="center">
<img src="https://ghfast.top/https://raw.githubusercontent.com/al01cn/sillyTavern-launcher/GUI/src/assets/images/banner.png" style="width: 100%; height: 100>

---

<img src="https://ghfast.top/https://raw.githubusercontent.com/al01cn/sillyTavern-launcher/GUI/public/logo.png" style="width: 100px; height: 100px;">

# SillyTavern Launcher GUI

**酒馆启动器** · 专为 SillyTavern 打造的跨平台桌面管理工具

[![版本](https://img.shields.io/badge/版本-1.0.2-blue?style=flat-square)](https://github.com/al01cn/sillyTavern-launcher/releases)
[![Tauri](https://img.shields.io/badge/Tauri-v2-orange?style=flat-square&logo=tauri)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-4FC08D?style=flat-square&logo=vue.js)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-latest-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Releases](https://img.shields.io/github/v/release/al01cn/sillytavern-launcher)](https://github.com/al01cn/sillytavern-launcher/releases)

[中文文档](#-中文文档) · [English Docs](#-english-documentation)

</div>

---

## 📖 中文文档

### 简介

**SillyTavern Launcher GUI** 是一个专为 [SillyTavern](https://github.com/SillyTavern/SillyTavern) 设计的跨平台桌面客户端，基于 **Tauri v2 + Vue 3 + Rust** 构建。它提供了直观、现代的图形界面，让用户无需命令行即可完整管理 SillyTavern 的安装、启动、配置和扩展。

> 🌐 官网：[st.al01.cn](https://st.al01.cn/)

---

### ✨ 功能特性

| 功能模块            | 说明                                                           |
| ------------------- | -------------------------------------------------------------- |
| 🚀 **一键启停**     | 一键启动/停止 SillyTavern 服务，实时查看控制台日志             |
| 📦 **版本管理**     | 可视化安装、切换、卸载任意 SillyTavern 版本（稳定版 / 开发版） |
| 🧩 **扩展管理**     | 浏览、启用/禁用、安装（支持 ZIP 离线包）、删除扩展             |
| ⚙️ **酒馆配置**     | 可视化编辑 `config.yaml`（端口、代理、白名单、备份、缩略图等） |
| 🌐 **Node.js 管理** | 自动检测系统 Node.js / npm，支持一键下载内置私有 Node 环境     |
| 🎭 **角色卡管理**   | 浏览、预览、导入、删除角色卡（PNG 格式）                       |
| 📚 **世界书管理**   | 浏览、导入、删除世界书（JSON 格式）                            |
| 🔄 **自动更新**     | 基于 GitHub Releases 的静默自动更新（可关闭提醒）              |
| 🌍 **多语言**       | 内置简体中文 / English，可跟随系统自动切换                     |
| 🎨 **主题切换**     | 深色 / 浅色主题，支持跟随系统                                  |
| 🔧 **GitHub 代理**  | 内置多节点代理加速，解决 GitHub 下载慢的问题                   |
| 💾 **窗口记忆**     | 可选记住上次窗口位置                                           |

---

### 🚀 快速安装（普通用户）

> 如果你只是想**使用**酒馆启动器，不需要搭建开发环境，按以下步骤操作即可：

1. 前往 [GitHub Releases](https://github.com/al01cn/sillyTavern-launcher/releases) 页面
2. 根据你的操作系统下载对应的安装包：

   | 系统    | 文件格式              |
   | ------- | --------------------- |
   | Windows | `.msi` 或 `.exe`      |
   | macOS   | `.dmg`                |
   | Linux   | `.AppImage` 或 `.deb` |

3. 运行安装程序完成安装，然后直接启动即可

> [!WARNING]
> **Windows 用户注意**：如果安装程序提示权限不足或安装失败，请右键点击安装文件，选择「**以管理员身份运行**」后重试。

---

### 🛠️ 技术栈

| 层次             | 技术                                                                                                                                 |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| **前端框架**     | [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vite.dev/)                              |
| **UI 组件**      | [TailwindCSS v4](https://tailwindcss.com/) + [FluxUI](https://fluxui.dev/)（所有组件可用）                                             |
| **图标库**       | [Lucide Vue Next](https://lucide.dev/) + [@phosphor-icons/vue](https://phosphoricons.com/) + [@iconify/vue](https://iconify.design/) |
| **国际化**       | [vue-i18n v11](https://vue-i18n.intlify.dev/)                                                                                        |
| **通知**         | [vue-sonner](https://vue-sonner.vercel.app/)                                                                                         |
| **后端框架**     | [Tauri v2](https://v2.tauri.app/) + [Rust](https://www.rust-lang.org/)                                                               |
| **主要 Rust 库** | `tokio`、`reqwest`、`serde`、`serde_yaml`、`serde_json`、`zip`、`tracing`                                                            |
| **包管理工具**   | [Bun](https://bun.sh/)（推荐）                                                                                                       |

---

### 📁 项目结构

```text
SillyTavern-Launcher/
├── src/                        # 前端 Vue 3 源码
│   ├── assets/                 # 静态资源（图片、Logo 等）
│   ├── components/             # 可复用公共组件（15 个）
│   │   ├── AppTooltip.vue      # 统一悬浮提示组件
│   │   ├── GlobalDialog.vue   # 全局弹窗容器
│   │   ├── InstallDialog.vue  # 版本安装弹窗
│   │   ├── InstallExtensionDialog.vue  # 扩展安装弹窗
│   │   ├── CharacterCardDialog.vue      # 角色卡预览弹窗
│   │   ├── ChatDialog.vue     # 对话历史弹窗
│   │   ├── NetworkLinkDialog.vue        # 局域网/公网链接面板
│   │   ├── OneClickCapsule.vue          # 一键安装胶囊
│   │   ├── ConfigMigrateDialog.vue      # 配置迁移弹窗
│   │   ├── ResourceMigrateDialog.vue    # 资源迁移弹窗
│   │   ├── RepairGitDialog.vue          # Git 修复弹窗
│   │   ├── UploadCharacterCardDialog.vue  # 上传角色卡弹窗
│   │   ├── UploadWorldInfoDialog.vue    # 上传世界书弹窗
│   │   ├── WorldInfoDialog.vue          # 世界书预览弹窗
│   │   └── Model.vue           # 通用的 FluxUI Modal 封装
│   ├── lang/                   # 国际化语言文件（zh-CN / en-US）
│   ├── layouts/                # 布局组件（自定义标题栏）
│   ├── lib/                    # 工具函数与 Tauri 命令封装（24 个）
│   │   ├── config.ts           # 前端配置状态管理
│   │   ├── consoleState.ts    # 控制台日志状态
│   │   ├── st_list.ts          # SillyTavern 实例列表
│   │   ├── theme.ts           # 主题切换
│   │   ├── cache.ts            # 缓存管理
│   │   ├── imageCache.ts       # 图片缓存
│   │   ├── updater.ts          # 自动更新
│   │   ├── useScan.ts          # 酒馆扫描逻辑（scanManager 单例）
│   │   ├── useInstall.ts       # 版本安装/删除
│   │   ├── useReleases.ts      # GitHub Releases 获取
│   │   ├── useExtensions.ts    # 扩展列表与状态
│   │   ├── useExtensionInstall.ts  # 扩展安装/修复/删除
│   │   ├── useOneClick.ts      # 一键安装流程
│   │   ├── useNodeGitInstall.ts  # Node.js / Git 安装进度
│   │   ├── useDialog.ts        # 全局弹窗控制
│   │   ├── useCharacterCardDialog.ts  # 角色卡弹窗
│   │   ├── useChatDialog.ts    # 对话历史弹窗
│   │   ├── useWorldInfoDialog.ts  # 世界书弹窗
│   │   ├── useUploadCharacterCard.ts  # 上传角色卡
│   │   ├── useUploadWorldInfo.ts  # 上传世界书
│   │   ├── useCheckpoint.ts   # 检查点管理
│   │   ├── useRepairGitDialog.ts  # Git 修复弹窗
│   │   ├── useAnimations.ts   # 动画与性能检测
│   │   └── index.ts            # lib 导出入口
│   ├── router/                 # Vue Router 路由配置
│   ├── views/                  # 页面视图
│   │   ├── Home.vue            # 主页（启动/停止/网络链接面板）
│   │   ├── Versions.vue        # 版本管理（本地实例扫描/在线下载）
│   │   ├── Tavern.vue          # 酒馆配置（config.yaml 可视化编辑器）
│   │   ├── Extensions.vue      # 扩展管理
│   │   ├── Resources.vue       # 资源管理（角色卡/世界书/对话历史）
│   │   ├── Console.vue         # 控制台日志
│   │   ├── Settings.vue        # 应用设置
│   │   ├── Tools.vue           # 鸣谢页面
│   │   └── App.vue             # 根组件
│   ├── App.vue                 # 根组件
│   ├── main.ts                 # 前端入口
│   ├── style.css               # 全局样式
│   └── tailwind.config.css     # Tailwind v4 配置
│
├── src-tauri/                  # 后端 Rust 源码 (Tauri)
│   ├── src/
│   │   ├── main.rs             # Rust 入口（Windows GUI 入口点）
│   │   ├── lib.rs              # 模块声明与 run()
│   │   ├── types.rs            # 所有公共类型/结构体/枚举定义
│   │   ├── utils.rs            # 日志、目录布局、tracing 子scriber
│   │   ├── config.rs           # AppConfig 读写、代理检测、窗口管理
│   │   ├── node.rs             # Node.js / npm 检测与安装
│   │   ├── git.rs              # Git 检测、内置 MinGit、GitHub 加速逻辑
│   │   ├── sillytavern.rs      # ST 版本管理、启停、YAML 配置读写
│   │   ├── extensions.rs       # 扩展管理（列表/启用/安装/删除/修复）
│   │   ├── character.rs        # 角色卡 PNG 管理
│   │   ├── worldinfo.rs        # 世界书 JSON 管理
│   │   ├── chat.rs             # 对话历史读取与删除
│   │   ├── finderst.rs         # SillyTavern 全盘扫描（jwalk + walkdir）
│   │   ├── logs.rs             # 应用运行日志读写
│   │   ├── elevation.rs        # Windows UAC 提权辅助
│   │   └── Cargo.toml          # Rust 依赖
│   ├── icons/                  # 应用图标（多平台）
│   └── tauri.conf.json         # Tauri 配置文件
│
├── data/                       # 运行时数据目录（自动生成）
│   ├── config.json             # 应用全局配置
│   ├── logs/                   # 应用运行日志（按天滚动）
│   ├── node/                   # 内置 Node.js 环境（可选）
│   ├── mingit/                 # 内置 PortableGit（可选）
│   ├── st_data/                # SillyTavern 全局数据
│   │   ├── config.yaml         # 全局酒馆配置
│   │   ├── characters/         # 角色卡目录
│   │   └── worlds/             # 世界书目录
│   └── sillytavern/            # 各版本酒馆安装目录
│       └── <version>/          # 具体版本，如 release-v1.12.0
│
├── scripts/                    # 构建辅助脚本
├── public/                     # Vite 静态资源
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── UPDATELOGS.md               # 版本更新记录
└── README.md
```

---

### 💻 开发环境搭建

#### 前置依赖

| 依赖                                                         | 版本要求          | 说明                                                            |
| ------------------------------------------------------------ | ----------------- | --------------------------------------------------------------- |
| [Rust](https://www.rust-lang.org/zh-CN/)                     | 1.80+             | 含 Cargo，Tauri 后端必须                                        |
| [Node.js](https://nodejs.org/zh-cn) / [Bun](https://bun.sh/) | Node 18+ / Bun 1+ | 推荐使用 Bun                                                    |
| Tauri 系统依赖                                               | —                 | [查看官方文档](https://v2.tauri.app/zh-cn/start/prerequisites/) |

> **Windows 用户**额外需要：安装 [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/zh-hans/visual-cpp-build-tools/) 或 Visual Studio。

#### 克隆与安装

```bash
# 克隆仓库
git clone https://github.com/al01cn/sillyTavern-launcher.git
cd sillyTavern-launcher

# 安装前端依赖（推荐使用 bun）
bun install
```

#### 开发模式

```bash
# 启动 Vite 热更新 + Tauri 桌面窗口
bun run tauri:dev
```

#### 生产构建

```bash
# 构建安装包（同时同步版本号）
bun run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下。

#### 其他命令

```bash
bun run cargo:check    # 仅检查 Rust 编译是否通过
bun run cargo:build    # 仅构建 Rust 后端
bun run sync-version   # 同步 package.json 版本号到 Tauri
```

---

### 🏗️ 架构说明

#### Rust 后端模块划分

```
main.rs           ← Rust 入口（Windows GUI 入口点）
lib.rs            ← 模块声明与 run()
  ├── types.rs       所有共享数据结构（类型/结构体/枚举，纯定义无逻辑）
  ├── utils.rs       日志初始化、目录布局、tracing subscriber
  ├── config.rs      AppConfig 读写、窗口管理、代理检测（PowerShell/reg）
  ├── node.rs        Node.js / npm 检测与安装
  ├── git.rs         Git 检测、内置 MinGit、GitHub 加速（--import / insteadOf）
  ├── sillytavern.rs ST 版本 CRUD、启停、YAML 配置读写
  ├── extensions.rs  扩展列表、启用/禁用/安装/删除/修复
  ├── character.rs   角色卡 PNG 管理
  ├── worldinfo.rs   世界书 JSON 管理
  ├── chat.rs        对话历史读取与删除
  ├── finderst.rs    SillyTavern 全盘扫描（jwalk + walkdir 双引擎）
  ├── logs.rs        应用运行日志读写
  └── elevation.rs   Windows UAC 提权辅助
```

#### 前后端通信

所有前端操作均通过 **Tauri Command**（`invoke`）调用 Rust 后端，前端不直接发起外部网络请求，确保无跨域问题。

```
前端 Vue (invoke)  →  Rust Command  →  系统/文件/网络
```

长耗时操作（下载、解压、npm 安装）通过 **Tauri Event**（`emit`）向前端推送进度：

| 事件名              | 说明                     |
| ------------------- | ------------------------ |
| `install-progress`  | 版本安装/删除进度        |
| `download-progress` | Node.js 下载/解压进度    |
| `process-log`       | SillyTavern 进程实时日志 |
| `process-exit`      | SillyTavern 进程退出通知 |

---

### 📝 开发规范

- **组合式 API**：始终使用 Vue 3 `<script setup>` + TypeScript 严格模式。
- **UI 组件**：统一使用 [FluxUI](https://fluxui.dev/) 组件库（`flux:card`、`flux:modal` 等），配合 TailwindCSS 工具类。
- **图标**：统一使用 `lucide-vue-next`、`@phosphor-icons/vue` 或 `@iconify/vue`，禁止使用 emoji。
- **悬浮提示**：使用 `AppTooltip` 组件，prop 名统一为 `:text=`，组件会自动处理 Teleport 定位。
- **后端交互**：所有系统操作、网络请求必须经由 Rust 后端处理，前端仅负责展示。
- **跨平台**：Windows / macOS / Linux 三平台均需兼容，使用条件编译处理平台差异。
- **包管理**：前端使用 `bun`，Rust 侧使用 `cargo`。
- **lint**：提交前运行 `bun run allcheck`（typecheck + eslint + cargo fmt + cargo check）。

---

### 🤝 参与贡献

欢迎通过以下方式参与贡献：

1. **提交 Issue**：报告 Bug 或提出新功能建议
2. **提交 PR**：Fork 后在新分支开发，PR 前请确保 `cargo check` 无错误
3. **完善文档**：改进 README 或添加使用说明

### 👥 贡献者

感谢所有为本项目做出贡献的朋友！

<a href="https://github.com/al01cn/sillyTavern-launcher/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=al01cn/sillyTavern-launcher" alt="Contributors" />
</a>

---

### 💖 鸣谢

本项目的开发离不开以下开源项目的支持：

- **核心框架**: [Tauri](https://tauri.app/), [Vue.js](https://vuejs.org/), [Rust](https://www.rust-lang.org/)
- **UI & 样式**: [TailwindCSS](https://tailwindcss.com/), [FluxUI](https://fluxui.dev/)
- **图标库**: [Lucide Icons](https://lucide.dev/), [Phosphor Icons](https://phosphoricons.com/), [Iconify](https://iconify.design/)
- **功能插件**: [vue-i18n](https://vue-i18n.intlify.dev/), [vue-sonner](https://vue-sonner.vercel.app/), [reqwest](https://github.com/seanmonstar/reqwest), [tokio](https://tokio.rs/)
- **特别鸣谢**: 感谢 [SillyTavern](https://github.com/SillyTavern/SillyTavern) 项目团队及其社区提供的出色工具。

---

### 📄 开源许可

本项目遵循 [MIT License](LICENSE) 开源许可。

---

## 📖 English Documentation

### Introduction

**SillyTavern Launcher GUI** is a cross-platform desktop client built with **Tauri v2 + Vue 3 + Rust**, designed specifically for [SillyTavern](https://github.com/SillyTavern/SillyTavern). It provides a modern graphical interface to fully manage SillyTavern without using the command line.

> 🌐 Website: [st.al01.cn](https://st.al01.cn/)

---

### ✨ Features

| Module                   | Description                                                              |
| ------------------------ | ------------------------------------------------------------------------ |
| 🚀 **One-Click Launch**  | Start/stop SillyTavern instantly with real-time console log streaming    |
| 📦 **Version Manager**   | Install, switch, or uninstall any SillyTavern version (stable / release) |
| 🧩 **Extension Manager** | Browse, enable/disable, install (from ZIP), and delete extensions        |
| ⚙️ **Tavern Config**     | Visual editor for `config.yaml` (port, proxy, whitelist, backups, etc.)  |
| 🌐 **Node.js Manager**   | Auto-detects system Node.js, supports one-click private Node.js install  |
| 🎭 **Character Cards**   | Browse, preview, import, and delete character PNG cards                  |
| 📚 **World Info**        | Browse, import, and delete World Info JSON files                         |
| 🔄 **Auto Update**       | Silent updates via GitHub Releases (configurable reminder)               |
| 🌍 **i18n**              | Built-in Chinese / English with automatic system locale detection        |
| 🎨 **Themes**            | Dark / Light mode with system follow support                             |
| 🔧 **GitHub Proxy**      | Built-in multi-node acceleration for GitHub downloads                    |
| 💾 **Window Memory**     | Optionally remember last window position                                 |

---

### 🚀 Quick Start (End Users)

> If you just want to **use** the launcher (not develop), follow these steps:

1. Go to the [GitHub Releases](https://github.com/al01cn/sillyTavern-launcher/releases) page
2. Download the installer for your operating system:

   | OS      | File Format           |
   | ------- | --------------------- |
   | Windows | `.msi` or `.exe`      |
   | macOS   | `.dmg`                |
   | Linux   | `.AppImage` or `.deb` |

3. Run the installer and launch the app

> [!WARNING]
> **Windows users**: If the installer fails with a permission error, right-click the file and select **"Run as administrator"**, then try again.

---

### 🛠️ Tech Stack

| Layer               | Technology                                                                                              |
| ------------------- | ------------------------------------------------------------------------------------------------------- |
| **Frontend**        | [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vite.dev/) |
| **UI**              | [TailwindCSS v4](https://tailwindcss.com/) + [FluxUI](https://fluxui.dev/) (full component library)         |
| **Icons**           | Lucide Vue Next + Phosphor Icons + Iconify                                                              |
| **i18n**            | vue-i18n v11                                                                                            |
| **Backend**         | [Tauri v2](https://v2.tauri.app/) + [Rust](https://www.rust-lang.org/)                                  |
| **Key Rust crates** | `tokio`, `reqwest`, `serde`, `serde_yaml`, `zip`, `tracing`                                             |
| **Package manager** | [Bun](https://bun.sh/) (recommended)                                                                    |

---

### 📁 Project Structure

```text
SillyTavern-Launcher/
├── src/                        # Frontend Vue 3 source
│   ├── assets/                 # Static assets (images, logos)
│   ├── components/             # Reusable components (15 files)
│   │   ├── AppTooltip.vue      # Unified hover tooltip
│   │   ├── GlobalDialog.vue    # Global dialog container
│   │   ├── InstallDialog.vue   # Version install dialog
│   │   ├── InstallExtensionDialog.vue  # Extension install dialog
│   │   ├── CharacterCardDialog.vue    # Character card preview
│   │   ├── ChatDialog.vue      # Chat history dialog
│   │   ├── NetworkLinkDialog.vue       # LAN/public link panel
│   │   ├── OneClickCapsule.vue # One-click install capsule
│   │   ├── ConfigMigrateDialog.vue     # Config migration
│   │   ├── ResourceMigrateDialog.vue   # Resource migration
│   │   ├── RepairGitDialog.vue # Git repair dialog
│   │   ├── UploadCharacterCardDialog.vue  # Upload character card
│   │   ├── UploadWorldInfoDialog.vue  # Upload world info
│   │   ├── WorldInfoDialog.vue # World info preview
│   │   └── Model.vue           # FluxUI Modal wrapper
│   ├── lang/                   # i18n files (zh-CN / en-US)
│   ├── layouts/                # Layout (custom titlebar)
│   ├── lib/                    # Utilities & Tauri command wrappers (24 files)
│   │   ├── config.ts          # Frontend config state management
│   │   ├── consoleState.ts    # Console log state
│   │   ├── st_list.ts         # SillyTavern instance list
│   │   ├── theme.ts           # Theme switching
│   │   ├── cache.ts           # Cache management
│   │   ├── imageCache.ts      # Image cache
│   │   ├── updater.ts         # Auto updater
│   │   ├── useScan.ts         # Tavern scanning (scanManager singleton)
│   │   ├── useInstall.ts      # Version install/uninstall
│   │   ├── useReleases.ts     # GitHub Releases fetch
│   │   ├── useExtensions.ts   # Extension list & state
│   │   ├── useExtensionInstall.ts  # Extension install/repair/delete
│   │   ├── useOneClick.ts     # One-click install flow
│   │   ├── useNodeGitInstall.ts  # Node.js / Git install progress
│   │   ├── useDialog.ts       # Global dialog control
│   │   ├── useCharacterCardDialog.ts  # Character card dialog
│   │   ├── useChatDialog.ts   # Chat history dialog
│   │   ├── useWorldInfoDialog.ts  # World info dialog
│   │   ├── useUploadCharacterCard.ts  # Upload character card
│   │   ├── useUploadWorldInfo.ts  # Upload world info
│   │   ├── useCheckpoint.ts   # Checkpoint management
│   │   ├── useRepairGitDialog.ts  # Git repair dialog
│   │   ├── useAnimations.ts   # Animation & perf detection
│   │   └── index.ts           # lib export entry
│   ├── router/                 # Vue Router
│   ├── views/                 # Page views
│   │   ├── Home.vue           # Dashboard (start/stop/network panel)
│   │   ├── Versions.vue       # Version manager (local scan / online)
│   │   ├── Tavern.vue         # Tavern config (config.yaml editor)
│   │   ├── Extensions.vue     # Extension manager
│   │   ├── Resources.vue      # Resources (characters/worlds/chat history)
│   │   ├── Console.vue        # Process log viewer
│   │   ├── Settings.vue       # App settings
│   │   ├── Tools.vue          # Acknowledgements
│   │   └── App.vue            # Root component
│   ├── App.vue                # Root component
│   ├── main.ts                # Frontend entry
│   ├── style.css              # Global styles
│   └── tailwind.config.css    # Tailwind v4 config
│
├── src-tauri/                  # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs            # Rust entry (Windows GUI entry point)
│   │   ├── lib.rs             # Module declarations & run()
│   │   ├── types.rs           # All shared types/structs/enums
│   │   ├── utils.rs          # Logger, layout, tracing subscriber
│   │   ├── config.rs         # AppConfig R/W, proxy detection, window mgmt
│   │   ├── node.rs           # Node.js / npm detection & install
│   │   ├── git.rs            # Git detection, bundled MinGit, GH accel
│   │   ├── sillytavern.rs    # ST version CRUD, start/stop, YAML config
│   │   ├── extensions.rs     # Extension mgmt (list/enable/install/delete)
│   │   ├── character.rs      # Character card PNG management
│   │   ├── worldinfo.rs       # World info JSON management
│   │   ├── chat.rs           # Chat history read & delete
│   │   ├── finderst.rs       # SillyTavern full-disk scan
│   │   ├── logs.rs           # App log read/write
│   │   ├── elevation.rs      # Windows UAC elevation helper
│   │   └── Cargo.toml        # Rust dependencies
│   ├── icons/                 # App icons (multi-platform)
│   └── tauri.conf.json       # Tauri config
│
└── data/                       # Runtime data (auto-created)
    ├── config.json            # App configuration
    ├── logs/                  # Daily rolling logs
    ├── node/                  # (Optional) bundled Node.js
    ├── mingit/                # (Optional) bundled PortableGit
    ├── st_data/               # Global SillyTavern data
    │   ├── config.yaml        # Global tavern config
    │   ├── characters/        # Character cards
    │   └── worlds/            # World info files
    └── sillytavern/           # Installed ST versions
        └── <version>/         # e.g. release-v1.12.0
│
├── scripts/                    # Build scripts
├── public/                     # Vite static assets
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── eslint.config.js
├── UPDATELOGS.md               # Changelog
└── README.md
```

---

### 💻 Development Setup

#### Prerequisites

| Dependency                                               | Required          | Notes                                                                |
| -------------------------------------------------------- | ----------------- | -------------------------------------------------------------------- |
| [Rust](https://www.rust-lang.org/)                       | 1.80+             | Includes Cargo                                                       |
| [Node.js](https://nodejs.org/) or [Bun](https://bun.sh/) | Node 18+ / Bun 1+ | Bun recommended                                                      |
| Tauri system deps                                        | —                 | See [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/) |

#### Clone & Install

```bash
git clone https://github.com/al01cn/sillyTavern-launcher.git
cd sillyTavern-launcher
bun install
```

#### Development

```bash
bun run tauri:dev
```

#### Production Build

```bash
bun run tauri:build
```

Build output: `src-tauri/target/release/bundle/`

---

### 🏗️ Architecture

#### Rust Backend Modules

```
main.rs           ← Rust entry (Windows GUI entry point)
lib.rs            ← Module declarations & run()
  ├── types.rs       All shared types/structs/enums
  ├── utils.rs       Logger init, layout, tracing subscriber
  ├── config.rs      AppConfig R/W, window mgmt, proxy detection
  ├── node.rs        Node.js / npm detection & install
  ├── git.rs         Git detection, bundled MinGit, GH acceleration
  ├── sillytavern.rs ST version CRUD, start/stop, YAML config
  ├── extensions.rs  Extension list/enable/install/delete/repair
  ├── character.rs   Character card PNG management
  ├── worldinfo.rs   World info JSON management
  ├── chat.rs        Chat history read & delete
  ├── finderst.rs    Full-disk ST scan (jwalk + walkdir dual-engine)
  ├── logs.rs        App log read/write
  └── elevation.rs   Windows UAC elevation helper
```

All frontend operations communicate with the Rust backend via **Tauri Commands** (`invoke`). Long-running tasks (download, extract, npm install) emit progress events back to the frontend:

| Event                | Description                                |
| -------------------- | ------------------------------------------ |
| `install-progress`   | ST version install/delete progress        |
| `download-progress`  | Node.js / Git download/extract progress    |
| `process-log`        | SillyTavern real-time stdout/stderr       |
| `process-exit`       | SillyTavern process exit notification      |
| `repair-missing-deps`| 运行时检测到 node_modules 缺包，触发自动修复 |

---

### 🤝 Contributing

1. **Issues**: Report bugs or suggest features via [GitHub Issues](https://github.com/al01cn/sillyTavern-launcher/issues)
2. **Pull Requests**: Fork → new branch → develop → `cargo check` → PR
3. **Docs**: Improvements to README or usage guides are welcome

Before committing, run `bun run allcheck` (typecheck + eslint + cargo fmt + cargo check).

### 👥 Contributors

Thanks to all contributors who have helped build this project!

<a href="https://github.com/al01cn/sillyTavern-launcher/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=al01cn/sillyTavern-launcher" alt="Contributors" />
</a>


---

### 💖 Credits & Acknowledgements

This project would not be possible without these amazing open-source projects:

- **Core Frameworks**: [Tauri](https://tauri.app/), [Vue.js](https://vuejs.org/), [Rust](https://www.rust-lang.org/)
- **UI & Styling**: [TailwindCSS](https://tailwindcss.com/), [FluxUI](https://fluxui.dev/)
- **Icon Libraries**: [Lucide Icons](https://lucide.dev/), [Phosphor Icons](https://phosphoricons.com/), [Iconify](https://iconify.design/)
- **Plugins & Libraries**: [vue-i18n](https://vue-i18n.intlify.dev/), [vue-sonner](https://vue-sonner.vercel.app/), [reqwest](https://github.com/seanmonstar/reqwest), [tokio](https://tokio.rs/)
- **Special Thanks**: Huge thanks to the [SillyTavern](https://github.com/SillyTavern/SillyTavern) team and their community for creating such a wonderful tool.

---

### 📄 License

This project is open-source under the [MIT License](LICENSE).
