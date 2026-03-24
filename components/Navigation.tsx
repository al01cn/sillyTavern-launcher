'use client';

import { Moon, Sun, Github, MessageCircle, GitBranch } from 'lucide-react';
import { Lang } from '@/lib/types';
import LanguageSwitcher from './LanguageSwitcher';
import { config } from '@/lib/config';

interface NavigationProps {
  t: any;
  lang: Lang;
  isDark: boolean;
  onToggleTheme: () => void;
}

export default function Navigation({ t, lang, isDark, onToggleTheme }: NavigationProps) {
  return (
    <nav className="fixed top-0 w-full z-50 bg-white/80 dark:bg-[#050505]/60 backdrop-blur-xl border-b border-black/5 dark:border-white/5 transition-colors duration-300">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-16">
          <div className="flex items-center space-x-3">
            <img src="/favicon.ico" className='w-10 h-10' alt="logo" />
            <span className="font-bold text-xl tracking-tight text-slate-900 dark:text-white">{t.navTitle}</span>
          </div>
          <div className="flex items-center space-x-2 sm:space-x-4">
            <LanguageSwitcher currentLocale={lang} />
            <button
              onClick={onToggleTheme}
              className="p-2 rounded-full text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
              aria-label="Toggle Theme"
            >
              {isDark ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
            </button>
            <a
              href={config.git.github}
              target="_blank"
              rel="noreferrer"
              className="p-2 rounded-full text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            >
              <Github className="w-5 h-5" />
            </a>
            <a
              href={config.git.gitee}
              target="_blank"
              rel="noreferrer"
              className="p-2 rounded-full text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
            >
              <GitBranch className="w-5 h-5" />
            </a>
          </div>
        </div>
      </div>
    </nav>
  );
}
