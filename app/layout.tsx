import type { Metadata } from 'next';
import { Inter, JetBrains_Mono } from 'next/font/google';
import '../styles/globals.css';
import { config } from '@/lib/config';
import { getSeoConfig, getCanonicalUrl, getAlternateLinks } from '@/lib/seo';

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-sans',
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-mono',
});

// 默认使用中文 SEO 配置
const defaultSeo = getSeoConfig('zh');
const canonicalUrl = getCanonicalUrl('/');
const alternateLinks = getAlternateLinks('/');

export const metadata: Metadata = {
  title: defaultSeo.title,
  description: defaultSeo.description,
  keywords: defaultSeo.keywords,
  openGraph: {
    title: defaultSeo.openGraph.title,
    description: defaultSeo.openGraph.description,
    type: 'website',
    locale: defaultSeo.openGraph.locale,
    siteName: config.site.name,
  },
  twitter: {
    card: 'summary_large_image',
    title: defaultSeo.twitter.title,
    description: defaultSeo.twitter.description,
  },
  robots: {
    index: true,
    follow: true,
  },
  alternates: {
    canonical: canonicalUrl,
    languages: {
      'zh-CN': `${config.site.url}/`,
      'en-US': `${config.site.url}/en`,
    },
  },
  manifest: '/manifest.json',
  icons: {
    icon: '/favicon.ico',
    apple: '/apple-touch-icon.png',
  },
};

interface RootLayoutProps {
  children: React.ReactNode;
}

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="zh-CN" className={`${inter.variable} ${jetbrainsMono.variable}`} suppressHydrationWarning>
      <head>
        {/* 添加 hreflang 标签 */}
        {alternateLinks.map((link) => (
          <link
            key={link.hreflang}
            rel="alternate"
            hrefLang={link.hreflang}
            href={link.href}
          />
        ))}
        {/* 防止 FOUC（Flash of Unstyled Content） */}
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                const theme = localStorage.getItem('theme');
                const systemPrefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
                if (theme === 'dark' || (!theme && systemPrefersDark)) {
                  document.documentElement.classList.add('dark');
                }
              })();
            `,
          }}
        />
      </head>
      <body className="font-sans antialiased bg-black text-white" suppressHydrationWarning>{children}</body>
    </html>
  );
}
