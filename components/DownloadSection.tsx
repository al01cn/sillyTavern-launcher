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
  pcReleases?: Release[];
  pcLoading?: boolean;
}

export default function DownloadSection({ 
  t, 
  releases, 
  loading, 
  useChinaMirror, 
  onToggleChinaMirror,
  pcReleases = [],
  pcLoading = false
}: DownloadSectionProps) {
  const [showChangelog, setShowChangelog] = useState(false);
  const [platform, setPlatform] = useState<'pc' | 'mobile'>('pc');
  const latestRelease = releases.length > 0 ? releases[0] : null;
  const pcLatestRelease = pcReleases.length > 0 ? pcReleases[0] : null;
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  
  // 当前选择的平台的最新版本
  const currentLatestRelease = platform === 'pc' ? pcLatestRelease : latestRelease;
  const isCurrentLoading = platform === 'pc' ? pcLoading : loading;

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

      {/* Platform Switcher */}
      <div className="w-full max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-4 mb-4">
        <div className="flex justify-center space-x-4">
          <button
            onClick={() => setPlatform('pc')}
            className={`px-6 py-2.5 rounded-xl font-medium transition-all border ${
              platform === 'pc'
                ? 'bg-[#C8102E] text-white border-[#C8102E] shadow-lg shadow-[#C8102E]/30'
                : 'bg-white dark:bg-[#0a0a0a] text-slate-600 dark:text-slate-400 border-black/10 dark:border-white/10 hover:border-[#C8102E]/50'
            }`}
          >
            <span className="flex items-center space-x-2">
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
              <span className="font-mono text-sm">PC</span>
            </span>
          </button>
          <button
            onClick={() => setPlatform('mobile')}
            className={`px-6 py-2.5 rounded-xl font-medium transition-all border ${
              platform === 'mobile'
                ? 'bg-[#C8102E] text-white border-[#C8102E] shadow-lg shadow-[#C8102E]/30'
                : 'bg-white dark:bg-[#0a0a0a] text-slate-600 dark:text-slate-400 border-black/10 dark:border-white/10 hover:border-[#C8102E]/50'
            }`}
          >
            <span className="flex items-center space-x-2">
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />
              </svg>
              <span className="font-mono text-sm">Mobile</span>
            </span>
          </button>
        </div>
      </div>

      {/* Android Architecture Warning - Only show for mobile */}
      {platform === 'mobile' && (
        <div className="w-full max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-4 mb-4">
          <div className="bg-yellow-50 dark:bg-yellow-900/20 border-l-4 border-yellow-400 dark:border-yellow-600 p-4 rounded-r-lg">
            <div className="flex">
              <div className="flex-shrink-0">
                <svg className="h-5 w-5 text-yellow-400 dark:text-yellow-600" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                </svg>
              </div>
              <div className="ml-3">
                <p className="flex items-center text-sm text-yellow-800 dark:text-yellow-200">
                  {t.androidArchWarning}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      <div className="download-section bg-white dark:bg-[#0a0a0a] rounded-3xl border border-black/10 dark:border-white/10 overflow-hidden shadow-2xl relative transition-colors duration-300">
        {/* Decorative top bar */}
        <div className="h-1 w-full bg-gradient-to-r from-[#C8102E] via-[#FFC72C] to-[#00A651]"></div>
        
        {isCurrentLoading || !currentLatestRelease ? (
          <div className="p-16 text-center flex flex-col items-center justify-center">
            <Cpu className="w-8 h-8 text-[#C8102E] animate-pulse mb-4" />
            <p className="text-slate-500 dark:text-slate-400 font-mono text-sm animate-pulse">{t.fetching}</p>
          </div>
        ) : releases.length > 0 ? (
          <div>
            <div className="bg-black/[0.02] dark:bg-white/[0.02] p-6 sm:p-8 border-b border-black/5 dark:border-white/5 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
              <div>
                <div className="flex items-center space-x-3 mb-2">
                  <h3 className="text-2xl font-bold text-slate-900 dark:text-white tracking-tight">{currentLatestRelease?.name || currentLatestRelease?.tag_name}</h3>
                  {/* 测试版标识 */}
                  {(currentLatestRelease?.name || currentLatestRelease?.tag_name || '').toLowerCase().includes('demo') || 
                   (currentLatestRelease?.name || currentLatestRelease?.tag_name || '').toLowerCase().includes('beta') || 
                   (currentLatestRelease?.name || currentLatestRelease?.tag_name || '').toLowerCase().includes('test') ? (
                    <span className="bg-yellow-100 border border-yellow-200 text-yellow-800 text-[10px] px-2.5 py-1 rounded-full font-mono uppercase tracking-wider">
                      Test
                    </span>
                  ) : (
                    <span className="bg-[#00A651]/10 border border-[#00A651]/20 text-[#00A651] text-[10px] px-2.5 py-1 rounded-full font-mono uppercase tracking-wider">
                      {t.latest}
                    </span>
                  )}
                </div>
                <p className="text-xs font-mono text-slate-500">
                  {t.published}: {currentLatestRelease && new Date(currentLatestRelease.published_at).toISOString().split('T')[0]}
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
              {currentLatestRelease?.assets
                .filter((asset) => !asset.name.endsWith('.sig') && asset.name !== 'latest.json')
                .map((asset) => {
                  // 检测是否为 Android APK 文件
                  const isAndroidApk = asset.name.toLowerCase().endsWith('.apk');
                  const supportedArches = ['armeabi-v7a', 'arm64-v8a', 'x86_64'];
                  let archInfo = null;
                  
                  if (isAndroidApk) {
                    // 从文件名中提取架构信息
                    const matchedArch = supportedArches.find(arch => 
                      asset.name.toLowerCase().includes(arch.toLowerCase())
                    );
                    if (matchedArch) {
                      archInfo = matchedArch;
                    }
                  }
                  
                  return (
                    <div key={asset.name} className="p-4 sm:p-6 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors group">
                      <div className="flex items-center space-x-4">
                        <div className="w-12 h-12 bg-black/5 dark:bg-white/5 rounded-xl flex items-center justify-center text-slate-500 dark:text-slate-400 border border-black/5 dark:border-white/5 group-hover:border-black/10 dark:group-hover:border-white/10 group-hover:text-slate-900 dark:group-hover:text-white transition-colors">
                          {getPlatformIcon(asset.name)}
                        </div>
                        <div>
                          <p className="font-medium text-slate-800 dark:text-slate-200 break-all font-mono text-sm mb-1">{asset.name}</p>
                          <p className="text-xs font-mono text-slate-500">{t.size}: {formatBytes(asset.size)}</p>
                          {archInfo && (
                            <p className="text-xs font-mono text-[#C8102E] dark:text-[#ff4d6d] mt-1">
                              Architecture: {archInfo}
                            </p>
                          )}
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
                  );
                })}
              {currentLatestRelease && currentLatestRelease.assets.filter((asset) => !asset.name.endsWith('.sig') && asset.name !== 'latest.json').length === 0 && (
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
