'use client';

import { useState, useEffect, useRef } from 'react';
import { Download, ChevronRight, Cpu, X } from 'lucide-react';
import { Release, formatBytes, getPlatformIconName } from '@/lib/types';

interface DownloadSectionProps {
  t: any;
  releases: Release[];
  loading: boolean;
  useChinaMirror: boolean;
  onToggleChinaMirror: () => void;
}

export default function DownloadSection({ t, releases, loading, useChinaMirror, onToggleChinaMirror }: DownloadSectionProps) {
  const [showChangelog, setShowChangelog] = useState(false);
  const latestRelease = releases.length > 0 ? releases[0] : null;
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  // 当弹窗打开时，禁止 body 滚动
  useEffect(() => {
    if (showChangelog) {
      document.body.style.overflow = 'hidden';
      // 聚焦到关闭按钮
      setTimeout(() => closeButtonRef.current?.focus(), 100);
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [showChangelog]);

  // 处理 Tab 键，限制焦点在弹窗内
  useEffect(() => {
    if (!showChangelog) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape 键关闭弹窗
      if (e.key === 'Escape') {
        setShowChangelog(false);
        return;
      }

      // Tab 键焦点循环
      if (e.key !== 'Tab') return;

      const modal = document.getElementById('changelog-modal');
      if (!modal) return;

      const focusableElements = modal.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      
      const firstElement = focusableElements[0] as HTMLElement;
      const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

      if (e.shiftKey) {
        // Shift + Tab
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      } else {
        // Tab
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [showChangelog]);

  const getPlatformIcon = (filename: string) => {
    const iconName = getPlatformIconName(filename);
    // 这里可以返回不同的图标，根据实际需要调整
    return <Download className="w-5 h-5" />;
  };

  return (
    <section id="download" className="relative z-10 py-32 px-4 sm:px-6 lg:px-8 max-w-5xl mx-auto">
      {/* 更新日志弹窗 */}
      {showChangelog && latestRelease && (
        <div 
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm" 
          onClick={() => setShowChangelog(false)}
          role="dialog"
          aria-modal="true"
          aria-labelledby="changelog-title"
        >
          <div 
            id="changelog-modal"
            className="bg-white dark:bg-[#0a0a0a] rounded-3xl border border-black/10 dark:border-white/10 shadow-2xl max-w-3xl w-full max-h-[80vh] overflow-hidden relative"
            onClick={(e) => e.stopPropagation()}
          >
            {/* 弹窗头部 */}
            <div className="flex items-center justify-between p-6 border-b border-black/5 dark:border-white/5">
              <div>
                <h3 className="text-xl font-bold text-slate-900 dark:text-white">
                  {latestRelease.name || latestRelease.tag_name}
                </h3>
                <p className="text-xs font-mono text-slate-500 mt-1">
                  {t.published}: {new Date(latestRelease.published_at).toLocaleDateString('zh-CN')}
                </p>
              </div>
              <button
                ref={closeButtonRef}
                onClick={() => setShowChangelog(false)}
                className="p-2 rounded-full hover:bg-black/5 dark:hover:bg-white/5 transition-colors text-slate-500 dark:text-slate-400 focus:outline-none focus:ring-2 focus:ring-[#C8102E]/50"
                aria-label="Close changelog"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            
            {/* 弹窗内容 - 使用 dangerouslySetInnerHTML 渲染 Markdown */}
            <div className="p-6 overflow-y-auto max-h-[calc(80vh-120px)]">
              <div 
                className="prose dark:prose-invert max-w-none"
                dangerouslySetInnerHTML={{ 
                  __html: latestRelease.body 
                    ? latestRelease.body
                        .replace(/^## /gm, '<h2 class="text-lg font-bold mt-6 mb-3 text-slate-900 dark:text-white">')
                        .replace(/^### /gm, '<h3 class="text-base font-semibold mt-4 mb-2 text-slate-800 dark:text-slate-200">')
                        .replace(/^- /gm, '<li class="ml-4 text-slate-600 dark:text-slate-400">')
                        .replace(/^\* /gm, '<li class="ml-4 text-slate-600 dark:text-slate-400">')
                        .replace(/\n\n/g, '</p><p class="mb-4">')
                        .replace(/\n/g, '<br/>')
                    : '<p class="text-slate-500">No changelog available</p>'
                }}
              />
            </div>
          </div>
        </div>
      )}

      <div className="download-section text-center mb-12">
        <h2 className="text-3xl md:text-4xl font-bold mb-4 text-slate-900 dark:text-white tracking-tight">{t.downloadTitle}</h2>
        <p className="text-slate-500 dark:text-slate-400 font-mono text-sm uppercase tracking-widest">{t.downloadSub}</p>
      </div>

      <div className="download-section bg-white dark:bg-[#0a0a0a] rounded-3xl border border-black/10 dark:border-white/10 overflow-hidden shadow-2xl relative transition-colors duration-300">
        {/* Decorative top bar */}
        <div className="h-1 w-full bg-gradient-to-r from-[#C8102E] via-[#FFC72C] to-[#00A651]"></div>
        
        {loading ? (
          <div className="p-16 text-center flex flex-col items-center justify-center">
            <Cpu className="w-8 h-8 text-[#C8102E] animate-pulse mb-4" />
            <p className="text-slate-500 dark:text-slate-400 font-mono text-sm animate-pulse">{t.fetching}</p>
          </div>
        ) : releases.length > 0 ? (
          <div>
            <div className="bg-black/[0.02] dark:bg-white/[0.02] p-6 sm:p-8 border-b border-black/5 dark:border-white/5 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
              <div>
                <div className="flex items-center space-x-3 mb-2">
                  <h3 className="text-2xl font-bold text-slate-900 dark:text-white tracking-tight">{releases[0].name || releases[0].tag_name}</h3>
                  <span className="bg-[#00A651]/10 border border-[#00A651]/20 text-[#00A651] text-[10px] px-2.5 py-1 rounded-full font-mono uppercase tracking-wider">
                    {t.latest}
                  </span>
                </div>
                <p className="text-xs font-mono text-slate-500">
                  {t.published}: {new Date(releases[0].published_at).toISOString().split('T')[0]}
                </p>
              </div>
              <div className="flex flex-col sm:flex-row items-start sm:items-center gap-4">
                <div className="flex items-center space-x-2 bg-black/5 dark:bg-white/5 border border-black/10 dark:border-white/10 px-3 py-1.5 rounded-lg">
                  <span className="text-xs font-mono text-slate-600 dark:text-slate-400 uppercase">{t.chinaMirror}</span>
                  <button
                    onClick={onToggleChinaMirror}
                    className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none ${useChinaMirror ? 'bg-[#00A651]' : 'bg-slate-300 dark:bg-slate-700'}`}
                    aria-label="Toggle China Mirror"
                  >
                    <span className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform ${useChinaMirror ? 'translate-x-5' : 'translate-x-1'}`} />
                  </button>
                </div>
                <button
                  onClick={() => setShowChangelog(true)}
                  className="text-sm font-mono text-[#C8102E] dark:text-[#ff4d6d] hover:text-[#a00d24] dark:hover:text-[#ff7b93] flex items-center group transition-colors cursor-pointer"
                >
                  {t.viewChangelog} <ChevronRight className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" />
                </button>
              </div>
            </div>
            
            <div className="divide-y divide-black/5 dark:divide-white/5">
              {releases[0].assets
                .filter((asset) => !asset.name.endsWith('.sig') && asset.name !== 'latest.json')
                .map((asset) => (
                <div key={asset.name} className="p-4 sm:p-6 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors group">
                  <div className="flex items-center space-x-4">
                    <div className="w-12 h-12 bg-black/5 dark:bg-white/5 rounded-xl flex items-center justify-center text-slate-500 dark:text-slate-400 border border-black/5 dark:border-white/5 group-hover:border-black/10 dark:group-hover:border-white/10 group-hover:text-slate-900 dark:group-hover:text-white transition-colors">
                      {getPlatformIcon(asset.name)}
                    </div>
                    <div>
                      <p className="font-medium text-slate-800 dark:text-slate-200 break-all font-mono text-sm mb-1">{asset.name}</p>
                      <p className="text-xs font-mono text-slate-500">{t.size}: {formatBytes(asset.size)}</p>
                    </div>
                  </div>
                  <a
                    href={useChinaMirror ? `https://ghfast.top/${asset.browser_download_url}` : asset.browser_download_url}
                    className="w-full sm:w-auto flex items-center justify-center space-x-2 bg-black/5 hover:bg-black/10 dark:bg-white/10 dark:hover:bg-white/20 text-slate-800 dark:text-white px-6 py-2.5 rounded-xl font-medium transition-all border border-black/5 hover:border-black/20 dark:border-white/5 dark:hover:border-white/20"
                  >
                    <Download className="w-4 h-4" />
                    <span className="font-mono text-sm">{t.download}</span>
                  </a>
                </div>
              ))}
              {releases[0].assets.filter((asset) => !asset.name.endsWith('.sig') && asset.name !== 'latest.json').length === 0 && (
                <div className="p-12 text-center font-mono text-sm text-slate-500 uppercase">
                  {t.noAssets}
                </div>
              )}
            </div>
          </div>
        ) : (
          <div className="p-12 text-center font-mono text-sm text-slate-500 uppercase">
            {t.noData}
          </div>
        )}
      </div>
    </section>
  );
}
