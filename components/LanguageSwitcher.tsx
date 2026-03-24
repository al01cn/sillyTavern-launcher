'use client';

import { usePathname, useRouter } from 'next/navigation';
import { Languages } from 'lucide-react';
import { LOCALES, getRoutePrefix, type Locale } from '@/lib/i18n-config';

interface LanguageSwitcherProps {
  currentLocale: Locale;
}

export default function LanguageSwitcher({ currentLocale }: LanguageSwitcherProps) {
  const pathname = usePathname();
  const router = useRouter();

  const handleLocaleChange = (newLocale: Locale) => {
    if (newLocale === currentLocale) return;

    // 获取路由前缀
    const routePrefix = getRoutePrefix(newLocale);
    
    // 移除路径中的旧语言前缀
    const cleanPathname = pathname.replace(/^\/(zh-CN|en-US|zh|en)/, '');
    
    // 构建新的路径
    let newPathname = cleanPathname;
    if (newLocale !== 'zh') {
      newPathname = `/${routePrefix}${cleanPathname}`;
    }
    
    // 如果路径没有变化，添加根路径
    if (newPathname === '') {
      newPathname = newLocale === 'zh' ? '/' : `/${routePrefix}`;
    }
    
    router.push(newPathname);
  };

  return (
    <div className="relative group">
      <button
        className="p-2 rounded-full text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
        aria-label="Switch Language"
      >
        <Languages className="w-5 h-5" />
      </button>
      
      {/* 语言选择下拉菜单 */}
      <div className="absolute right-0 mt-2 w-32 bg-white dark:bg-[#0a0a0a] rounded-xl shadow-lg border border-black/10 dark:border-white/10 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50">
        <div className="py-1">
          {(Object.keys(LOCALES) as Locale[]).map((locale) => (
            <button
              key={locale}
              onClick={() => handleLocaleChange(locale)}
              className={`w-full text-left px-4 py-2 text-sm transition-colors ${
                currentLocale === locale
                  ? 'bg-[#C8102E]/10 text-[#C8102E] dark:text-[#ff4d6d] font-medium'
                  : 'text-slate-700 dark:text-slate-300 hover:bg-black/5 dark:hover:bg-white/5'
              }`}
            >
              {LOCALES[locale].name}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
