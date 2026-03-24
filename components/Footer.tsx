'use client';

import { Github, ShieldCheck, MessageCircle, GitBranch } from 'lucide-react';
import { config } from '@/lib/config';
import type { Lang } from '@/lib/types';

interface FooterProps {
  t: any;
  lang: Lang;
}

export default function Footer({ t, lang }: FooterProps) {
  const isCN = lang === 'zh';
  
  return (
    <footer className="relative z-10 bg-slate-100 dark:bg-black border-t border-black/5 dark:border-white/5 py-12 transition-colors duration-300">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row justify-between items-center gap-6">
        <div className="flex items-center space-x-3">
          <img src="/favicon.ico" className='w-10 h-10' alt="logo" />
          <span className="font-semibold text-slate-900 dark:text-white tracking-tight">{t.navTitle}</span>
        </div>
        
        <div className="flex flex-col items-center md:items-start space-y-2">
          <p className="text-slate-500 font-mono text-xs text-center md:text-left">
            &copy; {new Date().getFullYear()} {t.footerRights}
          </p>
          {isCN && (
            <>
              <a href={config.qq.url} target="_blank" rel="noreferrer" className="flex items-center space-x-1 text-slate-400 hover:text-blue-600 dark:hover:text-blue-400 transition-colors text-xs mb-2">
                <MessageCircle className="w-3.5 h-3.5" />
                <span>QQ 交流群：{config.qq.value}</span>
              </a>
              <div className="flex flex-col sm:flex-row items-center gap-2 sm:gap-4 text-xs font-mono text-slate-400">
                <a href={config.icp.url} target="_blank" rel="noreferrer" className="flex items-center hover:text-slate-600 dark:hover:text-slate-300 transition-colors">
                  <ShieldCheck className="w-3 h-3 mr-1" />
                  {config.icp.value}
                </a>
                <span className="hidden sm:inline text-slate-300 dark:text-slate-700">|</span>
                <a href={config.gongan.url} target="_blank" rel="noreferrer" className="flex items-center hover:text-slate-600 dark:hover:text-slate-300 transition-colors">
                  <ShieldCheck className="w-3 h-3 mr-1" />
                  {config.gongan.value}
                </a>
              </div>
            </>
          )}
        </div>
        
        <div className="flex space-x-4">
          <a href={config.git.github} target="_blank" rel="noreferrer" className="text-slate-500 hover:text-slate-900 dark:hover:text-white transition-colors">
            <span className="sr-only">GitHub</span>
            <Github className="w-5 h-5" />
          </a>
          <a href={config.git.gitee} target="_blank" rel="noreferrer" className="text-slate-500 hover:text-slate-900 dark:hover:text-white transition-colors">
            <span className="sr-only">Gitee</span>
            <GitBranch className="w-5 h-5" />
          </a>
        </div>
      </div>
    </footer>
  );
}
