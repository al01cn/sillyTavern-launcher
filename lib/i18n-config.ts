// 国际化配置常量
export const LOCALES = {
  zh: {
    code: 'zh-CN',
    name: '中文',
    path: '', // 默认语言，路径为空
  },
  en: {
    code: 'en-US',
    name: 'English',
    path: '/en',
  },
} as const;

// 语言类型
export type Locale = keyof typeof LOCALES;

// 获取所有支持的语言列表
export const getSupportedLocales = () => Object.keys(LOCALES) as Locale[];

// 根据语言代码获取 locale
export const getLocaleByCode = (code: string): Locale | null => {
  if (code === 'zh-CN' || code === 'zh') return 'zh';
  if (code === 'en-US' || code === 'en') return 'en';
  return null;
};

// 根据路径获取 locale
export const getLocaleByPath = (path: string): Locale => {
  if (path === '/en' || path === '/en-US') return 'en';
  return 'zh'; // 默认为中文
};

// 获取当前语言的完整信息
export const getLocaleInfo = (locale: Locale) => {
  return LOCALES[locale];
};

// 获取语言的路由前缀（用于 URL）
export const getRoutePrefix = (locale: Locale): string => {
  const localeInfo = LOCALES[locale];
  // 使用语言代码的前半部分作为路由前缀（en-US → en）
  return localeInfo.code.split('-')[0];
};

// 生成带语言前缀的路径
export const getLocalizedPath = (path: string, locale: Locale): string => {
  const localeInfo = LOCALES[locale];
  if (locale === 'zh') {
    return path; // 中文是默认语言，不需要前缀
  }
  return `/${localeInfo.code}${path}`;
};

// 从路径中提取语言
export const extractLocaleFromPath = (pathname: string): Locale => {
  const segments = pathname.split('/').filter(Boolean);
  if (segments[0] === 'en' || segments[0] === 'en-US') {
    return 'en';
  }
  return 'zh';
};
