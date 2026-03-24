import { config } from './config';
import type { Locale } from './i18n-config';

interface SeoMetadata {
  title: string;
  description: string;
  keywords: string[];
  openGraph: {
    title: string;
    description: string;
    locale: string;
  };
  twitter: {
    title: string;
    description: string;
  };
}

/**
 * 获取指定语言的 SEO 配置
 */
export const getSeoConfig = (locale: Locale = 'zh'): SeoMetadata => {
  const seoConfig = config[locale];
  const localeCode = locale === 'zh' ? 'zh-CN' : 'en-US';
  
  return {
    title: seoConfig.title,
    description: seoConfig.description,
    keywords: seoConfig.keywords,
    openGraph: {
      title: seoConfig.title,
      description: seoConfig.description,
      locale: localeCode,
    },
    twitter: {
      title: seoConfig.title,
      description: seoConfig.description,
    },
  };
};

/**
 * 生成完整的标题（带网站名称后缀）
 */
export const getFullTitle = (title?: string, locale: Locale = 'zh'): string => {
  const baseTitle = title || config[locale].title;
  return `${baseTitle} | ${config.site.name}`;
};

/**
 * 生成关键词字符串（用于 meta 标签）
 */
export const getKeywordsString = (locale: Locale = 'zh'): string => {
  return config[locale].keywords.join(', ');
};

/**
 * 生成规范链接（Canonical URL）
 */
export const getCanonicalUrl = (path: string, locale: Locale = 'zh'): string => {
  const baseUrl = config.site.url;
  const localePath = locale === 'en' ? '/en' : '';
  
  // 移除路径中的语言前缀
  const cleanPath = path.replace(/^\/(zh-CN|en-US|zh|en)/, '');
  
  return `${baseUrl}${localePath}${cleanPath || '/'}`;
};

/**
 * 生成所有语言的替代链接（hreflang）
 */
export const getAlternateLinks = (path: string) => {
  const baseUrl = config.site.url;
  const cleanPath = path.replace(/^\/(zh-CN|en-US|zh|en)/, '');
  
  return [
    {
      hreflang: 'zh-CN',
      href: `${baseUrl}${cleanPath || '/'}`,
    },
    {
      hreflang: 'en-US',
      href: `${baseUrl}/en${cleanPath || '/'}`,
    },
    {
      hreflang: 'x-default',
      href: `${baseUrl}${cleanPath || '/'}`,
    },
  ];
};
