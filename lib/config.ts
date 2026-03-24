export const config = {
  // 网站基本信息
  site: {
    name: "SillyTavern Launcher",
    url: process.env.NEXT_PUBLIC_SITE_URL || "http://localhost:3000", // 替换为你的实际域名
  },

  // SEO 配置 - 中文（默认）
  zh: {
    title: "酒馆启动器 GUI - 专为 SillyTavern 打造的现代化启动器",
    description: "专为 SillyTavern 打造的现代化启动器。在所有平台上无缝安装、更新和管理您的酒馆实例。",
    keywords: ["SillyTavern", "SillyTavern Launcher", "SillyTavern Launcher GUI", "酒馆启动器 GUI", "AI 角色扮演", "SillyTavern 安装", "SillyTavern 下载"],
  },

  // SEO 配置 - 英文
  en: {
    title: "SillyTavern Launcher GUI - Modernized launcher for SillyTavern.",
    description: "Modernized launcher for SillyTavern. Seamlessly install, update and manage your SillyTavern instances across all platforms.",
    keywords: ["SillyTavern", "SillyTavern Launcher", "SillyTavern Launcher GUI", "SillyTavern GUI", "AI role-playing", "SillyTavern installation", "SillyTavern download"],
  },
  git: {
    github: "https://github.com/al01cn/sillyTavern-launcher",
    gitee: "https://gitee.com/al01/sillytavern-launcher"
  },

  // 备案信息
  icp: {
    value: "粤 ICP 备 2025454179 号",
    url: "https://beian.miit.gov.cn/"
  },
  gongan: {
    value: "粤公网安备 44060502003974 号",
    url: "https://beian.mps.gov.cn/#/query/webSearch?code=44060502003974"
  },
  qq: {
    value: "1091959450",
    url: "https://qm.qq.com/q/jy3viYcOre"
  }
}
