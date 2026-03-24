// 翻译配置
export const translations = {
  en: {
    navTitle: "SillyTavern Launcher GUI",
    systemOnline: "SYSTEM ONLINE / PC BETA / MOBILE DEV",
    heroTitle1: "The best way to",
    heroTitle2: "manage SillyTavern",
    heroDesc: "A modern launcher built for SillyTavern. Seamlessly install, update, and manage your tavern instances across all platforms.",
    dlWindows: "Download for Windows",
    dlMac: "Download for macOS",
    dlLinux: "Download for Linux",
    dlUnknown: "OS Not Supported",
    dlOther: "Other Systems",
    mobileWait: "Mobile Coming Soon",
    featuresTitle: "Core Features",
    featuresSub: "Why use the Launcher?",
    feat1Title: "Cross-Platform Support",
    feat1Desc: "Whether you use Windows, macOS, or Linux, you get a consistent and smooth experience. Mobile coming soon.",
    feat2Title: "One-Click Deploy & Update",
    feat2Desc: "Say goodbye to tedious command lines. Install, start, and update your SillyTavern instance with a single click.",
    feat3Title: "Easy Configuration",
    feat3Desc: "Intuitive graphical interface lets you easily modify ports, environment variables, and advanced settings without manually editing config files.",
    downloadTitle: "Get Latest Release",
    downloadSub: "Download Latest Release",
    fetching: "FETCHING_RELEASES...",
    latest: "Latest",
    published: "PUBLISHED",
    chinaMirror: "China Mirror",
    viewChangelog: "VIEW_CHANGELOG",
    size: "SIZE",
    download: "DOWNLOAD",
    noAssets: "No assets available for this release.",
    noData: "Release data unavailable.",
    footerRights: "MTI LICENSE，AL01, copyright 2026",
    macosWarning: "macOS Compatibility Notice: This application has not been tested on actual macOS devices due to developer's hardware limitations. It may or may not work properly on macOS."
  },
  zh: {
    navTitle: "酒馆启动器GUI",
    systemOnline: "系统在线 / PC 测试版 / 移动端开发中",
    heroTitle1: "管理 SillyTavern",
    heroTitle2: "的最佳方式",
    heroDesc: "专为 SillyTavern 打造的现代化启动器。在所有平台上无缝安装、更新和管理您的酒馆实例。",
    dlWindows: "下载 Windows 版",
    dlMac: "下载 macOS 版",
    dlLinux: "下载 Linux 版",
    dlUnknown: "暂不支持当前系统",
    dlOther: "其他系统下载",
    mobileWait: "移动端敬请期待",
    featuresTitle: "核心特性",
    featuresSub: "核心特性",
    feat1Title: "全平台支持",
    feat1Desc: "无论您使用 Windows、macOS 还是 Linux，都能获得一致的流畅体验。移动端也即将在未来加入。",
    feat2Title: "一键部署与更新",
    feat2Desc: "告别繁琐的命令行操作，一键安装、启动和更新您的 SillyTavern 实例。",
    feat3Title: "便捷配置管理",
    feat3Desc: "直观的图形界面让您轻松修改端口、环境变量等高级设置，无需手动编辑配置文件。",
    downloadTitle: "获取最新版本",
    downloadSub: "获取最新版本",
    fetching: "获取发布信息...",
    latest: "最新版本",
    published: "发布时间",
    chinaMirror: "国内加速",
    viewChangelog: "查看更新日志",
    size: "大小",
    download: "下载",
    noAssets: "此版本没有可用的安装包。",
    noData: "无法获取发行版数据。",
    footerRights: "MTI 开源协议，灵狼 AL01, copyright 2026",
    macosWarning: "macOS 兼容性提示：由于开发者缺乏 macOS 设备，此应用未在真实的 macOS 设备上进行测试。在 macOS 上可能无法正常运行。"
  }
};

// 类型定义
export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

export interface Release {
  name: string;
  tag_name: string;
  published_at: string;
  body: string;
  html_url: string;
  assets: ReleaseAsset[];
}

export type OS = 'windows' | 'macos' | 'linux' | 'unknown';
export type Lang = 'zh' | 'en';

// 工具函数
export const formatBytes = (bytes: number, decimals = 2) => {
  if (!+bytes) return '0 Bytes';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
};

export const getPlatformIconName = (filename: string): 'win' | 'mac' | 'linux' | 'other' => {
  const lower = filename.toLowerCase();
  if (lower.includes('win') || lower.endsWith('.exe')) return 'win';
  if (lower.includes('mac') || lower.endsWith('.dmg')) return 'mac';
  if (lower.includes('linux') || lower.endsWith('.AppImage') || lower.endsWith('.deb')) return 'linux';
  return 'other';
};

export const detectOS = (): OS => {
  try {
    const userAgent = window.navigator.userAgent.toLowerCase();
    if (userAgent.includes('win')) return 'windows';
    if (userAgent.includes('mac')) return 'macos';
    if (userAgent.includes('linux') || userAgent.includes('x11')) return 'linux';
    return 'unknown';
  } catch (e) {
    console.error('Failed to detect OS', e);
    return 'unknown';
  }
};

export const shouldUseChinaMirror = (): boolean => {
  try {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    if (tz === 'Asia/Shanghai' || tz === 'Asia/Chongqing' || tz === 'Asia/Urumqi' || tz === 'Asia/Macau' || tz === 'Asia/Hong_Kong') {
      return true;
    }
  } catch (e) {
    console.error('Failed to detect timezone', e);
  }
  return false;
};
