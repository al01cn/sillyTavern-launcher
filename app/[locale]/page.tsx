import { notFound } from 'next/navigation';
import { getSeoConfig, getAlternateLinks, getCanonicalUrl } from '@/lib/seo';
import type { Metadata } from 'next';
import HomeContent from '@/app/[locale]/HomeContent';

interface LocalePageProps {
  params: Promise<{
    locale: string;
  }>;
}

// 生成 SEO 元数据
export async function generateMetadata({ params }: LocalePageProps): Promise<Metadata> {
  const { locale } = await params;
  const validLocales = ['zh', 'en'] as const;
  
  if (!validLocales.includes(locale as 'zh' | 'en')) {
    return {};
  }
  
  const seo = getSeoConfig(locale as 'zh' | 'en');
  const canonicalUrl = getCanonicalUrl(`/${locale}`);
  const alternateLinks = getAlternateLinks(`/${locale}`);
  
  return {
    title: seo.title,
    description: seo.description,
    keywords: seo.keywords,
    openGraph: {
      ...seo.openGraph,
      url: canonicalUrl,
    },
    twitter: seo.twitter,
    alternates: {
      canonical: canonicalUrl,
      languages: {
        'zh-CN': `${process.env.NEXT_PUBLIC_SITE_URL}/`,
        'en-US': `${process.env.NEXT_PUBLIC_SITE_URL}/en`,
      },
    },
  };
}

// 渲染页面
export default async function LocalePage({ params }: LocalePageProps) {
  const { locale } = await params;
  const validLocales = ['zh', 'en'] as const;
  
  // 验证语言参数
  if (!validLocales.includes(locale as 'zh' | 'en')) {
    notFound();
  }
  
  // 在服务端确定语言，直接传递给客户端组件
  return <HomeContent initialLocale={locale as 'zh' | 'en'} />;
}
