# SillyTavern Launcher Web

专为 SillyTavern 打造的现代化启动器 Web 界面。基于 Next.js 构建，支持多语言国际化，提供优雅的 UI/UX 体验。

📦 **代码仓库**: 
- [GitHub](https://github.com/al01cn/sillyTavern-launcher)
- [Gitee](https://gitee.com/al01/sillytavern-launcher)

## ✨ 特性

- 🚀 **现代化架构** - 基于 Next.js 15 + React 19，性能卓越
- 🌍 **国际化支持** - 内置中文/英文双语支持
- 🎨 **精美 UI 设计** - 使用 Tailwind CSS 和 Framer Motion 动画
- 📱 **响应式布局** - 完美适配桌面端和移动端
- 🌓 **主题切换** - 支持亮色/暗色主题无缝切换
- 🔍 **SEO 优化** - 完善的搜索引擎优化配置
- ♿ **可访问性** - 遵循无障碍设计原则

## 🛠️ 技术栈

- **框架**: Next.js 15 (App Router)
- **语言**: TypeScript 5.9
- **样式**: Tailwind CSS 4.1
- **动画**: Framer Motion, GSAP
- **图标**: Lucide React
- **包管理器**: Bun / npm

## 📦 快速开始

### 前置要求

- Node.js 20+ (推荐使用 [fnm](https://github.com/Schniz/fnm) 或 [nvm](https://github.com/nvm-sh/nvm))
- Bun (可选，更快的包管理)

### 安装步骤

1. **克隆项目**
   ```bash
   git clone https://github.com/al01cn/sillyTavern-launcher.git
   cd sillyTavern-launcher/web
   ```

2. **安装依赖**
   ```bash
   # 使用 npm
   npm install
   
   # 或使用 Bun (推荐)
   bun install
   ```

3. **配置环境变量**
   ```bash
   # 复制环境变量示例文件
   cp .env.local.example .env.local
   ```
   
   编辑 `.env.local` 文件，设置必要的 API 密钥。

4. **启动开发服务器**
   ```bash
   # 使用 npm
   npm run dev
   
   # 或使用 Bun
   bun dev
   ```

5. **访问应用**
   
   打开浏览器访问 [http://localhost:3000](http://localhost:3000)

## 📜 可用脚本

```bash
# 开发模式
npm run dev

# 构建生产版本
npm run build

# 启动生产服务器
npm run start

# 运行代码检查
npm run lint

# 清理缓存
npm run clean
```

## 🌐 国际化

本项目支持中文和英文两种语言。语言切换功能已集成在导航栏中。

- **中文 (默认)**: `/zh`
- **English**: `/en`

## 📁 项目结构

```
web/
├── app/                  # Next.js App Router 目录
│   ├── [locale]/        # 国际化路由
│   │   ├── page.tsx     # 主页
│   │   └── HomeContent.tsx
│   ├── layout.tsx       # 根布局
│   └── globals.css      # 全局样式
├── components/          # React 组件
│   ├── HeroSection.tsx
│   ├── FeaturesSection.tsx
│   ├── DownloadSection.tsx
│   ├── Navigation.tsx
│   ├── Footer.tsx
│   └── LanguageSwitcher.tsx
├── lib/                 # 工具函数和配置
│   ├── config.ts        # 网站配置
│   ├── i18n-config.ts   # 国际化配置
│   ├── seo.ts           # SEO 配置
│   └── types.ts         # TypeScript 类型
├── hooks/               # 自定义 React Hooks
├── public/              # 静态资源
└── middleware.ts        # Next.js 中间件
```

## ⚙️ 配置说明

### 网站配置

编辑 [`lib/config.ts`](lib/config.ts) 文件以自定义：

- 网站名称和 URL
- SEO 标题和描述
- 关键词
- 社交媒体链接
- 备案信息（如适用）

### 环境变量

参考 `.env.local.example` 文件配置必要的环境变量。

## 🚀 部署

### Vercel 部署（推荐）

1. 将代码推送到 GitHub
2. 在 [Vercel](https://vercel.com) 导入项目
3. 配置环境变量
4. 点击部署

### 手动部署

```bash
# 构建生产版本
npm run build

# 启动生产服务器
npm run start
```

## 📄 许可证

本项目采用 MIT 许可证。

## 👥 联系方式

- **QQ 群**: [1091959450](https://qm.qq.com/q/jy3viYcOre)
- **GitHub Issues**: [问题反馈](https://github.com/al01cn/sillyTavern-launcher/issues)

## 🙏 致谢

感谢所有为 SillyTavern 生态做出贡献的开发者！